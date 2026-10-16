//! Offline benchmarks. All inputs come from the public synthetic test fixtures.
use neakasa_litterbox_sdk::{DeviceStatus, StatusUpdate, auth};
use serde_json::{Value, json};
use std::{hint::black_box, time::Instant};

fn measure<T>(iterations: usize, mut operation: impl FnMut() -> T) -> u128 {
    for _ in 0..1000 {
        black_box(operation());
    }
    let start = Instant::now();
    for _ in 0..iterations {
        black_box(operation());
    }
    start.elapsed().as_nanos()
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let case = &args[1];
    let iterations: usize = args[2].parse().unwrap();
    assert!(iterations > 0);
    let models: Value =
        serde_json::from_str(include_str!("../tests/fixtures/models.json")).unwrap();
    let crypto: Value =
        serde_json::from_str(include_str!("../tests/fixtures/crypto.json")).unwrap();
    let (elapsed_ns, validation, input_bytes) = match case.as_str() {
        "session_token" => {
            let token = "cafef00d".repeat(4);
            let operation = || {
                auth::session_token(
                    black_box(&token),
                    black_box("0123456789abcdef"),
                    black_box("abcdef0123456789"),
                    black_box(1700000000500),
                )
                .unwrap()
            };
            let validation = json!(operation());
            assert_eq!(validation, crypto["session_token"]);
            (measure(iterations, operation), validation, 46)
        }
        "iot_signing" => {
            let body = crypto["iot_body"].as_str().unwrap().as_bytes();
            let operation = || {
                auth::iot_headers(
                    black_box("/test?x-ca-request-id=fixed"),
                    black_box(body),
                    black_box(1779157013115),
                    black_box("fixed"),
                    black_box("Tue, 19 May 2026 02:16:53 GMT"),
                )
            };
            let validation = json!(operation());
            assert_eq!(validation, crypto["iot_headers"]);
            (measure(iterations, operation), validation, body.len())
        }
        "status_json" => {
            let body = serde_json::to_vec(&models["properties"]).unwrap();
            let operation = || {
                let parsed: Value = serde_json::from_slice(black_box(&body)).unwrap();
                DeviceStatus::from_response(&parsed)
            };
            let validation = json!(operation());
            assert_eq!(validation, models["status"]);
            (measure(iterations, operation), validation, body.len())
        }
        "push_json" => {
            let body = serde_json::to_vec(&models["push"]).unwrap();
            let operation = || {
                let parsed: Value = serde_json::from_slice(black_box(&body)).unwrap();
                StatusUpdate::from_push(&parsed).unwrap()
            };
            let mut validation = json!(operation());
            validation.as_object_mut().unwrap().remove("received_at");
            assert_eq!(validation, models["update"]);
            (measure(iterations, operation), validation, body.len())
        }
        _ => panic!("unknown benchmark"),
    };
    println!(
        "{}",
        json!({"case":case,"iterations":iterations,"elapsed_ns":elapsed_ns,
        "validation":validation,"input_bytes":input_bytes})
    );
}
