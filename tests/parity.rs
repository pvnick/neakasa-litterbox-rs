use neakasa_litterbox_sdk::{auth, state, stream, *};
use serde_json::{Value, json};

fn crypto() -> Value {
    serde_json::from_str(include_str!("fixtures/crypto.json")).unwrap()
}
fn models() -> Value {
    serde_json::from_str(include_str!("fixtures/models.json")).unwrap()
}

#[test]
fn aes_matches_python_byte_for_byte_including_empty_aligned_and_unicode() {
    for case in crypto()["boot"].as_array().unwrap() {
        let plain = case["plain"].as_str().unwrap();
        let cipher = case["cipher"].as_str().unwrap();
        assert_eq!(auth::boot_encrypt(plain).unwrap(), cipher);
        assert_eq!(auth::decrypt_login_token(cipher).unwrap(), plain);
    }
}
#[test]
fn password_and_session_token_match_python() {
    assert_eq!(
        auth::md5_double_hex("synthetic-password"),
        crypto()["double_md5"]
    );
    assert_eq!(
        auth::session_token(
            &"cafef00d".repeat(4),
            "0123456789abcdef",
            "abcdef0123456789",
            1700000000500
        )
        .unwrap(),
        crypto()["session_token"]
    );
    assert!(auth::aes_encrypt("hello", b"short", b"short").is_err());
    assert!(auth::decrypt_login_token("invalid base64").is_err());
}
#[test]
fn iot_signatures_match_python() {
    let f = crypto();
    let headers = auth::iot_headers(
        "/test?x-ca-request-id=fixed",
        f["iot_body"].as_str().unwrap().as_bytes(),
        1779157013115,
        "fixed",
        "Tue, 19 May 2026 02:16:53 GMT",
    );
    assert_eq!(json!(headers), f["iot_headers"]);
}
#[test]
fn openaccount_signatures_and_wire_encoding_match_python() {
    let (canonical, wire) = auth::oa_body(&json!({"request":{"k":"a&b"}})).unwrap();
    assert_eq!(canonical, "request={\"k\":\"a&b\"}");
    assert_eq!(wire, "request=%7B%22k%22%3A%22a%26b%22%7D");
    assert_eq!(
        json!(auth::oa_headers(
            "/api/prd/connect.json",
            &canonical,
            1779157013,
            "fixed",
            "Tue, 19 May 2026 02:16:53 GMT"
        )),
        crypto()["oa_headers"]
    );
}
#[test]
fn mqtt_bootstrap_matches_python_and_regions_are_preserved() {
    assert_eq!(
        auth::mqtt_auth_info(
            1779157013115,
            "cdf9bxg7",
            "m7yr2bp9zoxn9aj5e6yvbgsl7v3rxqdk"
        ),
        crypto()["mqtt_auth"]
    );
    let (host, cid, user, password) = stream::mqtt_credentials(
        "pk",
        "device",
        "secret",
        "eu-central-1.api-iot.aliyuncs.com",
    );
    assert_eq!(host, "pk.iot-as-mqtt.eu-central-1.aliyuncs.com");
    assert_eq!(user, "device&pk");
    assert_eq!(cid, "device&pk|securemode=2,signmethod=hmacsha1,ext=1|");
    assert_eq!(
        password,
        auth::hex(&auth::hmac_sha1(
            b"secret",
            b"clientIddevice&pkdeviceNamedeviceproductKeypk"
        ))
        .to_uppercase()
    );
}
#[test]
fn explicit_iot_envelope_has_no_fastjson_refs() {
    let v = auth::iot_envelope(
        json!({"iotId":"device"}),
        "1.0.4",
        Some("token"),
        "en-US",
        "request",
    );
    assert_eq!(
        v,
        json!({"id":"request","version":"1.0","request":{"apiVer":"1.0.4","language":"en-US","iotToken":"token"},"params":{"iotId":"device"}})
    );
}
#[test]
fn models_match_python_reference() {
    let f = models();
    assert_eq!(
        json!(DeviceStatus::from_response(&f["properties"])),
        f["status"]
    );
    assert_eq!(json!(Cat::from_api(&f["cat_raw"])), f["cat"]);
    assert_eq!(json!(Device::from_api(&f["device_raw"])), f["device"]);
    assert_eq!(json!(ToiletRecord::from_api(&f["record_raw"])), f["record"]);
    assert_eq!(
        json!(DailyStatistics::from_api(&f["statistics_raw"])),
        f["statistics"]
    );
    assert_eq!(
        ToiletRecord::from_api(&f["record_raw"]).duration_seconds(),
        47
    );
    let mut update = json!(StatusUpdate::from_push(&f["push"]).unwrap());
    update.as_object_mut().unwrap().remove("received_at");
    assert_eq!(update, f["update"]);
}
#[test]
fn unknown_enum_codes_and_partial_pushes_are_handled() {
    assert_eq!(OperatingState::from(999), OperatingState::Unknown);
    assert_eq!(RecordType::from(999), RecordType::Other);
    assert_eq!(CatGender::from(999), CatGender::Unknown);
    assert!(StatusUpdate::from_push(&json!({"params":{"deviceName":"x"}})).is_none());
    let update = StatusUpdate::from_push(
        &json!({"params":{"deviceName":"x","items":{"newThing":{"value":[1,2]},"invalid":3}}}),
    )
    .unwrap();
    assert_eq!(json!(update.changes), json!({"newThing":[1,2]}));
    assert_eq!(
        DeviceStatus::from_response(&json!({"Sand":{"value":{"percent":"42"}}})).sand_percent,
        42
    );
}
#[test]
fn android_import_and_python_session_roundtrip() {
    let f: Value = serde_json::from_str(include_str!("fixtures/android.json")).unwrap();
    let saved = state::decode_android_session(
        f["user_xml"].as_str().unwrap(),
        f["secure_xml"].as_str().unwrap(),
    )
    .unwrap();
    assert_eq!(saved.session.user_id, "456");
    assert_eq!(saved.session.user_token, "test-token");
    assert_eq!(saved.session.iot_token, "test-iot");
    assert_eq!(saved.base_url, Region::Us.web_url());
    let mut serialized = json!(saved.session);
    serialized.as_object_mut().unwrap().remove("iot_host");
    let restored: LoginResult = serde_json::from_value(serialized).unwrap();
    assert_eq!(restored.iot_host, US_GATEWAY);
    assert!(!format!("{restored:?}").contains("test-token"));
}
#[test]
fn login_token_accepts_server_metadata_after_authentication_fields() {
    let raw = json!({"loginToken":auth::boot_encrypt("token@123@0123456789abcdef@abcdef0123456789@metadata").unwrap(),"userInfo":{"aliUserId":456}});
    let login = LoginResult::from_api(&raw).unwrap();
    assert_eq!(login.user_id, "456");
    assert_eq!(login.user_token, "token");
    assert_eq!(login.aes_iv, "abcdef0123456789");
}
#[test]
fn saved_sessions_are_atomic_and_private() {
    let f: Value = serde_json::from_str(include_str!("fixtures/android.json")).unwrap();
    let login = LoginResult::from_api(&f["user"]).unwrap();
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("state");
    state::save_session(&path, &login, Region::Us.web_url()).unwrap();
    let saved: state::SavedSession = state::read_json(&path.join("session.json")).unwrap();
    assert_eq!(json!(saved.session), json!(login));
    state::save_session(&path, &login, Region::Us.web_url()).unwrap();
    assert_eq!(std::fs::read_dir(&path).unwrap().count(), 1);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o700
        );
        assert_eq!(
            std::fs::metadata(path.join("session.json"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
}
#[test]
fn reconnect_backoff_is_bounded_and_bad_policies_rejected() {
    let p = ReconnectPolicy::default();
    p.validate().unwrap();
    assert_eq!(p.delay_for(1).as_secs(), 1);
    assert_eq!(p.delay_for(3).as_secs(), 4);
    assert_eq!(p.delay_for(100000).as_secs(), 60);
    let bad = ReconnectPolicy {
        max_attempts: 0,
        ..p.clone()
    };
    assert!(bad.validate().is_err());
    let bad = ReconnectPolicy {
        multiplier: f64::NAN,
        ..p
    };
    assert!(bad.validate().is_err());
}
