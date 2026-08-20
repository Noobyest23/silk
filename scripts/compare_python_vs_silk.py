#!/usr/bin/env python3
"""Run the Python benchmark and the Silk benchmark and compare elapsed time."""

from __future__ import annotations

import argparse
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PYTHON_BENCH = ROOT / "scripts" / "benchmark_python_run_all_tests.py"
SILK_SCRIPT = ROOT / "scripts" / "test_all_in_one.silk"
SILK_BIN = ROOT / "target" / "release" / "loom"


def run_python_benchmark(repeat: int) -> tuple[float, int, str]:
    cmd = [sys.executable, str(PYTHON_BENCH), "--repeat", str(repeat)]
    start = time.perf_counter()
    result = subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True)
    elapsed = time.perf_counter() - start
    output = (result.stdout or "") + (result.stderr or "")
    return elapsed, result.returncode, output


def run_silk_benchmark(repeat: int) -> tuple[float, int, str]:
    if not SILK_BIN.exists():
        raise FileNotFoundError(f"Silk binary not found at {SILK_BIN}. Build it first with: cargo build --release")

    cmd = [str(SILK_BIN), "run", str(SILK_SCRIPT)]
    total_elapsed = 0.0
    last_output = ""
    last_code = 0

    for _ in range(repeat):
        start = time.perf_counter()
        result = subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True)
        total_elapsed += time.perf_counter() - start
        last_output = (result.stdout or "") + (result.stderr or "")
        last_code = result.returncode
        if result.returncode != 0:
            break

    return total_elapsed, last_code, last_output


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Compare execution time of the Python benchmark against Silk's run_all_tests.silk"
    )
    parser.add_argument("--repeat", type=int, default=1, help="Number of times to run each benchmark")
    args = parser.parse_args()

    if args.repeat < 1:
        raise SystemExit("--repeat must be >= 1")

    print(f"Running Python benchmark {args.repeat} time(s)...")
    py_time, py_code, py_output = run_python_benchmark(args.repeat)
    print(f"Python benchmark finished in {py_time:.6f}s (exit code {py_code})")

    print(f"\nRunning Silk benchmark {args.repeat} time(s)...")
    silk_time, silk_code, silk_output = run_silk_benchmark(args.repeat)
    print(f"Silk benchmark finished in {silk_time:.6f}s (exit code {silk_code})")

    print("\n=== Comparison ===")
    print(f"Python: {py_time:.6f}s")
    print(f"Silk:   {silk_time:.6f}s")
    print(f"Ratio (Python / Silk): {py_time / silk_time if silk_time else float('inf'):.3f}x")

    if py_code != 0:
        print("\nPython benchmark exited with a non-zero code.")
        print(py_output[:2000])
    if silk_code != 0:
        print("\nSilk benchmark exited with a non-zero code.")
        print(silk_output[:2000])


if __name__ == "__main__":
    main()
