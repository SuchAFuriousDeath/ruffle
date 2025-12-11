//! macOS-specific application delegate for handling file open events.
//!
//! This module implements a custom `NSApplicationDelegate` to receive file/URL
//! open events when users double-click SWF files associated with Ruffle in Finder,
//! or use "Open With" on macOS.

use crate::custom_event::RuffleEvent;
use crate::player::LaunchOptions;
use crate::preferences::GlobalPreferences;
use crate::util::parse_url;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{MainThreadMarker, MainThreadOnly, define_class};
use objc2_app_kit::{NSApplication, NSApplicationDelegate};
use objc2_foundation::{NSArray, NSObject, NSObjectProtocol, NSString, NSURL};
use std::cell::RefCell;
use std::path::Path;
use std::sync::OnceLock;
use url::Url;
use winit::event_loop::{EventLoopClosed, EventLoopProxy};

/// Stores the event loop proxy and preferences for the delegate.
struct DelegateContext {
    proxy: EventLoopProxy<RuffleEvent>,
    preferences: GlobalPreferences,
}

thread_local! {
    static DELEGATE_CONTEXT: RefCell<Option<DelegateContext>> = const { RefCell::new(None) };
}

static DELEGATE: OnceLock<Retained<RuffleAppDelegate>> = OnceLock::new();

/// Helper function to send an open event to the event loop.
fn send_open_event(url: Url, ctx: &DelegateContext) -> Result<(), EventLoopClosed<RuffleEvent>> {
    let event = RuffleEvent::Open(url, Box::new(LaunchOptions::from(&ctx.preferences)));

    ctx.proxy.send_event(event)
}

/// Helper function to process a single NSURL and send it to the event loop.
fn process_url(url: &NSURL, ctx: &DelegateContext) {
    // Use the safe absoluteString() method
    if let Some(url_string) = url.absoluteString() {
        let url_string = url_string.to_string();

        match Url::parse(&url_string) {
            Ok(url) => {
                let _ = send_open_event(url, ctx);
            }
            Err(e) => tracing::error!("Failed to parse URL '{}': {}", url_string, e),
        }
    }
}

/// Helper function to process a file path and send it to the event loop.
fn process_file_path(path: &str, ctx: &DelegateContext) {
    tracing::info!("Opening file: {}", path);

    match parse_url(Path::new(path)) {
        Ok(url) => {
            let _ = send_open_event(url, ctx);
        }
        Err(e) => tracing::error!("Failed to parse path '{}': {}", path, e),
    }
}

/// Helper function to access the delegate context within an autorelease pool.
fn with_context<F>(f: F)
where
    F: FnOnce(&DelegateContext),
{
    objc2::rc::autoreleasepool(|_| {
        DELEGATE_CONTEXT.with(|ctx| {
            let ctx = ctx.borrow();

            if let Some(ctx) = ctx.as_ref() {
                f(ctx);
            } else {
                tracing::warn!("Received file open request but delegate context is not set");
            }
        });
    });
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "RuffleAppDelegate"]
    struct RuffleAppDelegate;

    unsafe impl NSObjectProtocol for RuffleAppDelegate {}

    unsafe impl NSApplicationDelegate for RuffleAppDelegate {
        /// Called when the application receives URLs to open (after app launch).
        /// Ruffle only supports one movie at a time, so we open only the first URL.
        #[unsafe(method(application:openURLs:))]
        fn application_open_urls(&self, _application: &NSApplication, urls: &NSArray<NSURL>) {
            tracing::debug!(
                "application:openURLs: received {} URL(s), opening first only",
                urls.len()
            );

            with_context(|ctx| {
                if let Some(url) = urls.first() {
                    process_url(&url, ctx);
                }
            });
        }

        /// Called when the application is asked to open a single file (during launch).
        /// This is called when the user double-clicks a file while the app is not running.
        /// Returns true if the file was successfully handled.
        #[unsafe(method(application:openFile:))]
        fn application_open_file(&self, _application: &NSApplication, filename: &NSString) -> bool {
            tracing::debug!("application:openFile: received file: {}", filename);

            let mut success = false;
            with_context(|ctx| {
                let path = filename.to_string();
                if parse_url(Path::new(&path)).is_ok() {
                    process_file_path(&path, ctx);
                    success = true;
                }
            });

            success
        }

        /// Called when the application is asked to open multiple files (during launch).
        /// Ruffle only supports one movie at a time, so we open only the first file.
        #[unsafe(method(application:openFiles:))]
        fn application_open_files(
            &self,
            _application: &NSApplication,
            filenames: &NSArray<NSString>,
        ) {
            tracing::debug!(
                "application:openFiles: received {} file(s), opening first only",
                filenames.len()
            );

            with_context(|ctx| {
                if let Some(filename) = filenames.first() {
                    process_file_path(&filename.to_string(), ctx);
                }
            });
        }
    }
);

impl RuffleAppDelegate {
    /// Creates a new instance of the Ruffle application delegate.
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { objc2::msg_send![super(Self::alloc(mtm).set_ivars(())), init] }
    }
}

/// Registers the Ruffle application delegate with the shared NSApplication.
///
/// This should be called early in the application lifecycle, after the event
/// loop is created but before it starts running, to ensure file open events
/// are properly received.
///
/// # Arguments
/// * `event_loop_proxy` - The winit event loop proxy used to send `RuffleEvent`s
/// * `preferences` - The global preferences used to create `LaunchOptions`
///
/// # Panics
/// Panics if not called from the main thread.
pub fn register_delegate(event_loop_proxy: EventLoopProxy<RuffleEvent>, preferences: GlobalPreferences) {
    // Store the context in thread-local storage
    DELEGATE_CONTEXT.with(|ctx| {
        *ctx.borrow_mut() = Some(DelegateContext {
            proxy: event_loop_proxy,
            preferences,
        });
    });

    // Get the main thread marker - this will panic if not on main thread
    let mtm = MainThreadMarker::new().expect("Must be called from the main thread");

    // Create and store our delegate in static storage
    let delegate = DELEGATE.get_or_init(|| RuffleAppDelegate::new(mtm));

    // Get the shared application and set our delegate
    // Important: This must be called after EventLoop::new() per winit docs
    let app = NSApplication::sharedApplication(mtm);
    app.setDelegate(Some(ProtocolObject::from_ref(delegate.deref())));

    tracing::info!("Registered macOS application delegate for file associations");
}
