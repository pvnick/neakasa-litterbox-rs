"""Run sequential, interleaved benchmarks; emit raw samples without private paths.

Run with the Python interpreter containing the pinned reference SDK.
Uses no accounts, state files, network calls, or device operations.
"""
import argparse
import datetime
import hashlib
import importlib.metadata
import json
import os
from pathlib import Path
import platform
import random
import statistics
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]
CASES = {"session_token": 200_000, "iot_signing": 50_000,
         "status_json": 20_000, "push_json": 20_000}

def run(command):
    return subprocess.check_output(command, cwd=ROOT, text=True, stderr=subprocess.PIPE)

def summary(values):
    quartiles = statistics.quantiles(values, n=4, method="inclusive")
    return {"median": statistics.median(values), "q1": quartiles[0],
            "q3": quartiles[2], "min": min(values), "max": max(values), "samples": values}

def peak_rss(command):
    # A small native launcher prevents the Python runner's pre-exec RSS floor
    # from inflating the Rust child's peak. wait4 measures the CLI, not the launcher.
    return float(run([str(ROOT / "target/release/examples/benchmark_rss"), *command])) / 1024

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cpu", type=int, required=True, help="Linux CPU number for both implementations")
    parser.add_argument("--machine", required=True, help="Public CPU/model description; do not enter hostnames")
    parser.add_argument("--samples", type=int, default=15)
    parser.add_argument("--startup-samples", type=int, default=31)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    assert args.samples >= 3 and args.startup_samples >= 3
    os.sched_setaffinity(0, {args.cpu})  # Children, including runtime threads, inherit this.
    rng = random.Random(20261016)
    rust = ROOT / "target/release/examples/benchmark"
    workers = {"python": [sys.executable, str(ROOT / "benchmarks/python_worker.py")],
               "rust": [str(rust)]}
    cli = {"python": [sys.executable, str(ROOT / "benchmarks/python_api.py"), "--help"],
           "rust": [str(ROOT / "target/release/neakasa-api"), "--help"]}
    sdk = importlib.metadata.distribution("neakasa-litterbox-sdk")
    direct = json.loads(sdk.read_text("direct_url.json") or "{}")
    cpu_path = Path(f"/sys/devices/system/cpu/cpu{args.cpu}/cpufreq")
    metadata = {
        "date_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
        "machine": args.machine, "architecture": platform.machine(),
        "os": platform.system(), "kernel": platform.release(), "cpu": args.cpu,
        "governor": (cpu_path / "scaling_governor").read_text().strip(),
        "max_frequency_khz": int((cpu_path / "cpuinfo_max_freq").read_text()),
        "python": platform.python_version(), "python_sdk_version": sdk.version,
        "python_sdk_commit": direct.get("vcs_info", {}).get("commit_id"),
        "cryptography": importlib.metadata.version("cryptography"),
        "rustc": run([str(Path.home() / ".cargo/bin/rustc"), "--version"]).strip(),
        "sdk_base_commit": run(["git", "rev-parse", "HEAD"]).strip(),
        "build": "cargo build --release --locked --bin neakasa-api --example benchmark --example benchmark_rss",
        "rustflags": os.environ.get("RUSTFLAGS", ""),
        "batch_samples": args.samples, "startup_samples": args.startup_samples,
        "warmup_operations_per_batch": 1000, "seed": 20261016,
        "source_sha256": {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest()
                          for p in sorted((ROOT / "benchmarks").glob("*")) if p.suffix in {".py", ".rs"}},
    }
    results = {"metadata": metadata, "benchmarks": {}}
    for case, iterations in CASES.items():
        values = {"python": [], "rust": []}
        reference = None
        for _ in range(args.samples):
            order = list(workers)
            rng.shuffle(order)
            for language in order:
                sample = json.loads(run(workers[language] + [case, str(iterations)]))
                compared = (sample["validation"], sample["input_bytes"])
                if reference is None:
                    reference = compared
                assert compared == reference, f"output mismatch in {case}"
                values[language].append(sample["elapsed_ns"] / iterations)
        results["benchmarks"][case] = {"unit": "ns/op", "iterations_per_batch": iterations,
            "input_bytes": reference[1], "equivalent_outputs": True,
            **{lang: summary(v) for lang, v in values.items()}}
        print(f"Completed {case}", flush=True)
    # Separate fresh-process startup measurements. OS page caches stay warm.
    for command in cli.values():
        for _ in range(5):
            subprocess.run(command, cwd=ROOT, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, check=True)
    values = {"python": [], "rust": []}
    for _ in range(args.startup_samples):
        order = list(cli)
        rng.shuffle(order)
        for language in order:
            start = time.perf_counter_ns()
            subprocess.run(cli[language], cwd=ROOT, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, check=True)
            values[language].append((time.perf_counter_ns() - start) / 1e6)
    results["benchmarks"]["cli_help"] = {"unit": "ms/process", **{lang: summary(v) for lang,v in values.items()}}
    args.output.write_text(json.dumps(results, indent=2) + "\n")
    # Separate per-process peak RSS measurements; no timing instrumentation.
    values = {"python": [], "rust": []}
    for _ in range(7):
        order = list(cli)
        rng.shuffle(order)
        for language in order:
            values[language].append(peak_rss(cli[language]))
    results["benchmarks"]["cli_peak_rss"] = {"unit": "MiB/process", **{lang: summary(v) for lang,v in values.items()}}
    args.output.write_text(json.dumps(results, indent=2) + "\n")
    for case, result in results["benchmarks"].items():
        py, rs = result["python"]["median"], result["rust"]["median"]
        print(f"{case}: Python {py:.3f}, Rust {rs:.3f} {result['unit']}; Python/Rust {py/rs:.2f}x")

if __name__ == "__main__":
    main()
