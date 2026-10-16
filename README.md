# neakasa-litterbox-rs

A native asynchronous Rust port of Rodrigo Roque's
[Neakasa Litterbox Python SDK](https://github.com/roquerodrigo/neakasa-litterbox-sdk),
with a standalone `neakasa-api` CLI. Based on the
[cat-weight update fork at `4be4fc7`](https://github.com/pvnick/neakasa-litterbox-sdk/tree/4be4fc72034752f227327c750c34921c53cf5a11)
and its companion local Python API wrapper. Includes authentication, device
status and controls, cat profiles, weight updates, visit history, and MQTT streaming.

## Why Rust? Because the cat cannot wait.

Some software can tolerate delay. A cat has already filed three incident reports
by the time you finish saying "interpreter startup." This is urgent,
time-sensitive infrastructure: a distributed system whose primary stakeholder
is standing in a box of sand.

Naturally, it deserves a compiled executable, explicit error handling, and memory
safety. When the next litter-box event arrives, we intend to be ready. The cat
has declined to participate in sprint planning.

The practical benefits are a standalone executable with no Python runtime and
an asynchronous Rust library for integration into Rust applications. There are
no comparative performance benchmarks or hard real-time guarantees here; cloud
and network latency still apply. The original Python SDK did the protocol work
that makes this port possible.

The library and executable use Rust HTTP, cryptography, JSON, and MQTT
implementations. Python is not needed to build or run them. The optional
`--import-session` command requires `adb` and the already-running Android
environment; ordinary commands use the existing host-side state.

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
