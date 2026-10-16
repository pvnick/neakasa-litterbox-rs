use serde_json::{Value, json};
use std::process::Command;

#[test]
fn help_and_missing_state_work_without_python_or_network() {
    let help = Command::new(env!("CARGO_BIN_EXE_neakasa-api"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(help.status.success());
    let text = String::from_utf8(help.stdout).unwrap();
    for name in [
        "cats",
        "status",
        "history",
        "properties",
        "set-cat-weight",
        "watch",
    ] {
        assert!(text.contains(name));
    }
    let dir = tempfile::tempdir().unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_neakasa-api"))
        .arg("--state-dir")
        .arg(dir.path())
        .arg("cats")
        .output()
        .unwrap();
    assert!(!out.status.success());
    let error: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(error["error"], "StateError");
    assert!(out.stderr.is_empty());
}

#[test]
fn state_endpoint_guard_prevents_sending_saved_credentials_to_other_hosts() {
    let dir = tempfile::tempdir().unwrap();
    let fixture: Value = serde_json::from_str(include_str!("fixtures/android.json")).unwrap();
    let login = neakasa_litterbox_sdk::LoginResult::from_api(&fixture["user"]).unwrap();
    std::fs::write(
        dir.path().join("session.json"),
        serde_json::to_vec(&json!({"session":login,"base_url":"https://example.invalid/api"}))
            .unwrap(),
    )
    .unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_neakasa-api"))
        .arg("--state-dir")
        .arg(dir.path())
        .arg("cats")
        .output()
        .unwrap();
    assert!(!out.status.success());
    let error: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(error["error"], "ValidationError");
    assert!(
        !String::from_utf8(out.stdout)
            .unwrap()
            .contains("test-token")
    );
}
