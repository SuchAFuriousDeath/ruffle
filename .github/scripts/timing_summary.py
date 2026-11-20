#!/usr/bin/env python3
"""Summarizes where the time of a CI test job went.

Reads the report of `cargo --timings` and the JUnit report of nextest, and
prints a Markdown summary, meant for `$GITHUB_STEP_SUMMARY`:

    timing_summary.py <cargo-timing.html> <junit.xml>

Either report may be missing, e.g. when the build failed. As cargo only
writes its report as HTML on stable, the unit data is extracted from the
page; if that stops working with a newer cargo, a note is printed instead.
"""

import json
import re
import subprocess
import sys
import xml.etree.ElementTree as ET
from collections import defaultdict


def fmt(seconds):
    seconds = round(seconds)
    if seconds >= 60:
        return f"{seconds // 60}m {seconds % 60:02}s"
    return f"{seconds}s"


def workspace_packages():
    try:
        metadata = subprocess.run(
            ["cargo", "metadata", "--no-deps", "--format-version", "1"],
            capture_output=True,
            text=True,
            check=True,
        ).stdout
        return {
            (package["name"], package["version"])
            for package in json.loads(metadata)["packages"]
        }
    except (OSError, subprocess.CalledProcessError, ValueError, KeyError):
        return set()


def unit_label(unit):
    # Cargo describes targets like ` ruffle_core "lib" (test)`.
    target = unit["target"].strip()
    if not target:
        return unit["name"]
    parts = re.findall(r'"([^"]*)"', target) + re.findall(r"\(([^)]*)\)", target)
    return f"{unit['name']} ({' '.join(dict.fromkeys(parts)) or target})"


class Node:
    """A line of the summary tree."""

    def __init__(self, label, seconds, note="", children=()):
        self.label = label
        self.seconds = seconds
        self.note = note
        self.children = list(children)


def render(nodes):
    """Renders nodes as a tree, with the times aligned in a column."""
    rows = []

    def walk(node, prefix, child_prefix):
        rows.append((prefix + node.label, fmt(node.seconds), node.note))
        for i, child in enumerate(node.children):
            last = i == len(node.children) - 1
            walk(
                child,
                child_prefix + ("└── " if last else "├── "),
                child_prefix + ("    " if last else "│   "),
            )

    for node in nodes:
        walk(node, "", "")

    label_width = max(len(label) for label, _, _ in rows)
    time_width = max(len(time) for _, time, _ in rows)
    return [
        f"{label:<{label_width}}  {time:>{time_width}}  {note}".rstrip()
        for label, time, note in rows
    ]


def sections(unit):
    return ", ".join(
        f"{name} {fmt(section['end'] - section['start'])}"
        for name, section in unit.get("sections") or []
    )


def slowest(units, count=10):
    ordered = sorted(units, key=lambda unit: -unit["duration"])
    names = defaultdict(int)
    for unit in units:
        names[unit_label(unit)] += 1

    def label(unit):
        # Crates built more than once (in other versions, or for build
        # scripts) get their version, to tell them apart.
        name = unit_label(unit)
        return f"{name} {unit['version']}" if names[name] > 1 else name

    nodes = [Node(label(unit), unit["duration"], sections(unit)) for unit in ordered[:count]]
    rest = ordered[count:]
    if rest:
        nodes.append(
            Node(f"{len(rest)} more", sum(unit["duration"] for unit in rest))
        )
    return nodes


def build_tree(path):
    """Returns the build's node, or a note when there's no report."""
    try:
        with open(path, encoding="utf-8") as f:
            page = f.read()
    except OSError:
        return "No build timings (the build didn't finish)."

    match = re.search(r"const UNIT_DATA = (\[.*?\]);\n", page, re.S)
    if not match:
        return "Couldn't read the build timings (has cargo's report changed?)."

    units = json.loads(match.group(1))
    if not units:
        return "Nothing was built."

    workspace = workspace_packages()
    groups = defaultdict(list)

    for unit in units:
        target = unit["target"].strip()
        if (unit["name"], unit["version"]) not in workspace:
            groups["dependencies"].append(unit)
        elif target in ("", "(lib)") or target.startswith("build-script"):
            groups["workspace crates"].append(unit)
        else:
            groups["test and binary targets"].append(unit)

    wall = max(unit["start"] + unit["duration"] for unit in units)
    children = [
        Node(
            f"{name} (sum)",
            sum(unit["duration"] for unit in groups[name]),
            f"{len(groups[name])} units",
            slowest(groups[name]),
        )
        for name in ("dependencies", "workspace crates", "test and binary targets")
        if groups[name]
    ]
    return Node("Build (wall time)", wall, f"{len(units)} units", children)


def tests_tree(path):
    """Returns the tests' node, or a note when there's no report."""
    try:
        root = ET.parse(path).getroot()
    except (OSError, ET.ParseError):
        return "No test results (the tests didn't run)."

    groups = defaultdict(lambda: [0.0, 0])
    failed = skipped = 0

    for case in root.iter("testcase"):
        name = case.get("name", "")
        group = name.split("/")[0] if "/" in name else case.get("classname", "")
        groups[group][0] += float(case.get("time") or 0)
        groups[group][1] += 1
        failed += case.find("failure") is not None or case.find("error") is not None
        skipped += case.find("skipped") is not None

    count = sum(tests for _, tests in groups.values())
    ordered = sorted(groups.items(), key=lambda item: -item[1][0])
    children = [
        Node(f"{name} (sum)", time, f"{tests} tests") for name, (time, tests) in ordered[:10]
    ]
    rest = ordered[10:]
    if rest:
        children.append(
            Node(
                "other (sum)",
                sum(time for _, (time, _) in rest),
                f"{sum(tests for _, (_, tests) in rest)} tests",
            )
        )

    wall = float(root.get("time") or 0)
    note = f"{count} tests, {failed} failed, {skipped} skipped"
    return Node("Tests (wall time)", wall, note, children)


def main():
    if len(sys.argv) != 3:
        sys.exit(__doc__)

    # The tree uses box-drawing characters, which Windows' default encoding lacks.
    sys.stdout.reconfigure(encoding="utf-8")

    nodes = []
    notes = []
    for tree in (build_tree(sys.argv[1]), tests_tree(sys.argv[2])):
        if isinstance(tree, Node):
            nodes.append(tree)
        else:
            notes.append(tree)

    print("## Timings\n")
    if nodes:
        print("```")
        print("\n".join(render(nodes)))
        print("```\n")
    print(
        "The sums add up the times of units and tests that run in parallel, so they"
        " can be larger than the wall time. Sections: frontend is parsing and"
        " checking; for test and binary targets, codegen includes linking."
    )
    for note in notes:
        print(f"\n{note}")


if __name__ == "__main__":
    main()
