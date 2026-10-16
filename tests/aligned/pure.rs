use crate::{constants::*, error::*, json as safe_json, *};
use base64::{Engine, engine::general_purpose::STANDARD as B64};
use serde_json::{Value, json};
use std::sync::atomic::{AtomicUsize, Ordering};

struct Capture;
static WARNING_COUNT: AtomicUsize = AtomicUsize::new(0);
impl log::Log for Capture {
    fn enabled(&self, _: &log::Metadata<'_>) -> bool {
        true
    }
    fn log(&self, record: &log::Record<'_>) {
        if record.args().to_string() == "unknown operating state: 42" {
            WARNING_COUNT.fetch_add(1, Ordering::SeqCst);
        }
    }
    fn flush(&self) {}
}
fn login() -> LoginResult {
    super::support::session("")
}
fn push(items: Value) -> StatusUpdate {
    StatusUpdate::from_push(&json!({"params":{"deviceName":"PB01","items":items}})).unwrap()
}
pub(super) fn run(group: &str, name: &str) {
    let base = name.split('[').next().unwrap();
    match (group, base) {
        ("models", "test_cat_parses_live_sample") => {
            let raw = json!({"id":113196,"name":"Mini","weight":3,"unit":"kg","avatar":"https://jhc-en.oss-ap-southeast-1.aliyuncs.com/user/.../mini.png","birthday":"","variety":-1,"gender":2,"sterilization":1,"enabled":1,"path":"user/400068852/catbox/PB01009G25110010081/1750883181.png"});
            let c = Cat::from_api(&raw);
            assert_eq!(c.id, 113196);
            assert_eq!(c.name, "Mini");
            assert_eq!(c.weight, 3.0);
            assert_eq!(c.gender, CatGender::Female);
            assert!(c.is_sterilized());
            assert!(c.is_enabled());
        }
        ("models", "test_cat_unknown_gender_falls_back") => assert_eq!(
            Cat::from_api(&json!({"id":1,"name":"?","gender":99})).gender,
            CatGender::Unknown
        ),
        ("models", "test_cat_list_from_response_filters_non_objects") => {
            let c = Cat::list_from_response(
                &json!([{"id":1,"name":"A","weight":1.0,"gender":1},"not an object",{"id":2,"name":"B","weight":2.0,"gender":2}]),
            );
            assert_eq!(c.iter().map(|c| c.id).collect::<Vec<_>>(), vec![1, 2]);
        }
        ("models", "test_cat_list_from_response_handles_unexpected_top_level") => {
            for v in [json!({"unexpected":"object"}), Value::Null, json!("string")] {
                assert!(Cat::list_from_response(&v).is_empty());
            }
        }
        ("models", "test_toilet_record_visit_sample") => {
            let r = ToiletRecord::from_api(
                &json!({"type":1,"record_id":182739595,"cat_id":113196,"start_time":1779189528i64,"end_time":1779189563i64,"weight":2.9,"unit":"kg","way":0}),
            );
            assert_eq!(r.record_type, RecordType::CatVisit);
            assert_eq!(r.cat_id, 113196);
            assert_eq!(r.weight, 2.9);
            assert_eq!(r.duration_seconds(), 35);
        }
        ("models", "test_toilet_record_clean_cycle_sample") => {
            let r = ToiletRecord::from_api(
                &json!({"type":2,"record_id":141548746,"cat_id":0,"start_time":1779189865i64,"end_time":1779190006i64,"weight":20,"unit":"","way":1}),
            );
            assert_eq!(r.record_type, RecordType::CleanCycle);
            assert_eq!(r.cat_id, 0);
            assert_eq!(r.unit, "");
        }
        ("models", "test_toilet_record_list_from_response_unwraps_envelope") => {
            let r = ToiletRecord::list_from_response(
                &json!({"record_list":[{"type":1,"record_id":1,"cat_id":0,"start_time":0,"end_time":1},{"type":2,"record_id":2,"cat_id":0,"start_time":2,"end_time":3}]}),
            );
            assert_eq!(
                r.iter().map(|r| r.record_id).collect::<Vec<_>>(),
                vec![1, 2]
            );
            assert_eq!(r[0].record_type, RecordType::CatVisit);
            assert_eq!(r[1].record_type, RecordType::CleanCycle);
        }
        ("models", "test_toilet_record_list_from_response_tolerates_missing_envelope") => {
            for v in [json!({}), Value::Null, json!([{"type":1}])] {
                assert!(ToiletRecord::list_from_response(&v).is_empty());
            }
        }
        ("models", "test_device_status_bucket_full_reads_room_of_bin") => {
            assert!(
                DeviceStatus::from_response(
                    &json!({"room_of_bin":{"time":1,"value":1},"bucketStatus":{"time":1,"value":0}})
                )
                .bucket_full
            );
            assert!(
                !DeviceStatus::from_response(
                    &json!({"room_of_bin":{"time":1,"value":0},"bucketStatus":{"time":1,"value":1}})
                )
                .bucket_full
            );
        }
        ("models", "test_device_status_operating_state_reads_bucket_status") => {
            for (v, e) in [
                (
                    json!({"bucketStatus":{"time":1,"value":2}}),
                    OperatingState::Cleaning,
                ),
                (
                    json!({"bucketStatus":{"time":1,"value":0}}),
                    OperatingState::Idle,
                ),
                (json!({}), OperatingState::Idle),
                (
                    json!({"bucketStatus":{"time":1,"value":99}}),
                    OperatingState::Unknown,
                ),
            ] {
                assert_eq!(DeviceStatus::from_response(&v).operating_state, e);
            }
        }
        ("models", "test_operating_state_from_code") => {
            for (v, e) in [
                (3, OperatingState::Leveling),
                (1, OperatingState::Restoring),
                (5, OperatingState::CatAppears),
                (7, OperatingState::Unknown),
            ] {
                assert_eq!(OperatingState::from(v), e);
            }
        }
        ("models", "test_operating_state_unmapped_code_warns_once") => {
            let _ = log::set_logger(&Capture);
            log::set_max_level(log::LevelFilter::Warn);
            crate::models::WARNED_CODES.lock().unwrap().remove(&42);
            WARNING_COUNT.store(0, Ordering::SeqCst);
            assert_eq!(OperatingState::from(42), OperatingState::Unknown);
            assert_eq!(OperatingState::from(42), OperatingState::Unknown);
            assert_eq!(WARNING_COUNT.load(Ordering::SeqCst), 1);
        }
        ("models", "test_daily_statistics_parses_live_sample") => {
            let s = DailyStatistics::from_api(
                &json!({"date":"2026-04-23","num":11,"weight":4.8,"unit":"kg","toilet_total_second":760,"weight_avg":3.67}),
            );
            assert_eq!(s.date, "2026-04-23");
            assert_eq!(s.num, 11);
            assert_eq!(s.toilet_total_second, 760);
            assert_eq!(s.weight_avg, 3.67);
        }
        ("models", "test_daily_statistics_list_from_response") => {
            let s = DailyStatistics::list_from_response(
                &json!([{"date":"2026-04-23","num":11,"toilet_total_second":760},{"date":"2026-04-24","num":9,"toilet_total_second":694}]),
            );
            assert_eq!(
                s.iter().map(|s| s.date.as_str()).collect::<Vec<_>>(),
                vec!["2026-04-23", "2026-04-24"]
            );
            assert!(DailyStatistics::list_from_response(&Value::Null).is_empty());
        }
        ("models_extra", "test_user_info_round_trips_through_dict") => {
            let u = login().user_info;
            let v: UserInfo = serde_json::from_value(json!(u)).unwrap();
            assert_eq!(u, v);
        }
        (
            "models_extra",
            "test_login_result_round_trips_through_dict"
            | "test_login_result_round_trips_non_default_iot_host",
        ) => {
            let mut s = login();
            s.aes_key = "key".into();
            s.aes_iv = "iv".into();
            s.user_info = UserInfo {
                user_id: 1,
                user_name: "u".into(),
                ali_user_id: 2,
                ali_authentication_token: "tok".into(),
            };
            s.iot_token = "iot".into();
            if base.contains("non_default") {
                s.iot_host = "eu-central-1.api-iot.aliyuncs.com".into();
            }
            let r: LoginResult = serde_json::from_value(json!(s)).unwrap();
            assert_eq!(r, s);
            if base.contains("non_default") {
                assert_eq!(r.iot_host, "eu-central-1.api-iot.aliyuncs.com");
            }
        }
        ("models_extra", "test_login_result_from_dict_defaults_iot_host_for_legacy_cache") => {
            let s:LoginResult=serde_json::from_value(json!({"user_id":"1","user_token":"t","aes_key":"k","aes_iv":"iv","user_info":{"user_id":1,"user_name":"u","ali_user_id":2,"ali_authentication_token":"tok"},"issued_at":1000.0,"iot_token":"iot"})).unwrap();
            assert_eq!(s.iot_host, US_GATEWAY);
        }
        ("models_extra", "test_login_result_age_seconds") => {
            let mut s = login();
            s.user_id = "1".into();
            s.user_token = "t".into();
            s.aes_key = "k".into();
            s.aes_iv = "iv".into();
            s.user_info = UserInfo {
                user_id: 1,
                user_name: "u".into(),
                ali_user_id: 2,
                ali_authentication_token: "tok".into(),
            };
            s.issued_at = 1000.0;
            assert_eq!(s.age_seconds_at(1250.0), 250.0);
        }
        ("models_extra", "test_parse_login_token_empty_returns_blanks") => assert_eq!(
            LoginResult::parse_login_token("").unwrap(),
            ["", "", "", ""]
        ),
        ("models_extra", "test_parse_login_token_decrypts_and_splits") => assert_eq!(
            LoginResult::parse_login_token(&auth::boot_encrypt("tok@uid@key@iv").unwrap()).unwrap(),
            ["tok", "uid", "key", "iv"]
        ),
        ("models_extra", "test_parse_login_token_pads_short_token") => assert_eq!(
            LoginResult::parse_login_token(&auth::boot_encrypt("tok@uid").unwrap()).unwrap(),
            ["tok", "uid", "", ""]
        ),
        (
            "models_extra",
            "test_login_result_from_json_prefers_ali_user_id"
            | "test_login_result_from_json_falls_back_to_embedded_user_id",
        ) => {
            let prefer = base.ends_with("prefers_ali_user_id");
            let s=LoginResult::from_api(&json!({"loginToken":auth::boot_encrypt("tok@embedded@key@iv").unwrap(),"userInfo":if prefer{json!({"aliUserId":999,"userName":"u"})}else{json!({})}})).unwrap();
            assert_eq!(s.user_id, if prefer { "999" } else { "embedded" });
            if prefer {
                assert_eq!(s.user_token, "tok");
            }
        }
        ("models_extra", "test_device_list_from_response_ignores_non_array") => {
            assert!(Device::list_from_response(&json!({"data":"not-a-list"})).is_empty());
            assert!(Device::list_from_response(&json!({})).is_empty());
        }
        ("models_extra", "test_device_list_from_response_skips_non_mapping_entries") => {
            let d = Device::list_from_response(&json!({"data":[{"deviceName":"PB01"},"junk",5]}));
            assert_eq!(
                d.iter().map(|d| d.device_name.as_str()).collect::<Vec<_>>(),
                vec!["PB01"]
            );
        }
        ("models_extra", "test_record_type_unknown_falls_back_to_other") => {
            assert_eq!(RecordType::from(99), RecordType::Other);
            assert_eq!(RecordType::from(1), RecordType::CatVisit);
        }
        ("models_extra", "test_device_status_property_int_coerces_numeric_string") => {
            assert!(DeviceStatus::from_response(&json!({"room_of_bin":{"value":"1"}})).bucket_full)
        }
        (
            "models_extra",
            "test_device_status_property_int_non_numeric_string_falls_back_to_zero",
        ) => assert!(
            !DeviceStatus::from_response(&json!({"room_of_bin":{"value":"nope"}})).bucket_full
        ),
        ("models_extra", "test_loads_rejects_non_object_top_level") => {
            assert!(safe_json::loads(b"[1, 2, 3]").is_err())
        }
        ("models_extra", "test_get_str_default_on_wrong_type") => {
            assert_eq!(safe_json::get_str(&json!({"k":5}), "k", ""), "");
            assert_eq!(safe_json::get_str(&json!({"k":"v"}), "k", ""), "v");
            assert_eq!(safe_json::get_str(&json!({}), "k", "d"), "d");
        }
        ("models_extra", "test_get_int_coerces_and_falls_back") => {
            for (v, d, e) in [
                (json!(true), 0, 1),
                (json!(7), 0, 7),
                (json!("9"), 0, 9),
                (json!("nope"), -1, -1),
                (json!(1.5), 3, 3),
            ] {
                assert_eq!(safe_json::get_int(&json!({"k":v}), "k", d), e);
            }
        }
        ("models_extra", "test_get_float_coerces_and_falls_back") => {
            for (v, d, e) in [
                (json!(true), 0.0, 1.0),
                (json!(2), 0.0, 2.0),
                (json!("3.5"), 0.0, 3.5),
                (json!("bad"), 9.0, 9.0),
                (Value::Null, 4.0, 4.0),
            ] {
                assert_eq!(safe_json::get_float(&json!({"k":v}), "k", d), e);
            }
        }
        ("models_extra", "test_get_object_returns_empty_on_non_mapping") => {
            assert!(safe_json::get_object(&json!({"k":[1]}), "k").is_empty());
            assert_eq!(
                json!(safe_json::get_object(&json!({"k":{"x":1}}), "k")),
                json!({"x":1})
            );
        }
        ("session_crypto", "test_boot_key_encrypt_decrypt_roundtrip") => assert_eq!(
            auth::decrypt_login_token(&auth::boot_encrypt("400115938").unwrap()).unwrap(),
            "400115938"
        ),
        ("session_crypto", "test_aes_encrypt_pads_with_nul_to_block_size") => {
            let c = auth::boot_encrypt("400115938").unwrap();
            assert_eq!(c.len(), 24);
            assert!(c.ends_with("=="));
        }
        ("session_crypto", "test_aes_encrypt_with_custom_key_roundtrip") => {
            use aes::cipher::{BlockDecryptMut, KeyIvInit, block_padding::NoPadding};
            let key = b"0123456789abcdef";
            let iv = b"abcdef0123456789";
            let c = auth::aes_encrypt("hello world", key, iv).unwrap();
            let d = cbc::Decryptor::<aes::Aes128>::new_from_slices(key, iv)
                .unwrap()
                .decrypt_padded_vec_mut::<NoPadding>(&B64.decode(c).unwrap())
                .unwrap();
            assert_eq!(
                String::from_utf8(d).unwrap().trim_end_matches('\0'),
                "hello world"
            );
        }
        ("session_crypto", "test_session_token_pins_against_known_inputs") => {
            let f = |t| {
                auth::session_token(
                    &"cafef00d".repeat(4),
                    "0123456789abcdef",
                    "abcdef0123456789",
                    t,
                )
                .unwrap()
            };
            let a = f(1700000000500);
            assert_eq!(a, f(1700000000500));
            assert_ne!(a, f(1700000001000));
        }
        ("aliyun_signing", "test_headers_match_signature_fixture") => {
            use md5::{Digest, Md5};
            let body=br#"{"a":"req","b":"1.0","c":{"apiVer":"1.0.8","language":"en-US","iotToken":"tok"},"d":{},"id":"req","params":{"$ref":"$.d"},"request":{"$ref":"$.c"},"version":"1.0"}"#;
            let path = "/uc/listBindingByAccount?x-ca-request-id=fixed";
            let nonce = "2b1d5870-52c2-4b91-9b2a-9c2c2f0da0b8";
            let date = "Tue, 19 May 2026 02:16:53 GMT";
            let h = auth::iot_headers(path, body, 1779157013115, nonce, date);
            let md5 = B64.encode(Md5::digest(body));
            for (k, v) in [
                ("x-ca-key", APP_KEY),
                ("x-ca-signature-method", "HmacSHA1"),
                (
                    "x-ca-signature-headers",
                    "x-ca-nonce,x-ca-timestamp,x-ca-key,x-ca-signature-method",
                ),
                ("accept", "application/json; charset=utf-8"),
                ("content-type", "application/octet-stream; charset=utf-8"),
                ("ca_version", "1"),
                ("x-ca-nonce", nonce),
                ("x-ca-timestamp", "1779157013115"),
                ("content-md5", &md5),
            ] {
                assert_eq!(h[k], v);
            }
            let canonical = format!(
                "POST\napplication/json; charset=utf-8\n{md5}\napplication/octet-stream; charset=utf-8\n{date}\nx-ca-key:{APP_KEY}\nx-ca-nonce:{nonce}\nx-ca-signature-method:HmacSHA1\nx-ca-timestamp:1779157013115\n{path}"
            );
            assert_eq!(
                h["x-ca-signature"],
                B64.encode(auth::hmac_sha1(APP_SECRET.as_bytes(), canonical.as_bytes()))
            );
        }
        ("aliyun_signing", "test_empty_body_gets_md5_of_empty_string") => assert_eq!(
            auth::iot_headers(
                "/uc/listBindingByAccount?x-ca-request-id=fixed",
                b"",
                0,
                "n",
                "d"
            )["content-md5"],
            "1B2M2Y8AsgTpgAmY7PhCfg=="
        ),
        ("oa_signing", "test_oa_signing_canonical_string") => {
            let (c, _) = auth::oa_body(&json!({"request":{"context":{"appKey":APP_KEY}}})).unwrap();
            assert_eq!(
                c,
                format!("request={{\"context\":{{\"appKey\":\"{APP_KEY}\"}}}}")
            );
            let date = "Tue, 19 May 2026 02:16:53 GMT";
            let h = auth::oa_headers("/api/prd/connect.json", &c, 1779157013, "fixed-nonce", date);
            for (k, v) in [
                ("x-ca-key", APP_KEY),
                ("x-ca-signature-method", "HmacSHA256"),
                ("accept", "application/json"),
                ("content-type", "application/x-www-form-urlencoded"),
                ("x-ca-nonce", "fixed-nonce"),
                ("x-ca-timestamp", "1779157013"),
            ] {
                assert_eq!(h[k], v);
            }
            let s = format!(
                "POST\napplication/json\n\napplication/x-www-form-urlencoded\n{date}\nx-ca-key:{APP_KEY}\nx-ca-nonce:fixed-nonce\nx-ca-signature-method:HmacSHA256\nx-ca-timestamp:1779157013\n/api/prd/connect.json?{c}"
            );
            assert_eq!(
                h["x-ca-signature"],
                B64.encode(auth::hmac_sha256(APP_SECRET.as_bytes(), s.as_bytes()))
            );
        }
        ("oa_signing", "test_oa_body_wire_form_url_encodes_json_values") => {
            let (c, w) = auth::oa_body(&json!({"request":{"k":"a&b"}})).unwrap();
            assert_eq!(c, "request={\"k\":\"a&b\"}");
            assert_eq!(w, "request=%7B%22k%22%3A%22a%26b%22%7D");
        }
        ("aliyun_transport", "test_encode_oa_body_url_encoded_vs_canonical") => {
            let (c, w) = auth::oa_body(&json!({"a":{"b":"c d"}})).unwrap();
            assert_eq!(c, "a={\"b\":\"c d\"}");
            assert_eq!(w, "a=%7B%22b%22%3A%22c+d%22%7D");
        }
        ("error_dispatch", "test_zero_code_returns_data") => assert_eq!(
            unwrap_rest(json!({"code":0,"data":{"hello":"world"}})).unwrap(),
            json!({"hello":"world"})
        ),
        (
            "error_dispatch",
            "test_session_expired_codes_dispatch_to_session_expired_error"
            | "test_invalid_credentials_codes_dispatch_to_invalid_credentials_error"
            | "test_unknown_auth_code_falls_back_to_authentication_error"
            | "test_non_auth_context_uses_plain_api_error",
        ) => {
            let code = if base.contains("non_auth") {
                1007
            } else if base.contains("unknown") {
                10050
            } else {
                name.split('[')
                    .nth(1)
                    .unwrap()
                    .trim_end_matches(']')
                    .parse()
                    .unwrap()
            };
            let e = unwrap_rest_context(
                json!({"request_id":"test","code":code,"message":"test error","data":null}),
                !base.contains("non_auth"),
            )
            .unwrap_err();
            assert_eq!(e.code(), Some(code));
            if base.contains("non_auth") {
                assert!(matches!(e, Error::Api(_)));
            } else if base.contains("session_expired_codes") {
                assert!(matches!(e, Error::SessionExpired(_)));
            } else if base.contains("invalid_credentials") {
                assert!(matches!(e, Error::InvalidCredentials(_)));
            } else {
                assert!(matches!(e, Error::Authentication(_)));
            }
        }
        ("aliyun_session_refresh", "test_unwrap_aliyun_maps_401_to_session_expired") => {
            let e = unwrap_iot(json!({"code":401,"message":"iotToken is invalid"})).unwrap_err();
            assert!(matches!(e, Error::SessionExpired(401)));
            assert_eq!(e.code(), Some(401));
            assert!(!format!("{e}").contains("iotToken"));
        }
        ("aliyun_session_refresh", "test_unwrap_aliyun_other_errors_stay_apierror") => {
            let e = unwrap_iot(json!({"code":500,"message":"internal"})).unwrap_err();
            assert!(matches!(e, Error::Api(500)));
            assert_eq!(e.code(), Some(500));
        }
        ("aliyun_session_refresh", "test_unwrap_aliyun_returns_data_on_success") => assert_eq!(
            unwrap_iot(json!({"code":200,"data":{"items":[]}})).unwrap(),
            json!({"items":[]})
        ),
        ("aliyun_handshake", "test_get_code_rejects_bool_and_non_int") => {
            for v in [json!(true), json!("200"), Value::Null] {
                assert!(matches!(unwrap_iot(json!({"code":v})), Err(Error::Api(-1))));
            }
            assert_eq!(unwrap_iot(json!({"code":200})).unwrap(), Value::Null);
        }
        ("status_update", "test_known_property_mapping_is_snake_case") => {
            let u = push(
                json!({"silentMode":{"time":1,"value":1},"childLockOnOff":{"time":1,"value":0},"Sand":{"time":1,"value":{"percent":87,"level":2,"sand":1234}},"catLeft":{"time":1,"value":{"kitten":1,"stayTime":42,"needClean":1}},"cleanCfg":{"time":1,"value":{"cleanType":1,"cleanParam":5,"active":1}}}),
            );
            assert_eq!(u.device_name, "PB01");
            assert_eq!(
                json!(u.changes),
                json!({"silent_mode":true,"child_lock":false,"sand_percent":87,"cat_present":true,"cat_stay_seconds":42,"needs_cleaning":true,"cleaning_enabled":true})
            );
        }
        ("status_update", "test_room_of_bin_maps_to_bucket_full") => {
            assert_eq!(
                json!(push(json!({"room_of_bin":{"time":1,"value":1}})).changes),
                json!({"bucket_full":true})
            );
            assert_eq!(
                json!(push(json!({"room_of_bin":{"time":1,"value":0}})).changes),
                json!({"bucket_full":false})
            );
        }
        ("status_update", "test_bucket_status_maps_to_operating_state") => assert_eq!(
            json!(push(json!({"bucketStatus":{"time":1,"value":2}})).changes),
            json!({"operating_state":OperatingState::Cleaning})
        ),
        ("status_update", "test_unknown_property_passes_through_under_camelcase") => {
            let u = push(
                json!({"silentMode":{"time":1,"value":1},"futureProp":{"time":1,"value":99},"Reboot":{"time":1,"value":{"reason":1,"rebootTS":"..."}}}),
            );
            assert_eq!(u.changes["silent_mode"], true);
            assert_eq!(u.changes["futureProp"], 99);
            assert_eq!(u.changes["Reboot"], json!({"reason":1,"rebootTS":"..."}));
        }
        ("status_update", "test_missing_device_or_items_drops_to_none") => {
            for v in [
                json!({"params":{"items":{"silentMode":{"value":1}}}}),
                json!({"params":{"deviceName":"PB01"}}),
                json!({}),
            ] {
                assert!(StatusUpdate::from_push(&v).is_none());
            }
        }
        _ => super::structural::run(group, name),
    }
}
