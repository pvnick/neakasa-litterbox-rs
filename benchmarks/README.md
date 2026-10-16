# Reproducing the comparison

These benchmarks compare the Rust port with the installed Python SDK at commit
`4be4fc72034752f227327c750c34921c53cf5a11`. They use the same synthetic inputs as
the parity tests. They do not access credentials, saved sessions, or the network.

On Linux with a Rust toolchain and Python 3.14:

```bash
python3.14 -m venv .venv-bench
.venv-bench/bin/python -m pip install -r benchmarks/requirements.txt
cargo build --release --locked --bin neakasa-api --example benchmark --example benchmark_rss
.venv-bench/bin/python benchmarks/run.py \
  --cpu 4 --machine 'Apple M1 (Firestorm performance core)' \
  --output benchmarks/results-local.json
```

Choose an allowed CPU number and change the machine description for your system.
CPU 4 is a performance core on the machine used for the checked-in results. The
runner and all child processes inherit a one-core affinity mask. This also limits
the Rust CLI's automatically sized Tokio worker pool. No governor, CPU frequency,
or other system settings are changed. Normal background system activity remains.

`run.py` saves individual measurements, medians, quartiles, environment versions,
source hashes, and the reference SDK commit. It omits hostnames, account details,
and local paths. Run this script with the interpreter containing the reference
SDK. `requirements.txt` pins the Python environment; `Cargo.lock` pins Rust crates.

## Workloads

| Workload | Work per iteration |
| --- | --- |
| `session_token` | Format the timestamped synthetic token, AES-CBC encrypt it, and base64 encode it using the SDK function. |
| `iot_signing` | Build the full IoT request header map, including content MD5 and HMAC-SHA1. Timestamp, nonce, date, path, and body are fixed and identical. |
| `status_json` | Decode identical JSON bytes and build `DeviceStatus`, including all modeled fields. |
| `push_json` | Decode identical JSON bytes and build `StatusUpdate`, including a timestamp and flattened property changes. |
| `cli_help` | Start a fresh process, import/initialize the application, generate `--help`, and exit; stdout goes to `/dev/null`. |
| `cli_peak_rss` | Peak resident memory for a separate invocation of each CLI's `--help`. |

The Python worker calls the SDK itself; it is not a simplified Python replica.
The Rust worker calls the port's public functions. Both validate against the
reference fixtures before timing. The runner compares normalized outputs across
implementations and every batch (excluding only the wall-clock push timestamp).
Validation, input setup, fixture reads, and imports are outside microbenchmark
timing. Parsing, allocations, object construction, and destruction are inside.
Neither timed parsing workload serializes its resulting model back to JSON.

Each microbenchmark runs 15 fresh-process batches per implementation, with 1,000
warmup operations in each batch. Batch sizes are 200,000 token operations, 50,000
signing operations, and 20,000 operations for each JSON workload. The runner
randomizes Python/Rust order within each pair with a recorded seed and never runs
the workers concurrently. It reports elapsed wall time divided by operations;
loop/function-call overhead is included. Rust uses `black_box` on inputs and
outputs to prevent elimination of work. Python garbage collection stays enabled.

Startup uses five warmup processes per implementation and 31 measured processes
per implementation, with interleaved order. OS file caches stay warm: this is
fresh-process latency, not a cold-boot or cold-disk benchmark. Memory uses seven
separate samples per implementation via Linux `wait4().ru_maxrss`, reported in
MiB; this includes shared resident pages and is not exclusive heap allocation.
A small native launcher (`rss.rs`) collects the CLI's usage, avoiding the Python
runner's pre-exec resident-memory floor in the child. Memory collection is outside
the timed startup samples.

`python_api.py` is a snapshot of the original companion Python wrapper, with only
its machine-specific shebang replaced by `#!/usr/bin/env python3`. The runner
only invokes `--help`, which exits before any account or state access. Both CLIs
have their normal imports/runtime setup; their help text and command sets differ.
This is a comparison of these applications' startup, not an isolated language
runtime comparison. Use the SDK microbenchmarks to compare matching operations.

The results are measurements of these versions on one machine, not general Rust
versus Python claims. They do not measure live HTTP, TLS establishment, MQTT
delivery, mechanical response, or time until a cat gets a clean litter box. The
cloud can easily dominate any local savings. No device settings are changed.

## Test-suite benchmark

`results-m1.json` above records the earlier startup and operation benchmarks.
The newer `suite-results-m1.json` measures the **actual tests**, with two clocks:
whole process and the sum of test-call durations. See
[the full coverage map and its limitations](../tests/ALIGNMENT.md).

There are 240 Python-to-Rust case mappings. The full 240-case timing includes
explicit language and mock-boundary adaptations, so it is context rather than
an apples-to-apples SDK ratio. The strict paired set uses the same 50 approved
case IDs on both sides. The 28 extra Rust integration tests are validated
separately and excluded from both cross-language timings.

Reproduce from the repository root with Rust and a Python 3.14 environment:

```bash
python3.14 -m venv .venv-suite
.venv-suite/bin/python -m pip install -r benchmarks/suite-requirements.txt
python benchmarks/suite.py --python "$PWD/.venv-suite/bin/python" --cpu 4
```

To check provenance and the one-to-one inventory without timing:

```bash
python benchmarks/suite.py --python "$PWD/.venv-suite/bin/python" --audit-only
```

The script first builds the release test binary, then audits hashes and collected
IDs, warms both implementations, and alternates 21 measured runs per language.
Compilation is outside the timer; the recorded prebuild duration is incremental,
**not a cold compilation benchmark**. CPU 4 was an M1 performance core on the
measurement machine; choose an available equivalent core on another machine.
The existing `schedutil` governor is recorded, not changed.

`test_calls` excludes imports/collection and timing-file writes. It includes
assertions, test doubles, and async scheduling inside the test calls. Rust native
harness threads and Tokio runtime creation are outside this clock; Python
external fixtures are outside it, while pytest's call wrapper remains. Thus it
reduces the startup distortion without pretending the two test runners are
identical. Whole-process results include all of these costs.

No account credentials or cloud access are needed. Some sandbox profiles block
asyncio's local wakeup sockets even with mocked network I/O; run the offline
benchmark in an environment that permits local sockets. Every sample is rejected
if the case count, pass status, or per-case timing inventory differs.
