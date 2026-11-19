//! Runs the tests in a few long-lived worker processes.
//!
//! `cargo nextest` runs every test in its own process, which isolates crashes
//! and hangs, but makes every render test pay for setting up a GPU device and
//! compiling its pipelines again. `cargo test` shares those, but a single
//! crashing or hanging test takes the whole run down with it.
//!
//! Setting `RUFFLE_TEST_WORKERS=N` makes this binary a supervisor instead: it
//! lists the tests (honoring the usual libtest filter arguments), starts `N`
//! copies of itself in worker mode and hands them one test at a time. A worker
//! that crashes or exceeds the timeout is killed and replaced, and the test is
//! reported as failed (or retried, see below).
//!
//! Protocol: the supervisor writes one test name (with kind, as printed by
//! `--list`) per line to the worker's stdin. The worker runs it and answers on
//! stdout with a line containing [`MARKER`] followed by the outcome. Everything
//! else the worker writes to stdout or stderr (both go into the same pipe) is
//! captured as the output of the test that is currently running.
//!
//! Other environment variables:
//! - `RUFFLE_TEST_TIMEOUT`: per-test timeout in seconds (default 300).
//! - `RUFFLE_TEST_RETRIES`: how many times to retry failing tests (default 0).
//! - `RUFFLE_TEST_NEXTEST_PROFILE`: also apply the `retries` of this profile
//!   (and its `test(...)` overrides) from `.config/nextest.toml`.
//! - `RUFFLE_TEST_TIMINGS`: write a TSV file with the result of every test.

use libtest_mimic::{Arguments, ColorSetting, Trial};
use std::collections::VecDeque;
use std::fmt::Write as _;
use std::io::{BufRead, BufReader, Write};
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, channel};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

const WORKERS_VAR: &str = "RUFFLE_TEST_WORKERS";
const WORKER_VAR: &str = "RUFFLE_TEST_WORKER";
const SELF_TEST_VAR: &str = "RUFFLE_TEST_SUPERVISOR_SELF_TEST";

/// Prefixes the outcome lines of the worker protocol. The control characters
/// keep test output from accidentally looking like it.
const MARKER: &str = "\u{1}ruffle-test-worker\u{2} ";

pub enum Mode {
    /// Run the tests in this process, as usual.
    Direct,
    Supervisor {
        workers: NonZeroUsize,
    },
    Worker,
}

impl Mode {
    pub fn from_env() -> Self {
        if std::env::var_os(WORKER_VAR).is_some() {
            return Mode::Worker;
        }

        let args = Arguments::from_args();

        if args.list {
            return Mode::Direct;
        }

        match std::env::var(WORKERS_VAR).ok().and_then(|n| n.parse().ok()) {
            Some(workers) => Mode::Supervisor { workers },
            None => Mode::Direct,
        }
    }
}

