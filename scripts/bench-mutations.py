#!/usr/bin/env python3
"""Compare mutation benchmarks on one runner; timings are not a CI gate."""

import csv
import io
import json
import os
from pathlib import Path
import shutil
import statistics
import subprocess
import sys


BENCH = "source_mutations"
FIELDS = ["label", "case", "path", "iterations", "elapsed_ns", "ns_per_op", "output_len"]


def command(args, cwd, timeout=900):
    result = subprocess.run(
        args, cwd=cwd, check=True, text=True, stdout=subprocess.PIPE, timeout=timeout
    )
    return result.stdout


def executable(root):
    output = command(
        ["cargo", "bench", "--locked", "--bench", BENCH, "--no-run", "--message-format=json"],
        root,
    )
    matches = []
    for line in output.splitlines():
        entry = json.loads(line)
        if (
            entry.get("reason") == "compiler-artifact"
            and entry.get("target", {}).get("name") == BENCH
            and entry.get("executable")
        ):
            matches.append(entry["executable"])
    if len(matches) != 1:
        raise RuntimeError(f"expected one {BENCH} executable, got {matches!r}")
    return matches[0]


def main():
    baseline, candidate, output = (Path(arg).resolve() for arg in sys.argv[1:])
    output.mkdir(parents=True, exist_ok=False)
    shutil.copyfile(candidate / "benches" / f"{BENCH}.rs", baseline / "benches" / f"{BENCH}.rs")
    manifest = baseline / "Cargo.toml"
    text = manifest.read_text()
    if f'name = "{BENCH}"' not in text:
        manifest.write_text(text + f'\n[[bench]]\nname = "{BENCH}"\nharness = false\n')
    command(["cargo", "generate-lockfile"], candidate)
    shutil.copyfile(candidate / "Cargo.lock", baseline / "Cargo.lock")
    shutil.copyfile(candidate / "Cargo.lock", output / "Cargo.lock")
    versions = command(["rustc", "--version", "--verbose"], candidate)
    versions += command(["cargo", "--version"], candidate)
    versions += f"RUSTFLAGS={os.environ.get('RUSTFLAGS', '')}\n"
    versions += "features=default; profile=bench; execution=serial\n"
    (output / "environment.txt").write_text(versions)
    print(versions, flush=True)
    roots = {"baseline": baseline, "candidate": candidate}
    binaries = {label: executable(root) for label, root in roots.items()}
    samples = {label: {} for label in roots}
    expected_keys = None
    lengths = {}
    for repetition in range(3):
        order = ["baseline", "candidate"] if repetition % 2 == 0 else ["candidate", "baseline"]
        for label in order:
            raw = command([binaries[label], label], roots[label], timeout=30)
            (output / f"{label}-{repetition}.csv").write_text(raw)
            reader = csv.DictReader(io.StringIO(raw))
            if reader.fieldnames != FIELDS:
                raise RuntimeError(f"unexpected CSV header: {reader.fieldnames!r}")
            keys = set()
            for row in reader:
                key = (row["case"], row["path"])
                if row["label"] != label or key in keys:
                    raise RuntimeError(f"bad or duplicate benchmark row: {row!r}")
                keys.add(key)
                timing = float(row["ns_per_op"])
                if timing <= 0 or int(row["iterations"]) <= 0:
                    raise RuntimeError(f"invalid timing: {row!r}")
                length = int(row["output_len"])
                if lengths.setdefault(key, length) != length:
                    raise RuntimeError(f"output length changed for {key!r}")
                samples[label].setdefault(key, []).append(timing)
            if not keys or (expected_keys is not None and keys != expected_keys):
                raise RuntimeError("benchmark case sets differ")
            expected_keys = keys
    report = [
        "## Source mutation benchmark",
        "",
        "Same runner/compiler/lockfile; median of three alternating runs. Positive change means faster.",
        "Shared-runner wall-clock results are informational, not a pass/fail gate.",
        "Exact bytes and independent semantic trees are asserted outside each timed sample.",
        "",
        "| Case | Path | Baseline ns/op | Candidate ns/op | Change |",
        "| --- | --- | ---: | ---: | ---: |",
    ]
    for key in sorted(expected_keys):
        before = statistics.median(samples["baseline"][key])
        after = statistics.median(samples["candidate"][key])
        change = (before - after) / before * 100
        report.append(f"| {key[0]} | {key[1]} | {before:.1f} | {after:.1f} | {change:+.1f}% |")
    summary = "\n".join(report) + "\n"
    (output / "summary.md").write_text(summary)
    print(summary)
    if os.environ.get("GITHUB_STEP_SUMMARY"):
        with open(os.environ["GITHUB_STEP_SUMMARY"], "a") as destination:
            destination.write(summary)


if __name__ == "__main__":
    main()
