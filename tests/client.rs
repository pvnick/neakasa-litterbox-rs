use neakasa_litterbox_sdk::*;
use serde_json::{Value, json};
use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{body_json, header, method, path, query_param},
};

fn session(host: &str) -> LoginResult {
    LoginResult {
        user_id: "456".into(),
        user_token: "test-token".into(),
        aes_key: "0123456789abcdef".into(),
        aes_iv: "abcdef0123456789".into(),
        user_info: UserInfo {
            user_id: 1,
            user_name: "test@example.invalid".into(),
            ali_user_id: 456,
            ali_authentication_token: "test-auth".into(),
        },
        issued_at: 0.0,
        iot_token: "test-iot".into(),
        iot_host: host.into(),
    }
}
fn device() -> Device {
    Device::from_api(&json!({"iotId":"iot-1","deviceName":"TEST01","productName":"Test","owned":1}))
}
fn client(server: &MockServer) -> NeakasaClient {
    let mut c = NeakasaClient::with_endpoint(
        "test@example.invalid",
        "password",
        &format!("{}/api", server.uri()),
        Duration::from_secs(2),
    )
    .unwrap();
    c.resume_session(session(&server.uri()));
    c.register_device(device());
    c.set_bootstrap_host(server.uri()).unwrap();
    c
}
async fn respond(server: &MockServer, route: &str, value: Value) {
    Mock::given(path(route))
        .respond_with(ResponseTemplate::new(200).set_body_json(value))
        .mount(server)
        .await;
}
async fn handshake(server: &MockServer) {
    respond(server,"/living/account/region/get",json!({"code":200,"data":{"oaApiGatewayEndpoint":server.uri(),"apiGatewayEndpoint":server.uri()}})).await;
    respond(
        server,
        "/api/prd/connect.json",
        json!({"success":"true","data":{"successful":"true","vid":"vid"}}),
    )
    .await;
    respond(server,"/api/prd/loginbyoauth.json",json!({"success":"true","data":{"successful":"true","data":{"loginSuccessResult":{"sid":"sid"}}}})).await;
    respond(
        server,
        "/account/createSessionByAuthCode",
        json!({"code":200,"data":{"iotToken":"new-iot"}}),
    )
    .await;
}

