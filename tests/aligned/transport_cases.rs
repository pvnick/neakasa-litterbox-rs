use super::support::*;
use crate::*;
use serde_json::json;
pub async fn run(group: &str, name: &str) {
    let b = base(name);
    let w = Wire::default();
    let t = w.transport();
    let aliyun = group == "aliyun_transport";
    if b.contains("close") {
        let weak = std::sync::Arc::downgrade(t.mock.as_ref().unwrap());
        if b.contains("injected") {
            let shared = t.clone();
            drop(t);
            assert!(weak.upgrade().is_some());
            drop(shared);
            assert!(weak.upgrade().is_none());
        } else {
            if !b.contains("never_used") {
                w.json(json!({"code":200}));
                if aliyun {
                    t.iot(US_GATEWAY, "/x", "1.0", None, json!({}), "en-US")
                        .await
                        .unwrap();
                } else if b.contains("post") {
                    t.post_json("https://host/api", json!({}), Default::default())
                        .await
                        .unwrap();
                } else {
                    t.get("https://host/api", &[], Default::default())
                        .await
                        .unwrap();
                }
            }
            let mut owned = Some(t);
            drop(owned.take());
            drop(owned.take());
            assert!(weak.upgrade().is_none());
        }
        return;
    }
    let mut h = auth::rest_headers("en", auth::now_ms() / 1000);
    if b.contains("authenticated") {
        h = auth::authenticated_headers(
            &session("iot-token"),
            if b.contains("sends_json") { "pt" } else { "en" },
            auth::now_ms(),
        )
        .unwrap();
        h.insert("uid".into(), "enc".into());
        h.insert("token".into(), "tok".into());
    }
    let err = b.contains("error") || b.contains("timeout");
    if b.contains("status") || b == "test_authenticated_post_http_error" {
        w.responses.lock().unwrap().push_back(Ok((
            if aliyun { 500 } else { 503 },
            b"internal error".to_vec(),
        )));
    } else if err {
        w.error(if b.contains("timeout") || name.contains("[error1]") {
            crate::transport::IoFailure::Timeout
        } else {
            crate::transport::IoFailure::Network
        });
    } else {
        w.json(if aliyun {
            json!({"code":200,"data":{"ok":true}})
        } else if b.contains("decodes") {
            json!({"code":0,"data":{"x":1}})
        } else {
            json!({"code":0,"data":null})
        });
    }
    let r = if aliyun {
        if b == "test_call_oa_form_encodes_and_signs" {
            t.oa(
                "oa.example.com",
                "/api/prd/connect.json",
                json!({"request":{"context":{"appKey":"k"}}}),
                Some("v1"),
            )
            .await
        } else {
            t.iot(
                US_GATEWAY,
                if err {
                    "/x"
                } else {
                    "/uc/listBindingByAccount"
                },
                "1.0.8",
                Some("tok"),
                json!({}),
                "en-US",
            )
            .await
        }
    } else if b.contains("authenticated_post") {
        t.post_json(
            if b.contains("sends_json") {
                "https://host/api/catbox/cat/update"
            } else {
                "https://host/api"
            },
            if b.contains("sends_json") {
                json!({"id":7,"weight":4.8,"unit":"kg"})
            } else {
                json!({})
            },
            h,
        )
        .await
    } else if b.contains("data_envelope") {
        t.get("https://host/api", &[], h.clone()).await.unwrap();
        w.json(json!({"code":0}));
        t.get(
            "https://host/login",
            &[("data".into(), json!({"email":"a@b.c"}).to_string())],
            h,
        )
        .await
    } else {
        t.get(
            if b.contains("authenticated") {
                "https://host/api/x"
            } else {
                "https://host/api"
            },
            &[("a".into(), "b".into())],
            h,
        )
        .await
    };
    if err {
        let e = r.unwrap_err();
        assert_eq!(e.kind(), "TransportError");
        if b.contains("status") || b == "test_authenticated_post_http_error" {
            assert!(matches!(e,Error::Http(n) if n==if aliyun{500}else{503}));
        }
        assert!(!e.to_string().contains("internal error"));
        return;
    }
    let v = r.unwrap();
    let req = w.requests.lock().unwrap();
    let last = req.last().unwrap();
    match b {
        "test_signed_get_decodes_json_object" => {
            assert_eq!(v, json!({"code":0,"data":{"x":1}}));
            assert_eq!(last.url().as_str(), "https://host/api?a=b");
        }
        "test_signed_get_data_envelope_wraps_params" => {
            assert!(last.url().as_str().starts_with("https://host/login?data="))
        }
        "test_authenticated_get_sets_uid_and_token_headers" => {
            assert_eq!(last.headers()["uid"], "enc");
            assert_eq!(last.headers()["token"], "tok");
        }
        "test_authenticated_post_sends_json_and_session_headers" => {
            assert_eq!(v, json!({"code":0,"data":null}));
            assert_eq!(last.url().as_str(), "https://host/api/catbox/cat/update");
            assert_eq!(
                serde_json::from_slice::<serde_json::Value>(
                    last.body().unwrap().as_bytes().unwrap()
                )
                .unwrap(),
                json!({"id":7,"weight":4.8,"unit":"kg"})
            );
            for (k, v) in [
                ("content-type", "application/json"),
                ("uid", "enc"),
                ("token", "tok"),
                ("accept-language", "pt"),
            ] {
                assert_eq!(last.headers()[k], v);
            }
            assert!(!last.headers().contains_key("appId"));
        }
        "test_call_posts_iot_envelope" => {
            assert_eq!(v, json!({"code":200,"data":{"ok":true}}));
            assert!(last.url().query().unwrap().contains("x-ca-request-id="));
            assert!(
                String::from_utf8_lossy(last.body().unwrap().as_bytes().unwrap())
                    .contains("\"apiVer\":\"1.0.8\"")
            );
            assert!(!last.headers()["x-ca-key"].is_empty());
        }
        "test_call_oa_form_encodes_and_signs" => {
            assert_eq!(
                last.url().as_str(),
                "https://oa.example.com/api/prd/connect.json"
            );
            assert_eq!(last.headers()["vid"], "v1");
            assert!(
                last.body()
                    .unwrap()
                    .as_bytes()
                    .unwrap()
                    .starts_with(b"request=")
            );
        }
        _ => panic!("unmapped {name}"),
    }
}
