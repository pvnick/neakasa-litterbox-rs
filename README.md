# neakasa-litterbox-rs

A native asynchronous Rust port of Rodrigo Roque's
[Neakasa Litterbox Python SDK](https://github.com/roquerodrigo/neakasa-litterbox-sdk),
with a standalone `neakasa-api` CLI. Based on the
[cat-weight update fork at `4be4fc7`](https://github.com/pvnick/neakasa-litterbox-sdk/tree/4be4fc72034752f227327c750c34921c53cf5a11)
and its companion local Python API wrapper. Includes authentication, device
status and controls, cat profiles, weight updates, visit history, and MQTT streaming.

> **202.41 critical milliseconds reclaimed per CLI startup.**
> Our M1 benchmark measured median `--help` startup at **203.586 ms in Python**
> versus **1.176 ms in Rust**. That is approximately **0.202 seconds** returned
> to the demanding business of standing beside a litter box.
> These are measured local startup savings; the cloud remains on its own schedule.

## Why Rust? Because the cat cannot wait.

Some software can tolerate delay. A cat has already filed three incident reports
by the time you finish saying "interpreter startup." This is urgent,
time-sensitive infrastructure: a distributed system whose primary stakeholder
is standing in a box of sand.

Naturally, it deserves a compiled executable, explicit error handling, and memory
safety. When the next litter-box event arrives, we intend to be ready. The cat
has declined to participate in sprint planning.

Every one of those **202.41 milliseconds** matters to a stakeholder who considers
a closed bathroom door a production outage. We have eliminated approximately
**99.42% of the measured startup wait**. The remaining 1.176 ms is presumably
already the subject of an escalated support ticket.

The practical benefits are a standalone executable with no Python runtime and
an asynchronous Rust library for integration into Rust applications. The
[benchmarks below](#benchmarks) measure local overhead; they do not provide hard
real-time guarantees, and cloud/network latency still apply. The original Python
SDK did the protocol work that makes this port possible.

The library and executable use Rust HTTP, cryptography, JSON, and MQTT
implementations. Python is not needed to build or run them. The optional
`--import-session` command requires `adb` and the already-running Android
environment; ordinary commands use the existing host-side state.

## A truly smart home needs a sufficiently overengineered toaster

A home cannot be truly smart until its breakfast appliances can inquire about
the litter box. This port brings that architectural ambition one executable
closer to reality. Whether the toaster should possess this knowledge remains
outside the scope of the SDK.

The practical benefit is simpler deployment: build for a compatible processor
and operating system, then run the native executable without installing Python
or its package environment. Small Linux computers, home servers, and compatible
Linux-based appliance controllers are potential deployment targets. The async
library can also be embedded in a Rust home-automation service, while the CLI's
JSON output can feed scripts, dashboards, and scheduled status checks.

A sufficiently capable Linux-powered toaster could therefore host a custom
integration. The toaster's original microcontroller probably cannot: this SDK
uses the Rust standard library and expects OS threads, sockets, TLS support,
internet access, and enough memory for its runtime and connections. It is not a
bare-metal firmware library. Each target needs an appropriate build and testing;
the verified platform here is Linux `aarch64`, not a kitchen-appliance test lab.

The measured **3.58 MiB** peak belongs to `--help`, not an active cloud session.
Rust does not automatically make unsupported hardware compatible, and this port
does not include a ready-made toaster or home-automation integration. What it
provides is a native SDK and CLI with which to build one. Device access still
uses the Neakasa cloud; deploying locally does not make it an offline LAN API.

At last, a foundation for a home where every appliance can be kept informed of
the cat's business. The cat has requested administrator privileges.

## Benchmarks

Measured on **October 16, 2026**, on an **Apple M1 performance core**, Linux
`aarch64`, with both implementations pinned to CPU 4. Rust **1.99.1**, default
release profile (no custom `RUSTFLAGS`); CPython **3.14.8**, Python SDK **0.2.2**
at fork commit `4be4fc7`, and `cryptography` **50.0.2**. The CPU used its existing
`schedutil` governor; frequency was not fixed.

Values are **median [25th–75th percentile]**. The ratio is Python/Rust:
higher means lower Rust elapsed time or resident memory for this workload.

| Workload | Python | Rust | Python/Rust median |
| --- | ---: | ---: | ---: |
| CLI startup (`--help`) | 203.586 [198.811–207.122] ms | 1.176 [1.068–2.344] ms | 173.08× |
| CLI peak resident memory (`--help`) | 52.359 [51.914–52.375] MiB | 3.578 [3.562–3.594] MiB | 14.63× |
| Session-token encryption | 3.774 [3.742–3.804] µs/op | 1.743 [1.701–1.753] µs/op | 2.17× |
| IoT request signing | 2.743 [2.733–2.761] µs/op | 2.714 [2.312–2.810] µs/op | 1.01× |
| JSON → device status | 16.381 [16.334–16.446] µs/op | 10.089 [9.959–10.170] µs/op | 1.62× |
| JSON → push update | 14.291 [14.243–14.356] µs/op | 11.501 [11.406–11.538] µs/op | 1.24× |

The biggest measured advantage is CLI startup: roughly **204 ms → 1.18 ms**,
with peak resident memory around **52.4 MiB → 3.58 MiB**. Matching SDK operations
show smaller gains: about **2.17×** for token encryption, **1.62×** for status
parsing, and **1.24×** for push parsing. **Request signing is effectively tied**;
its timing distributions overlap. These results do not establish a universal
Rust/Python speed ratio.

Each SDK workload uses identical synthetic inputs and checks equivalent outputs
before timing. Results cover 15 interleaved batches per implementation, each
with 1,000 warmup operations. JSON workloads include decoding and model creation.
Startup uses 31 fresh processes per implementation after five warmups, with
warm OS file caches; memory uses seven separate processes. The Python startup
target is a snapshot of the original companion API wrapper, and Rust uses the
release CLI. Their normal imports/runtime setup are included; help text differs.
CPU affinity also constrains the Rust runtime worker pool.

**Scope:** offline local work only. No credentials, network requests, device
operations, or cloud latency are involved. Saving milliseconds in startup does
not establish faster cleaning, MQTT delivery, or live API responses. The cat’s
patience was not instrumented.

See [raw samples and environment metadata](benchmarks/results-m1.json) and
[benchmark code, methodology, and reproduction instructions](benchmarks/README.md).

## Build and run

```bash
git clone https://github.com/pvnick/neakasa-litterbox-rs.git
cd neakasa-litterbox-rs
cargo build --release --locked
./target/release/neakasa-api cats
./target/release/neakasa-api status
./target/release/neakasa-api history
```

The Rust SDK is in `src/client.rs`, with models in `src/models.rs`.
The API wrapper is `src/bin/neakasa-api.rs` and builds into `neakasa-api`.
The default command is `cats`, and `history` covers the preceding 24 hours.
`--help` lists all commands. `properties` returns raw device fields, which can
include owner identifiers. `devices` lists account devices; `statistics` accepts
`--start` and `--end` Unix seconds, and `--zone-seconds` (UTC offset).
`watch` streams JSON property updates until Ctrl-C.

Like the Python wrapper, the CLI loads `session.json`, `device.json`, and optional
`credentials.json` from `~/.local/state/neakasa-api`. Use `--state-dir PATH` for
another state directory. Session refreshes are written atomically with mode 0600
in a directory with mode 0700. Both implementations share the same session file;
avoid running fresh logins concurrently.

If the service reports an expired session:

```bash
./target/release/neakasa-api --login cats
```

This uses the saved credentials and performs the full REST/OpenAccount/IoT login.
It can replace the app's session. It does not start Android or re-pair the box.
Errors omit response bodies, request URLs, and tokens.

To update an explicitly selected cat, first obtain its current ID using
`NeakasaClient::list_cats`; then supply the requested weight and unit:

```text
neakasa-api set-cat-weight --cat-id <ID> --weight <WEIGHT> --unit <kg|lb> --zone-seconds <UTC_OFFSET_SECONDS>
```

The command reads all profiles, submits only the target weight/unit, and verifies
that the other fields and profiles remain unchanged. Provide the current local
UTC offset explicitly (New York changes with daylight saving time). It does not
assign old visits to a cat. After an uncertain write result, read back before
retrying. Profile updates are never automatically replayed.

## Library

Add this local dependency to a Rust application's `Cargo.toml`:

```toml
[dependencies]
neakasa-litterbox-sdk = { git = "https://github.com/pvnick/neakasa-litterbox-rs.git" }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

`examples/read_cats.rs` demonstrates resuming the existing session and saving any
refresh, without printing authentication material:

```bash
~/.cargo/bin/cargo run --example read_cats --locked
```

| Python capability | Rust API |
| --- | --- |
| Login / cached session | `login(None)` / `login(Some(session))`, `resume_session` |
| Session persistence | `state::SavedSession`, `state::read_json`, `state::save_session` |
| Devices and cat profiles | `list_devices`, `list_cats`, `register_device` |
| Visits and daily statistics | `get_toilet_records`, `get_toilet_statistics` |
| Cat profile weight | `set_cat_weight` with typed `WeightUnit` |
| Status / raw properties | `get_status`, `get_properties` |
| Clean and level controls | `start_clean`, `stop_clean`, `start_level`, `stop_level` |
| Settings | `set_auto_clean`, `set_auto_level`, `set_silent_mode`, `set_child_lock` |
| Sand calibration | `calibrate_sand` |
| Status callbacks | `watch_status`, then `StatusStream::next().await` |
| Stream shutdown | `stop().await` or drop the stream |

Settings and mechanical operations require explicit calls. `set_auto_clean`
preserves the existing cleaning mode, delay, and additional configuration fields.
Read current properties first and verify a fresh device report after a change.
Cloud acceptance does not establish that a physical operation completed.

Rust differences: errors are a typed `Error` enum; HTTP resources close when their
owners drop; status updates use a bounded async receiver instead of Python
callbacks. The stream verifies TLS certificates, waits for subscription and
account-binding acknowledgements, and reconnects with bounded backoff.
`StreamOptions` supports custom certificate configuration and broker endpoints.

The working wrapper's explicit Aliyun JSON envelopes and US app endpoint
(`https://usapi.neakasapet.com/api`) are incorporated directly. App protocol
constants and the Python 0.2.2 User-Agent are retained for compatibility.
The CLI enforces that US endpoint; the library supports regional/custom endpoints.

## Verification

```bash
~/.cargo/bin/cargo fmt --all -- --check
~/.cargo/bin/cargo clippy --locked --all-targets -- -D warnings
~/.cargo/bin/cargo test --locked
```

Tests use synthetic fixtures, local HTTP mocks, and a local TLS MQTT broker.
They do not read the real account or operate the litter box. Python-produced
fixtures verify cryptography, signing, data models, and Android session decoding.
HTTP tests exercise login, session refresh, profile writes, device settings,
history, and error handling. Stream tests exercise authenticated TLS connection,
binding, push delivery, reconnect, timeouts, and shutdown. CLI tests cover local
errors and the endpoint guard. Real device writes and mechanical operations are
not part of automated verification.

Local verification on October 16, 2026: all 28 tests passed, formatting and
Clippy checks passed, and the release executable built successfully. Live Rust
login, cat profiles, status, and 24-hour history succeeded. Profiles returned
the expected profile weights. No profile or device settings were changed.

Live `watch` returned API code **2043**. A comparison using the installed Python
SDK returned the same code during MQTT account binding. The Rust stream passes
the local TLS broker tests, but live push delivery remains unverified while this
shared cloud/account failure persists. REST status polling works.

## Credentials and test data

Account credentials and saved sessions belong outside this repository, in the
private state directory. This repository does not include an account, paired
device, or working session. The CLI currently expects that state to have been
provisioned already; it is not an interactive account-setup tool.

Test accounts, tokens, IDs, encryption inputs, and Android preferences in
`tests/fixtures` are synthetic. The bundled PEM file is a public root certificate,
not a private key. Shared app protocol constants in `src/constants.rs` and the
boot encryption constants in `src/auth.rs` originate in the upstream SDK; they
are not a user's password, session token, or per-account encryption keys.

Build outputs, environment files, credentials, and session files are ignored by
Git. Do not commit real API responses, device metadata, or diagnostic captures.

## Attribution

The original protocol implementation is
[roquerodrigo/neakasa-litterbox-sdk](https://github.com/roquerodrigo/neakasa-litterbox-sdk).
The cat profile weight method comes from
[upstream PR #66](https://github.com/roquerodrigo/neakasa-litterbox-sdk/pull/66).
This is an independent Rust port, not an official Neakasa product.
MIT licensed; the upstream copyright notice is retained in `LICENSE`.