#[tokio::test]
async fn cat_weight_sends_only_intended_fields_in_json_for_both_units_and_roles() {
    for (unit, weight, role) in [
        (WeightUnit::Kg, 4.8, DeviceRole::Owner),
        (WeightUnit::Lb, 10.58, DeviceRole::Shared),
    ] {
        let server = MockServer::start().await;
        let mut c = client(&server);
        let mut d = device();
        d.role = role;
        c.register_device(d);
        Mock::given(path("/api/catbox/cat/update")).and(method("POST")).and(header("content-type","application/json"))
            .and(body_json(json!({"device_name":"TEST01","bind_status":role,"id":7,"weight":weight,"unit":unit,"scene":"catDetail","zone":-14400,"user_id":456})))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"code":0,"data":null}))).expect(1).mount(&server).await;
        c.set_cat_weight("TEST01", 7, weight, unit, -14400)
            .await
            .unwrap();
        let requests = server.received_requests().await.unwrap();
        let h = &requests[0].headers;
        assert!(h.contains_key("uid"));
        assert!(h.contains_key("token"));
        assert!(!h.contains_key("appid"));
    }
}
#[tokio::test]
async fn bad_weight_and_id_fail_before_network() {
    let server = MockServer::start().await;
    let mut c = client(&server);
    for weight in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(matches!(
            c.set_cat_weight("TEST01", 7, weight, WeightUnit::Kg, 0)
                .await,
            Err(Error::InvalidInput(_))
        ));
    }
    for id in [0, -1] {
        assert!(
            c.set_cat_weight("TEST01", id, 4.8, WeightUnit::Kg, 0)
                .await
                .is_err()
        );
    }
    assert!(server.received_requests().await.unwrap().is_empty());
}
#[tokio::test]
async fn session_expired_and_invalid_credentials_are_not_retried_on_mutation() {
    for code in [1007, 3026, 3027, 3019, 10060, 10061, 10192, 10050] {
        let server = MockServer::start().await;
        let mut c = client(&server);
        Mock::given(path("/api/catbox/cat/update"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(json!({"code":code,"message":"sensitive token"})),
            )
            .expect(1)
            .mount(&server)
            .await;
        let e = c
            .set_cat_weight("TEST01", 7, 4.8, WeightUnit::Kg, 0)
            .await
            .unwrap_err();
        assert_eq!(e.code(), Some(code));
        assert!(!e.to_string().contains("sensitive"));
        if [1007, 3026, 3027].contains(&code) {
            assert!(matches!(e, Error::SessionExpired(_)));
        }
        if [3019, 10060, 10061, 10192].contains(&code) {
            assert!(matches!(e, Error::InvalidCredentials(_)));
        }
    }
}
#[tokio::test]
async fn read_endpoints_parse_records_and_statistics_and_sign_query() {
    let server = MockServer::start().await;
    let mut c = client(&server);
    Mock::given(path("/api/catbox/cat/list"))
        .and(query_param("user_id", "456"))
        .and(query_param("bind_status", "1"))
        .and(query_param("device_name", "TEST01"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            json!({"code":0,"data":[{"id":7,"name":"Example","weight":4.8,"unit":"kg"}]}),
        ))
        .mount(&server)
        .await;
    assert_eq!(c.list_cats("TEST01").await.unwrap()[0].name, "Example");
    Mock::given(path("/api/catbox/record")).and(query_param("start_time","100")).and(query_param("end_time","200"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"code":0,"data":{"record_list":[{"record_id":8,"start_time":100,"end_time":147,"type":1,"weight":4.8,"unit":"kg"}]}}))).mount(&server).await;
    assert_eq!(
        c.get_toilet_records("TEST01", 100, 200).await.unwrap()[0].duration_seconds(),
        47
    );
    Mock::given(path("/api/catbox/toilet/statistics"))
        .and(query_param("zone", "-14400"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"code":0,"data":[{"num":2,"date":"2026-10-15"}]})),
        )
        .mount(&server)
        .await;
    assert_eq!(
        c.get_toilet_statistics("TEST01", 100, 200, -14400)
            .await
            .unwrap()[0]
            .num,
        2
    );
}
#[tokio::test]
async fn clean_toggle_preserves_mode_delay_and_unknown_fields() {
    let server = MockServer::start().await;
    let mut c = client(&server);
    respond(&server,"/thing/properties/get",json!({"code":200,"data":{"cleanCfg":{"value":{"active":1,"cleanType":2,"cleanParam":15,"future":9}}}})).await;
    respond(
        &server,
        "/thing/properties/set",
        json!({"code":200,"data":{}}),
    )
    .await;
    c.set_auto_clean("TEST01", false).await.unwrap();
    let reqs = server.received_requests().await.unwrap();
    let v: Value = serde_json::from_slice(&reqs[1].body).unwrap();
    assert_eq!(
        v["params"],
        json!({"iotId":"iot-1","items":{"cleanCfg":{"active":0,"cleanType":2,"cleanParam":15,"future":9}}})
    );
    assert_eq!(v["request"]["iotToken"], "test-iot");
    assert_eq!(v["request"]["apiVer"], "1.0.5");
}
#[tokio::test]
async fn commands_have_correct_identifiers_and_do_not_run_without_explicit_calls() {
    let server = MockServer::start().await;
    let mut c = client(&server);
    respond(
        &server,
        "/thing/service/invoke",
        json!({"code":200,"data":{}}),
    )
    .await;
    respond(
        &server,
        "/thing/properties/set",
        json!({"code":200,"data":{}}),
    )
    .await;
    c.start_clean("TEST01").await.unwrap();
    c.stop_clean("TEST01").await.unwrap();
    c.start_level("TEST01").await.unwrap();
    c.stop_level("TEST01").await.unwrap();
    c.calibrate_sand("TEST01", 40).await.unwrap();
    assert!(c.calibrate_sand("TEST01", 0).await.is_err());
    assert!(c.calibrate_sand("TEST01", 101).await.is_err());
    c.set_auto_level("TEST01", true).await.unwrap();
    c.set_child_lock("TEST01", false).await.unwrap();
    c.set_silent_mode("TEST01", true).await.unwrap();
    let reqs = server.received_requests().await.unwrap();
    assert_eq!(reqs.len(), 8);
    let bodies: Vec<Value> = reqs
        .iter()
        .map(|r| serde_json::from_slice::<Value>(&r.body).unwrap()["params"].clone())
        .collect();
    for (i, (identifier, args)) in [
        ("cleanNow", json!({"bStartClean":1})),
        ("cleanNow", json!({"bStartClean":0})),
        ("sandLeveling", json!({"bStartLeveling":1})),
        ("sandLeveling", json!({"bStartLeveling":0})),
        ("sandAdj", json!({"percent":40})),
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(bodies[i]["identifier"], identifier);
        assert_eq!(bodies[i]["args"], args);
    }
    assert_eq!(bodies[5]["items"], json!({"autoLevel":1}));
    assert_eq!(bodies[6]["items"], json!({"childLockOnOff":0}));
    assert_eq!(bodies[7]["items"], json!({"silentMode":1}));
}
#[tokio::test]
async fn full_login_handshake_and_regional_device_lookup() {
    let server = MockServer::start().await;
    let mut c = client(&server);
    handshake(&server).await;
    let f: Value = serde_json::from_str(include_str!("fixtures/android.json")).unwrap();
    Mock::given(path("/api/login"))
        .and(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"code":0,"data":f["user"]})))
        .expect(1)
        .mount(&server)
        .await;
    let login = c.login(None).await.unwrap();
    assert_eq!(login.user_id, "456");
    assert_eq!(login.iot_host, server.uri());
    assert_eq!(login.iot_token, "new-iot");
    respond(
        &server,
        "/uc/listBindingByAccount",
        json!({"code":200,"data":{"data":[{"iotId":"iot-1","deviceName":"TEST01","owned":1}]}}),
    )
    .await;
    assert_eq!(c.list_devices().await.unwrap()[0].role, DeviceRole::Owner);
    let reqs = server.received_requests().await.unwrap();
    let login_params = reqs[0]
        .url
        .query_pairs()
        .find(|(k, _)| k == "data")
        .unwrap()
        .1
        .into_owned();
    let login_params: Value = serde_json::from_str(&login_params).unwrap();
    assert_eq!(login_params["password"], auth::md5_double_hex("password"));
    assert_eq!(reqs.len(), 6);
}
#[tokio::test]
async fn iot_401_refreshes_once_and_preserves_cached_rest_session() {
    let server = MockServer::start().await;
    let mut c = client(&server);
    handshake(&server).await;
    let calls = Arc::new(AtomicUsize::new(0));
    let count = calls.clone();
    Mock::given(path("/thing/properties/get"))
        .respond_with(move |_: &wiremock::Request| {
            if count.fetch_add(1, Ordering::SeqCst) == 0 {
                ResponseTemplate::new(200).set_body_json(json!({"code":401}))
            } else {
                ResponseTemplate::new(200)
                    .set_body_json(json!({"code":200,"data":{"Sand":{"value":{"percent":42}}}}))
            }
        })
        .expect(2)
        .mount(&server)
        .await;
    assert_eq!(c.get_status("TEST01").await.unwrap().sand_percent, 42);
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    assert_eq!(c.login_result().unwrap().iot_token, "new-iot");
}
#[tokio::test]
async fn cached_login_is_network_free_and_requires_credentials_for_fresh_login() {
    let server = MockServer::start().await;
    let mut c = client(&server);
    c.login(Some(session(&server.uri()))).await.unwrap();
    assert!(server.received_requests().await.unwrap().is_empty());
    let mut empty = NeakasaClient::new("", "", Region::Us).unwrap();
    assert!(matches!(
        empty.list_cats("x").await,
        Err(Error::NotAuthenticated)
    ));
    assert!(matches!(
        empty.login(None).await,
        Err(Error::InvalidInput(_))
    ));
}
#[tokio::test]
async fn http_errors_malformed_responses_and_timeouts_are_redacted() {
    for response in [
        ResponseTemplate::new(503).set_body_string("secret-token"),
        ResponseTemplate::new(200).set_body_string("not json secret-token"),
        ResponseTemplate::new(200).set_body_json(json!([])),
    ] {
        let server = MockServer::start().await;
        let mut c = client(&server);
        Mock::given(path("/api/catbox/cat/list"))
            .respond_with(response)
            .mount(&server)
            .await;
        let error = c.list_cats("TEST01").await.unwrap_err();
        assert!(!format!("{error:?}").contains("secret"));
    }
    let server = MockServer::start().await;
    Mock::given(path("/api/catbox/cat/list"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_millis(100)))
        .mount(&server)
        .await;
    let mut c = NeakasaClient::with_endpoint(
        "",
        "",
        &format!("{}/api", server.uri()),
        Duration::from_millis(10),
    )
    .unwrap();
    c.resume_session(session(&server.uri()));
    c.register_device(device());
    assert!(matches!(c.list_cats("TEST01").await, Err(Error::Transport)));
}
