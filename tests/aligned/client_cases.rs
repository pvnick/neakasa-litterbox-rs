use super::support::*;
use crate::*;
use serde_json::{Value, json};
pub async fn run(group: &str, name: &str) {
    let b = base(name);
    let p = param(name);
    let w = Wire::default();
    let mut c = if group == "eu_gateway_integration" {
        let mut client = NeakasaClient::new("user@example.com", "pw", Region::Eu).unwrap();
        w.attach(&mut client.transport);
        client
    } else {
        client(&w, true)
    };
    match group {
        "client_commands" => {
            if b.contains("out_of_range") {
                assert!(matches!(
                    c.calibrate_sand("PB01", p.parse().unwrap()).await,
                    Err(Error::InvalidInput(_))
                ));
                assert_eq!(w.count(), 0);
                return;
            }
            if b == "test_resolve_role_returns_device_role" {
                assert_eq!(c.resolve("PB01").await.unwrap().role, DeviceRole::Owner);
                assert_eq!(w.count(), 0);
                return;
            }
            if b.contains("resolve_device") || b == "test_list_devices_builds_index" {
                let data = if b.contains("unknown") {
                    json!([])
                } else if b.contains("refreshes") {
                    json!([{"deviceName":"PB99","iotId":"iot-99","owned":1}])
                } else {
                    json!([{"deviceName":"PB01","iotId":"iot-1","owned":1},{"deviceName":"PB02","iotId":"iot-2","owned":0}])
                };
                w.json(json!({"code":200,"data":{"data":data}}));
                if b.contains("unknown") {
                    assert!(c.resolve("PBXX").await.is_err());
                } else if b.contains("refreshes") {
                    assert_eq!(c.resolve("PB99").await.unwrap().iot_id, "iot-99");
                } else {
                    assert_eq!(
                        c.list_devices()
                            .await
                            .unwrap()
                            .iter()
                            .map(|d| d.device_name.as_str())
                            .collect::<Vec<_>>(),
                        vec!["PB01", "PB02"]
                    );
                    assert_eq!(c.resolve("PB02").await.unwrap().role, DeviceRole::Shared);
                }
                assert_eq!(w.count(), 1);
                return;
            }
            let enabled = p.starts_with("True");
            let mut idx = 0;
            if b.contains("auto_clean") {
                w.json(json!({"code":200,"data":{"cleanCfg":{"value":if b.contains("disable"){json!({"cleanType":1,"cleanParam":3})}else{json!({"cleanType":2,"cleanParam":7,"active":0})}}}}));
                idx = 1;
            }
            w.json(json!({"code":200,"data":if b.contains("get_status"){json!({"Sand":{"value":{"percent":80}}})}else{json!({"ok":true})}}));
            let expected = match b {
                "test_start_clean_invokes_clean_now_with_flag" => {
                    c.start_clean("PB01").await.unwrap();
                    json!({"identifier":"cleanNow","args":{"bStartClean":1},"iotId":"iot-1"})
                }
                "test_stop_clean_sends_zero_flag" => {
                    c.stop_clean("PB01").await.unwrap();
                    json!({"identifier":"cleanNow","args":{"bStartClean":0},"iotId":"iot-1"})
                }
                "test_start_level_invokes_sand_leveling" => {
                    c.start_level("PB01").await.unwrap();
                    json!({"identifier":"sandLeveling","args":{"bStartLeveling":1},"iotId":"iot-1"})
                }
                "test_stop_level_sends_zero_flag" => {
                    c.stop_level("PB01").await.unwrap();
                    json!({"identifier":"sandLeveling","args":{"bStartLeveling":0},"iotId":"iot-1"})
                }
                "test_set_auto_level" => {
                    c.set_auto_level("PB01", enabled).await.unwrap();
                    json!({"iotId":"iot-1","items":{"autoLevel":i32::from(enabled)}})
                }
                "test_set_silent_mode" => {
                    c.set_silent_mode("PB01", enabled).await.unwrap();
                    json!({"iotId":"iot-1","items":{"silentMode":i32::from(enabled)}})
                }
                "test_set_child_lock" => {
                    c.set_child_lock("PB01", enabled).await.unwrap();
                    json!({"iotId":"iot-1","items":{"childLockOnOff":i32::from(enabled)}})
                }
                "test_set_auto_clean_preserves_existing_mode" => {
                    c.set_auto_clean("PB01", true).await.unwrap();
                    json!({"iotId":"iot-1","items":{"cleanCfg":{"cleanType":2,"cleanParam":7,"active":1}}})
                }
                "test_set_auto_clean_disable_sets_active_zero" => {
                    c.set_auto_clean("PB01", false).await.unwrap();
                    json!({"iotId":"iot-1","items":{"cleanCfg":{"cleanType":1,"cleanParam":3,"active":0}}})
                }
                "test_calibrate_sand_valid" => {
                    c.calibrate_sand("PB01", 50).await.unwrap();
                    json!({"iotId":"iot-1","identifier":"sandAdj","args":{"percent":50}})
                }
                "test_get_status_unwraps_properties" => {
                    assert_eq!(c.get_status("PB01").await.unwrap().sand_percent, 80);
                    json!({"iotId":"iot-1"})
                }
                _ => panic!("unmapped {name}"),
            };
            assert_eq!(w.body(idx)["params"], expected);
            assert_eq!(w.count(), idx + 1);
            let path = if b.contains("get_status") {
                "/thing/properties/get"
            } else if b.starts_with("test_set_") {
                "/thing/properties/set"
            } else {
                "/thing/service/invoke"
            };
            assert_eq!(w.path(idx), path);
        }
        "client_history" => match b {
            "test_get_toilet_records_signs_and_parses" => {
                w.json(json!({"code":0,"data":{"record_list":[{"id":1,"type":1}]}}));
                assert_eq!(
                    c.get_toilet_records("PB01", 100, 200).await.unwrap().len(),
                    1
                );
                assert_eq!(w.path(0), "/api/catbox/record");
                let q = w.query(0);
                for (k, v) in [
                    ("device_name", "PB01"),
                    ("bind_status", "1"),
                    ("start_time", "100"),
                    ("end_time", "200"),
                ] {
                    assert_eq!(q[k], v);
                }
            }
            "test_get_toilet_statistics_passes_zone" => {
                c.register_device(Device::from_api(&json!({"deviceName":"PB01","owned":0})));
                w.json(json!({"code":0,"data":[{"date":"2026-05-31","count":3}]}));
                assert_eq!(
                    c.get_toilet_statistics("PB01", 100, 200, -10800)
                        .await
                        .unwrap()
                        .len(),
                    1
                );
                let q = w.query(0);
                assert_eq!(q["zone"], "-10800");
                assert_eq!(q["bind_status"], "2");
            }
            "test_list_cats_parses_array" => {
                w.json(json!({"code":0,"data":[{"id":1,"name":"Mini"}]}));
                assert_eq!(
                    c.list_cats("PB01")
                        .await
                        .unwrap()
                        .iter()
                        .map(|c| c.name.as_str())
                        .collect::<Vec<_>>(),
                    vec!["Mini"]
                );
            }
            "test_authenticated_get_stamps_user_id_and_unwraps" => {
                w.json(json!({"code":0,"data":{"hello":"world"}}));
                assert_eq!(
                    c.rest_get(
                        "/catbox/cat/list",
                        vec![("device_name".into(), "PB01".into())]
                    )
                    .await
                    .unwrap(),
                    json!({"hello":"world"})
                );
                assert_eq!(w.query(0)["user_id"], "400068852");
                let r = w.requests.lock().unwrap();
                assert!(r[0].headers().contains_key("uid"));
                assert!(r[0].headers().contains_key("token"));
            }
            "test_require_session_raises_before_login" => assert!(matches!(
                client(&w, false).require_session(),
                Err(Error::NotAuthenticated)
            )),
            "test_require_iot_session_raises_without_token" => {
                c.resume_session(session(""));
                assert!(matches!(
                    c.iot("/x", "1", json!({})).await,
                    Err(Error::NotAuthenticated)
                ));
                assert_eq!(w.count(), 0);
            }
            "test_require_iot_session_returns_token" => {
                assert_eq!(c.require_iot_session().unwrap(), "iot-token")
            }
            "test_watch_status_requires_login" => assert!(matches!(
                client(&w, false).status_stream(Default::default()),
                Err(Error::NotAuthenticated)
            )),
            "test_watch_status_returns_stream" => {
                let _: StatusStream = c.status_stream(Default::default()).unwrap();
                assert_eq!(w.count(), 0);
            }
            "test_set_cat_weight_sends_only_weight_and_context" => {
                let role = if p.ends_with("-2") { 2 } else { 1 };
                let (weight, unit) = if p.starts_with("4.8") {
                    (4.8, WeightUnit::Kg)
                } else {
                    (10.58, WeightUnit::Lb)
                };
                c.register_device(Device::from_api(
                    &json!({"deviceName":"PB01","owned":if role==1{1}else{0}}),
                ));
                w.json(json!({"code":0,"data":null}));
                c.set_cat_weight("PB01", 7, weight, unit, -14400)
                    .await
                    .unwrap();
                assert_eq!(w.count(), 1);
                assert_eq!(w.path(0), "/api/catbox/cat/update");
                assert_eq!(
                    w.body(0),
                    json!({"device_name":"PB01","bind_status":role,"id":7,"weight":weight,"unit":unit,"scene":"catDetail","zone":-14400,"user_id":400068852})
                );
                let r = w.requests.lock().unwrap();
                assert!(!r[0].headers()["uid"].is_empty());
                assert!(!r[0].headers()["token"].is_empty());
            }
            "test_set_cat_weight_rejects_invalid_weight_before_io" => {
                if p == "True" {
                    assert!(serde_json::from_value::<f64>(json!(true)).is_err());
                } else {
                    let n = match p {
                        "nan" => f64::NAN,
                        "inf" => f64::INFINITY,
                        "-inf" => f64::NEG_INFINITY,
                        _ => p.parse().unwrap(),
                    };
                    assert!(matches!(
                        c.set_cat_weight("PB01", 7, n, WeightUnit::Kg, 0).await,
                        Err(Error::InvalidInput(_))
                    ));
                }
                assert_eq!(w.count(), 0);
            }
            "test_set_cat_weight_rejects_invalid_cat_id" => {
                if p == "True" {
                    assert!(serde_json::from_value::<i64>(json!(true)).is_err());
                } else {
                    assert!(matches!(
                        c.set_cat_weight("PB01", p.parse().unwrap(), 4.8, WeightUnit::Kg, 0)
                            .await,
                        Err(Error::InvalidInput(_))
                    ));
                }
                assert_eq!(w.count(), 0);
            }
            "test_set_cat_weight_rejects_unknown_unit" => {
                assert!(serde_json::from_value::<WeightUnit>(json!("oz")).is_err());
                assert_eq!(w.count(), 0);
            }
            "test_set_cat_weight_requires_login" => assert!(matches!(
                client(&w, false)
                    .set_cat_weight("PB01", 7, 4.8, WeightUnit::Kg, 0)
                    .await,
                Err(Error::NotAuthenticated)
            )),
            "test_set_cat_weight_does_not_retry_expired_session" => {
                w.json(json!({"code":1007,"data":null}));
                assert!(matches!(
                    c.set_cat_weight("PB01", 7, 4.8, WeightUnit::Kg, 0).await,
                    Err(Error::SessionExpired(1007))
                ));
                assert_eq!(w.count(), 1);
            }
            _ => panic!("unmapped {name}"),
        },
        "aliyun_handshake" => {
            let mut h = handshake();
            let expected = match b {
                "test_full_handshake_returns_iot_session_with_endpoint" => None,
                "test_empty_auth_token_raises_before_network" => {
                    let mut s = session("");
                    s.user_info.ali_authentication_token.clear();
                    c.resume_session(s);
                    Some("ValidationError")
                }
                "test_region_non_200_raises_apierror" => {
                    h[0] = json!({"code":500,"message":"boom"});
                    Some("ApiError")
                }
                "test_region_missing_endpoints_raises" => {
                    h[0] = json!({"code":200,"data":{"oaApiGatewayEndpoint":""}});
                    Some("ApiError")
                }
                "test_vid_top_level_failure_raises" => {
                    h[1] = json!({"success":"false","errorMsg":"nope"});
                    Some("ApiError")
                }
                "test_vid_inner_failure_raises" => {
                    h[1] = json!({"success":"true","data":{"successful":"false","message":"bad"}});
                    Some("ApiError")
                }
                "test_vid_missing_value_raises" => {
                    h[1] = json!({"success":"true","data":{"successful":"true"}});
                    Some("ApiError")
                }
                "test_sid_failure_raises_authentication_error" => {
                    h[2] = json!({"success":"false","errorMsg":"denied"});
                    Some("AuthenticationError")
                }
                "test_sid_missing_value_raises_authentication_error" => {
                    h[2] = json!({"success":"true","data":{"successful":"true","data":{}}});
                    Some("AuthenticationError")
                }
                "test_create_token_non_200_raises_authentication_error" => {
                    h[3] = json!({"code":401,"message":"expired"});
                    Some("AuthenticationError")
                }
                "test_create_token_missing_value_raises" => {
                    h[3] = json!({"code":200,"data":{}});
                    Some("AuthenticationError")
                }
                _ => panic!("unmapped {name}"),
            };
            for v in h {
                w.json(v);
            }
            let result = c.authenticate_aliyun().await;
            if let Some(kind) = expected {
                assert_eq!(result.unwrap_err().kind(), kind);
                if b.contains("empty_auth") {
                    assert_eq!(w.count(), 0);
                }
            } else {
                result.unwrap();
                assert_eq!(c.login_result().unwrap().iot_token, "iot-token-789");
                assert_eq!(c.login_result().unwrap().iot_host, "api.example.com");
                assert_eq!(w.count(), 4);
            }
        }
        "aliyun_session_refresh" => {
            let fallback = b.contains("falls_back");
            let direct = b.starts_with("test_refresh_");
            let retry = b.contains("refreshes_on") || b.contains("second_401");
            if fallback {
                w.json(json!({"code":500}));
                w.json(rest_login());
            }
            if retry {
                w.json(json!({"code":401,"message":"expired"}));
            }
            if direct || retry {
                for v in handshake() {
                    w.json(v);
                }
            }
            if !direct {
                w.json(if b.contains("second_401"){json!({"code":401,"message":"expired"})}else{json!({"code":200,"data":if b.contains("first_success"){json!([{"id":1}])}else{json!({"ok":true})}})});
            }
            if b.contains("regional_host") {
                let mut s = session("iot-token");
                s.iot_host = "eu-central-1.api-iot.aliyuncs.com".into();
                c.resume_session(s);
            }
            if direct {
                c.refresh_iot().await.unwrap();
                assert_eq!(w.count(), if fallback { 6 } else { 4 });
            } else {
                let r = c.iot("/uc/listBindingByAccount", "1.0.8", json!({})).await;
                if b.contains("second_401") {
                    assert!(matches!(r, Err(Error::SessionExpired(401))));
                } else {
                    assert_eq!(
                        r.unwrap(),
                        if b.contains("first_success") {
                            json!([{"id":1}])
                        } else {
                            json!({"ok":true})
                        }
                    );
                }
                assert_eq!(w.count(), if retry { 6 } else { 1 });
                if b.contains("regional_host") {
                    assert_eq!(
                        w.requests.lock().unwrap()[0].url().host_str(),
                        Some("eu-central-1.api-iot.aliyuncs.com")
                    );
                }
            }
        }
        "client_login" | "client_lifecycle" => login_cases(group, name, w).await,
        "eu_gateway_integration" => {
            let mut h = handshake();
            h[0]["data"]["oaApiGatewayEndpoint"] = json!("oa-eu.example.com");
            h[0]["data"]["apiGatewayEndpoint"] = json!("eu-central-1.api-iot.aliyuncs.com");
            for v in h {
                w.json(v);
            }
            w.json(json!({"code":200,"data":{"data":[]}}));
            let s = c.login(Some(session(""))).await.unwrap();
            assert_eq!(s.iot_host, "eu-central-1.api-iot.aliyuncs.com");
            assert!(c.list_devices().await.unwrap().is_empty());
            let req = w.requests.lock().unwrap();
            assert_eq!(
                req[0].url().host_str(),
                Some("cn-shanghai.api-iot.aliyuncs.com")
            );
            for i in [3, 4] {
                assert_eq!(
                    req[i].url().host_str(),
                    Some("eu-central-1.api-iot.aliyuncs.com")
                );
            }
            assert!(req.iter().all(|r| r.url().host_str() != Some(US_GATEWAY)));
        }
        _ => panic!("unmapped {group} {name}"),
    }
}
async fn login_cases(group: &str, name: &str, w: Wire) {
    let b = base(name);
    let mut c = client(&w, false);
    match b {
        "test_login_fresh_runs_rest_then_handshake" => {
            w.json(rest_login());
            for v in handshake() {
                w.json(v);
            }
            let r = c.login(None).await.unwrap();
            assert_eq!(r.iot_token, "iot-token-789");
            assert!(c.is_authenticated());
            assert_eq!(w.path(0), "/api/login");
            assert_eq!(w.path(1), "/living/account/region/get");
            assert_eq!(w.count(), 5);
        }
        "test_login_cached_with_iot_token_mints_nothing"
        | "test_login_cached_with_iot_token_restores_cached_aliyun_host" => {
            let mut s = session("cached-iot-token");
            if b.contains("cached_aliyun_host") {
                s.iot_host = "eu-central-1.api-iot.aliyuncs.com".into();
            }
            let result = c.login(Some(s.clone())).await.unwrap();
            assert_eq!(json!(result), json!(s));
            assert!(c.is_authenticated());
            assert_eq!(w.count(), 0);
        }
        "test_login_cached_without_iot_token_only_handshakes" => {
            let s = session("");
            for v in handshake() {
                w.json(v);
            }
            let result = c.login(Some(s.clone())).await.unwrap();
            assert_eq!(result.iot_token, "iot-token-789");
            assert!(s.iot_token.is_empty());
            assert_eq!(w.count(), 4);
        }
        "test_region_property" => assert_eq!(c.base_url(), Region::Us.web_url()),
        "test_login_result_property_starts_none" => {
            assert!(c.login_result().is_none());
            assert!(!c.is_authenticated());
        }
        "test_login_params_carry_sdk_user_agent_as_dev_sys_ver" => {
            w.json(rest_login());
            c.login_rest().await.unwrap();
            let v: Value = serde_json::from_str(&w.query(0)["data"]).unwrap();
            assert_eq!(v["devSysVer"], crate::constants::USER_AGENT);
        }
        "test_login_rest_parses_and_stores" => {
            w.json(rest_login());
            c.login_rest().await.unwrap();
            let s = c.login_result().unwrap();
            assert_eq!(s.user_id, "400068852");
            assert_eq!(s.user_info.ali_authentication_token, "ali-auth-token");
        }
        "test_authenticate_aliyun_stores_iot_token" => {
            c.resume_session(session(""));
            let mut h = handshake();
            h[0]["data"]["apiGatewayEndpoint"] = json!("eu-central-1.api-iot.aliyuncs.com");
            h[3]["data"]["iotToken"] = json!("fresh-iot-token");
            for v in h {
                w.json(v);
            }
            c.authenticate_aliyun().await.unwrap();
            let s = c.login_result().unwrap();
            assert_eq!(s.iot_token, "fresh-iot-token");
            assert_eq!(s.iot_host, "eu-central-1.api-iot.aliyuncs.com");
        }
        "test_close_delegates_to_both_transports" | "test_async_context_manager_closes_on_exit" => {
            let weak = std::sync::Arc::downgrade(c.transport.mock.as_ref().unwrap());
            assert!(weak.upgrade().is_some());
            drop(c);
            assert!(weak.upgrade().is_none());
        }
        _ => super::pure::run(group, name),
    }
}
