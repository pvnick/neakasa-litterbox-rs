# Python → Rust test alignment

The reference is Rodrigo Roque's Python SDK with the cat-weight patch at
[`4be4fc72034752f227327c750c34921c53cf5a11`](https://github.com/pvnick/neakasa-litterbox-sdk/tree/4be4fc72034752f227327c750c34921c53cf5a11).
All **240 collected pytest cases**, including every parametrized instance, have
one unique Rust test entry in [alignment.json](alignment.json). The Rust suite
also retains its **28 additional integration/parity tests**.

**The inventory is exact; language semantics and test scaffolding are not
identical for all 240 cases.** A matching count alone is not evidence of equal
coverage or equal timed work. The manifest records the differences case by case.
In particular, the missing-Python-package-metadata fallback has no Rust runtime
equivalent; its counterpart verifies Cargo's embedded version. That is an
explicit adaptation, not a claim to test the Python fallback in Rust.

## What is checked

- [reference/](reference/) contains unchanged upstream test files and their
  original Python packaging configuration, under the repository's MIT license.
  The manifest pins their SHA-256 hashes and the installed reference SDK's source
  and bundled certificate hashes. Upstream example account values are upstream
  fixtures, not the repository owner's credentials.
- [aligned/mod.rs](aligned/mod.rs) names each Rust counterpart explicitly.
  Parameterized cases have deterministic suffixes and separate test entries.
  Dispatch tables in [aligned/](aligned/) contain the assertions; an unknown
  dispatch branch panics. No generated empty tests or ignored placeholders.
- `benchmarks/suite.py --audit-only --python PATH` collects both suites and
  rejects missing, duplicate, extra, pending, or unapproved benchmark cases.
  It verifies the reference source hashes before running either benchmark.
- All tests are offline. The 240 mapped Rust cases use injected HTTP/MQTT I/O.
  The additional integration tests use local HTTP and TLS MQTT servers.

The audit checks inventory and provenance. It does **not** mechanically prove
that arbitrary Rust assertions have the same meaning as Python assertions;
that requires reviewing the paired sources and the manifest notes.

## Deliberate adaptations

| Area | Difference and treatment |
| --- | --- |
| Models, cryptography, signing, status updates | Same fixture values and expected observable results. Eligible cases form the strict paired timing set. |
| HTTP/client/handshake unit mocks | Python often replaces a whole SDK method; many Rust counterparts run the real signing/request stack and mock HTTP I/O. These exercise additional work and are excluded from strict timing. |
| Full EU routing regression | Both run the real five-request handshake/device flow with the same responses and mock only HTTP I/O. Included in strict timing. |
| Error categories | Rust enum variants replace Python exception classes/inheritance. Sanitized Rust errors deliberately omit server bodies and some context strings. Such cases are excluded from strict timing. |
| Boolean weights/IDs and unknown units | Rust typed arguments cannot express those inputs. Counterparts verify rejection at the serde boundary; Python rejects them at runtime. Excluded. |
| HTTP ownership/context managers | Rust shared ownership and `Drop` replace Python `close()` and `async with`. Weak references verify resource ownership release. Excluded. |
| Session identity | Rust owned values preserve session contents; Python additionally checks object identity. Excluded. |
| User-Agent/region defaults | The port retains the working wrapper's Python 0.2.2 User-Agent and `usapi.neakasapet.com` US endpoint. This intentionally differs from Rust package metadata and the upstream US endpoint. Excluded where affected. |
| Package metadata fallback | Rust embeds version metadata at compile time. The Python missing-metadata runtime path has no direct counterpart. The replacement checks embedded metadata availability and is excluded. |
| Four packaging checks | Rust reads the same pinned Python `pyproject.toml` and checks the same Python version/classifier/ruff/mypy constraints. These verify reference provenance, not Rust compiler support. Excluded because setup boundaries differ. |
| MQTT lifecycle and callbacks | Rust futures, channel cancellation, `Result` callbacks, ownership, and rustls replace Python futures/exceptions, context managers, and SSL contexts. Real Rust code is exercised with fake I/O; setup differs and these cases are excluded from strict timing. |
| Logging and random IDs | Rust global logging versus pytest capture, and UUID hex versus Python random alphanumeric IDs. Behavioral checks remain, but excluded from strict timing. |

## Bugs exposed and fixed while porting

The expanded suite now checks empty/short login tokens, tolerant model list
parsing, unknown-state warn-once behavior, JSON coercion, negative sand
calibration, handshake error categories, regional MQTT credentials, HTTP
ownership, bind reply correlation/timeouts, callback failure isolation, topic
filtering, cancellation, restart, and reconnect exhaustion.

The stream now supports lazy construction with `client.status_stream(options)`,
explicit idempotent `start()`/`stop()`, and callbacks via `stream.handlers()`.
`watch_status(options).await` remains the connect-and-start convenience method.
The MQTT transport's injectable boundary is shared by production and test code;
the tests do not exercise a separate implementation of the stream state machine.

## Timing policy

The manifest approves **50 strict paired cases** based on comparable fixture
values, observable assertions, and mock boundaries—not on which language wins.
Both sides run exactly those case IDs. The remaining cases still run in the
full-suite validation and timing, but the full-suite timing is labelled context,
not an apples-to-apples SDK speed comparison.

Two clocks are reported for each set:

1. **Whole test process:** process creation through successful exit, including
   imports, collection, test framework, fixtures, assertions, teardown, and the
   small timing-log writes. Compilation is excluded.
2. **Test calls:** sum of per-case elapsed call time. Python measures the
   `pytest_runtest_call` hook; Rust measures each function/future body. Imports,
   collection, external fixture setup/teardown, native harness thread creation,
   Tokio runtime construction, and timing-log writes are excluded. Assertions,
   in-function fixtures/mocks, scheduling, and framework call dispatch remain.
   The frameworks are different; this is not an isolated SDK-operation timer.

Both run sequentially on the same CPU, alternating Python/Rust order, with
three warmups and 21 recorded samples. Coverage instrumentation and third-party
pytest plugin autoload are disabled. Every timed run must pass with the exact
expected count, with no skipped or ignored cases. Raw per-case timings are saved
so the aggregate can be inspected. Full results and reproduction instructions
are in [the benchmark documentation](../benchmarks/README.md).