/// The name of the trial as printed by `--list`, which unlike the plain name
/// is unique.
fn trial_key(trial: &Trial) -> String {
    if trial.kind().is_empty() {
        trial.name().to_owned()
    } else {
        format!("[{}] {}", trial.kind(), trial.name())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Status {
    Passed,
    Failed,
    Ignored,
    Crashed,
    TimedOut,
}

impl Status {
    fn as_str(self) -> &'static str {
        match self {
            Status::Passed => "passed",
            Status::Failed => "failed",
            Status::Ignored => "ignored",
            Status::Crashed => "crashed",
            Status::TimedOut => "timed-out",
        }
    }

    fn parse(s: &str) -> Option<Self> {
        [Status::Passed, Status::Failed, Status::Ignored]
            .into_iter()
            .find(|status| status.as_str() == s)
    }
}

/// Runs the tests requested on stdin, one at a time, until stdin is closed.
///
/// `find_trials` returns the trials matching the given key; it's called
/// for every request, as running a trial consumes it.
pub fn serve(args: &Arguments, find_trials: impl Fn(&str) -> Vec<Trial>) -> i32 {
    let run_args = Arguments {
        test_threads: Some(1),
        quiet: true,
        color: Some(ColorSetting::Never),
        include_ignored: args.include_ignored,
        ignored: args.ignored,
        ..Arguments::default()
    };

    for key in std::io::stdin().lock().lines() {
        let Ok(key) = key else { break };
        let trial = find_trials(&key)
            .into_iter()
            .find(|trial| trial_key(trial) == key);

        let status = match trial {
            Some(trial) => {
                // Run the test on a new thread like libtest does, as some tests
                // need more stack than the main thread has on Windows (1 MiB).
                let conclusion = std::thread::scope(|scope| {
                    scope
                        .spawn(|| libtest_mimic::run(&run_args, vec![trial]))
                        .join()
                        .expect("libtest-mimic catches test panics")
                });

                if conclusion.num_failed > 0 {
                    Status::Failed
                } else if conclusion.num_ignored > 0 {
                    Status::Ignored
                } else {
                    Status::Passed
                }
            }
            None => {
                println!("No test named {key:?}");
                Status::Failed
            }
        };

        let _ = std::io::stderr().flush();
        let mut stdout = std::io::stdout().lock();
        let _ = writeln!(stdout, "\n{MARKER}{}", status.as_str());
        let _ = stdout.flush();
    }

    0
}

struct Config {
    timeout: Duration,
    retries: RetryPolicy,
    timings: Option<PathBuf>,
}

impl Config {
    fn from_env() -> Self {
        let timeout = std::env::var("RUFFLE_TEST_TIMEOUT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(300);

        Self {
            timeout: Duration::from_secs(timeout),
            retries: RetryPolicy::from_env(),
            timings: std::env::var_os("RUFFLE_TEST_TIMINGS").map(PathBuf::from),
        }
    }
}

#[derive(Default)]
struct RetryPolicy {
    default: u32,
    /// Test name substrings, mirroring nextest's `test(...)` filters.
    overrides: Vec<(Vec<String>, u32)>,
}

impl RetryPolicy {
    fn from_env() -> Self {
        let mut policy = RetryPolicy::default();

        if let Ok(profile) = std::env::var("RUFFLE_TEST_NEXTEST_PROFILE") {
            let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../.config/nextest.toml");
            let config = std::fs::read_to_string(path).expect("Couldn't read nextest config");
            policy = Self::from_nextest_config(&config, &profile);
        }

        if let Some(retries) = std::env::var("RUFFLE_TEST_RETRIES")
            .ok()
            .and_then(|s| s.parse().ok())
        {
            policy.default = retries;
        }

        policy
    }

    /// Understands just enough of the nextest config for our overrides:
    /// filters of the form `test(a) or test(b)`.
    fn from_nextest_config(config: &str, profile: &str) -> Self {
        let config: toml::Table = toml::from_str(config).expect("Invalid nextest config");
        let profile = config
            .get("profile")
            .and_then(|p| p.get(profile))
            .and_then(|p| p.as_table());

        let Some(profile) = profile else {
            return RetryPolicy::default();
        };

        let retries = |value: &toml::Value| {
            value
                .get("retries")
                .and_then(|r| r.as_integer())
                .map(|r| r as u32)
        };

        let default = retries(&toml::Value::Table(profile.clone())).unwrap_or(0);
        let overrides = profile
            .get("overrides")
            .and_then(|o| o.as_array())
            .into_iter()
            .flatten()
            .filter_map(|o| {
                let filter = o.get("filter")?.as_str()?;
                let patterns = filter
                    .split(" or ")
                    .map(|part| {
                        let part = part.trim();
                        part.strip_prefix("test(")
                            .and_then(|p| p.strip_suffix(')'))
                            .unwrap_or_else(|| panic!("Unsupported nextest filter: {part}"))
                            .to_owned()
                    })
                    .collect();
                Some((patterns, retries(o)?))
            })
            .collect();

        RetryPolicy { default, overrides }
    }

    fn retries_for(&self, key: &str) -> u32 {
        self.overrides
            .iter()
            .find(|(patterns, _)| patterns.iter().any(|p| key.contains(p.as_str())))
            .map_or(self.default, |(_, retries)| *retries)
    }
}

struct Job {
    key: String,
    attempts: Vec<Attempt>,
}

struct Attempt {
    status: Status,
    output: String,
    duration: Duration,
    worker: usize,
}

struct Queue {
    jobs: VecDeque<Job>,
    in_flight: usize,
}

/// Runs `trials` in `workers` worker processes and prints the results.
pub fn supervise(args: &Arguments, trials: Vec<Trial>, workers: NonZeroUsize) -> i32 {
    let start = Instant::now();
    let config = Arc::new(Config::from_env());

    let mut num_filtered_out = 0;
    let mut finished = Vec::new();
    let mut jobs = VecDeque::new();

    for trial in &trials {
        if args.is_filtered_out(trial) {
            num_filtered_out += 1;
            continue;
        }

        let job = Job {
            key: trial_key(trial),
            attempts: Vec::new(),
        };

        if args.is_ignored(trial) {
            finished.push(job);
        } else {
            jobs.push_back(job);
        }
    }

    println!(
        "\nrunning {} tests in {workers} worker processes",
        jobs.len() + finished.len()
    );

    for job in &finished {
        println!("test {} ... ignored", job.key);
    }

    let workers = workers.get().min(jobs.len());
    let queue = Arc::new((Mutex::new(Queue { jobs, in_flight: 0 }), Condvar::new()));
    let (results_tx, results_rx) = channel();

    let threads: Vec<_> = (0..workers)
        .map(|id| {
            let queue = queue.clone();
            let config = config.clone();
            let results = results_tx.clone();
            std::thread::spawn(move || drive_worker(id, &queue, &config, &results))
        })
        .collect();
    drop(results_tx);

    for job in results_rx {
        print_result(&job);
        finished.push(job);
    }

    for thread in threads {
        thread.join().expect("Worker thread panicked");
    }

    finished.sort_by(|a, b| a.key.cmp(&b.key));
    summarize(&finished, num_filtered_out, start.elapsed(), &config)
}

fn final_status(job: &Job) -> Status {
    job.attempts.last().map_or(Status::Ignored, |a| a.status)
}

fn print_result(job: &Job) {
    let status = match final_status(job) {
        Status::Passed if job.attempts.len() > 1 => {
            format!("ok (flaky, passed on attempt {})", job.attempts.len())
        }
        Status::Passed => "ok".to_owned(),
        Status::Ignored => "ignored".to_owned(),
        Status::Failed => "FAILED".to_owned(),
        Status::Crashed => "CRASHED".to_owned(),
        Status::TimedOut => "TIMED OUT".to_owned(),
    };
    let retries = if job.attempts.len() > 1 && final_status(job) != Status::Passed {
        format!(" ({} attempts)", job.attempts.len())
    } else {
        String::new()
    };
    let duration = job.attempts.iter().map(|a| a.duration).sum::<Duration>();
    println!(
        "test {} ... {status}{retries} [{:.2}s]",
        job.key,
        duration.as_secs_f64()
    );
}

fn summarize(jobs: &[Job], num_filtered_out: usize, elapsed: Duration, config: &Config) -> i32 {
    let count = |f: &dyn Fn(&Job) -> bool| jobs.iter().filter(|job| f(job)).count();
    let failed: Vec<_> = jobs
        .iter()
        .filter(|job| !matches!(final_status(job), Status::Passed | Status::Ignored))
        .collect();
    let flaky = count(&|job| final_status(job) == Status::Passed && job.attempts.len() > 1);

    if let Some(path) = &config.timings {
        let mut tsv = String::from("test\tstatus\tattempts\tseconds\tworker\n");
        for job in jobs {
            for (i, attempt) in job.attempts.iter().enumerate() {
                let _ = writeln!(
                    tsv,
                    "{}\t{}\t{}\t{:.3}\t{}",
                    job.key,
                    attempt.status.as_str(),
                    i + 1,
                    attempt.duration.as_secs_f64(),
                    attempt.worker
                );
            }
        }
        std::fs::write(path, tsv).expect("Couldn't write timings");
    }

    if !failed.is_empty() {
        println!("\nfailures:");
        for job in &failed {
            for (i, attempt) in job.attempts.iter().enumerate() {
                println!(
                    "\n---- {} (attempt {}, {}) ----\n{}",
                    job.key,
                    i + 1,
                    attempt.status.as_str(),
                    attempt.output.trim_end()
                );
            }
        }

        println!("\nfailures:");
        for job in &failed {
            println!("    {}", job.key);
        }
    }

    println!(
        "\ntest result: {}. {} passed; {} failed; {} ignored; {flaky} flaky; {num_filtered_out} filtered out; finished in {:.2}s\n",
        if failed.is_empty() { "ok" } else { "FAILED" },
        count(&|job| final_status(job) == Status::Passed),
        failed.len(),
        count(&|job| final_status(job) == Status::Ignored),
        elapsed.as_secs_f64(),
    );

    if failed.is_empty() { 0 } else { 101 }
}

/// Takes jobs from the queue and runs them in a worker process, which is
/// (re)started as needed. Returns once there's nothing left to do.
fn drive_worker(
    id: usize,
    queue: &(Mutex<Queue>, Condvar),
    config: &Config,
    results: &Sender<Job>,
) {
    let (lock, condvar) = queue;
    let mut worker = None;

    loop {
        let mut job = {
            let mut queue = lock.lock().unwrap();

            // A job in flight elsewhere may still come back for a retry.
            while queue.jobs.is_empty() && queue.in_flight > 0 {
                queue = condvar.wait(queue).unwrap();
            }

            let Some(job) = queue.jobs.pop_front() else {
                break;
            };

            queue.in_flight += 1;
            job
        };

        let worker_process = worker.get_or_insert_with(Worker::spawn);
        let attempt = worker_process.run(&job.key, config.timeout, id);

        if matches!(attempt.status, Status::Crashed | Status::TimedOut) {
            worker = None;
        }

        let retry = matches!(
            attempt.status,
            Status::Failed | Status::Crashed | Status::TimedOut
        ) && job.attempts.len() < config.retries.retries_for(&job.key) as usize;
        job.attempts.push(attempt);

        let mut queue = lock.lock().unwrap();
        queue.in_flight -= 1;

        if retry {
            queue.jobs.push_back(job);
        } else {
            let _ = results.send(job);
        }

        condvar.notify_all();
    }

    if let Some(worker) = worker {
        worker.finish();
    }
}

enum Line {
    Output(String),
    Done(String),
}

struct Worker {
    child: Child,
    stdin: ChildStdin,
    lines: Receiver<Line>,
}

impl Worker {
    fn spawn() -> Self {
        let (reader, writer) = std::io::pipe().expect("Couldn't create pipe");

        let mut child = {
            // The command keeps its copies of the pipe open until it's dropped,
            // and then the reader would never see the end of the output.
            let mut command = Command::new(std::env::current_exe().unwrap());
            command
                .args(std::env::args_os().skip(1))
                .env(WORKER_VAR, "1")
                .env_remove(WORKERS_VAR)
                .stdin(Stdio::piped())
                .stdout(writer.try_clone().expect("Couldn't clone pipe"))
                .stderr(writer);
            command.spawn().expect("Couldn't start worker process")
        };

        let stdin = child.stdin.take().unwrap();
        let (tx, lines) = channel();

        std::thread::spawn(move || {
            let mut reader = BufReader::new(reader);
            let mut buf = Vec::new();

            while let Ok(n) = reader.read_until(b'\n', &mut buf) {
                if n == 0 {
                    break;
                }

                let line = String::from_utf8_lossy(&buf).into_owned();
                buf.clear();

                let line = match line.split_once(MARKER) {
                    Some((output, status)) => {
                        if !output.is_empty() {
                            let _ = tx.send(Line::Output(output.to_owned()));
                        }
                        Line::Done(status.trim().to_owned())
                    }
                    None => Line::Output(line),
                };

                if tx.send(line).is_err() {
                    break;
                }
            }
        });

        Self {
            child,
            stdin,
            lines,
        }
    }

    fn run(&mut self, key: &str, timeout: Duration, worker: usize) -> Attempt {
        let start = Instant::now();
        let mut output = String::new();

        let status = if writeln!(self.stdin, "{key}")
            .and_then(|_| self.stdin.flush())
            .is_err()
        {
            self.crashed(&mut output)
        } else {
            loop {
                let remaining = timeout.saturating_sub(start.elapsed());

                match self.lines.recv_timeout(remaining) {
                    Ok(Line::Output(line)) => output.push_str(&line),
                    Ok(Line::Done(status)) => {
                        break Status::parse(&status).unwrap_or(Status::Failed);
                    }
                    Err(RecvTimeoutError::Timeout) => {
                        let _ = self.child.kill();
                        let _ = self.child.wait();
                        let _ = writeln!(
                            output,
                            "\nKilled the worker after {:.0}s",
                            timeout.as_secs_f64()
                        );
                        break Status::TimedOut;
                    }
                    Err(RecvTimeoutError::Disconnected) => break self.crashed(&mut output),
                }
            }
        };

        Attempt {
            status,
            output,
            duration: start.elapsed(),
            worker,
        }
    }

    /// Lets the worker exit on its own and waits for it.
    fn finish(self) {
        drop(self.stdin);
        let mut child = self.child;
        let _ = child.wait();
    }

    fn crashed(&mut self, output: &mut String) -> Status {
        // Collect whatever the worker printed before it died.
        while let Ok(Line::Output(line)) = self.lines.recv_timeout(Duration::from_secs(5)) {
            output.push_str(&line);
        }

        let exit_status = self.child.wait();
        let _ = writeln!(output, "\nThe worker process died: {exit_status:?}");
        Status::Crashed
    }
}

/// Tests for the supervisor itself, which only exist when
/// `RUFFLE_TEST_SUPERVISOR_SELF_TEST` is set to a scratch directory.
pub fn self_tests() -> Vec<Trial> {
    let Some(dir) = std::env::var_os(SELF_TEST_VAR).map(PathBuf::from) else {
        return Vec::new();
    };

    vec![
        Trial::test("supervisor_self_test/abort", || {
            println!("About to abort");
            std::process::abort()
        }),
        Trial::test("supervisor_self_test/hang", || {
            println!("About to hang");
            loop {
                std::thread::sleep(Duration::from_secs(1));
            }
        }),
        Trial::test("supervisor_self_test/panic", || {
            panic!("Panicking on purpose")
        }),
        Trial::test("supervisor_self_test/flaky", move || {
            let marker = dir.join("flaky");

            if marker.exists() {
                Ok(())
            } else {
                std::fs::write(&marker, "").unwrap();
                Err("Failing the first attempt on purpose".into())
            }
        }),
    ]
}
