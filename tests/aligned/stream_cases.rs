use super::support::*;
use crate::{mqtt::*, stream::*, *};
use rustls::client::danger::ServerCertVerifier;
use serde_json::{Value, json};
use std::{
    collections::VecDeque,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
use tokio::sync::{mpsc, oneshot};
type Message = Result<Option<(String, Vec<u8>)>>;
type Calls = Arc<Mutex<Vec<(String, String, Vec<u8>, u8)>>>;
#[derive(Clone)]
struct Fake {
    tx: mpsc::UnboundedSender<Message>,
    rx: Arc<Mutex<Option<mpsc::UnboundedReceiver<Message>>>>,
    calls: Calls,
    reply: Arc<Mutex<Option<i64>>>,
    fail_sub: Arc<Mutex<bool>>,
    live: Arc<AtomicUsize>,
}
impl Fake {
    fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            tx,
            rx: Arc::new(Mutex::new(Some(rx))),
            calls: Default::default(),
            reply: Arc::new(Mutex::new(Some(200))),
            fail_sub: Default::default(),
            live: Default::default(),
        }
    }
    fn send(&self, topic: &str, payload: &[u8]) {
        self.tx
            .send(Ok(Some((topic.into(), payload.into()))))
            .unwrap();
    }
    fn loss(&self, e: Error) {
        self.tx.send(Err(e)).unwrap();
    }
    fn end(&self) {
        self.tx.send(Ok(None)).unwrap();
    }
    fn connection(&self) -> Connection {
        self.live.fetch_add(1, Ordering::SeqCst);
        (
            Arc::new(self.clone()),
            Box::new(FakeEvents {
                rx: self.rx.lock().unwrap().take().expect("connected twice"),
                live: self.live.clone(),
            }),
        )
    }
}
struct FakeEvents {
    rx: mpsc::UnboundedReceiver<Message>,
    live: Arc<AtomicUsize>,
}
impl Drop for FakeEvents {
    fn drop(&mut self) {
        self.live.fetch_sub(1, Ordering::SeqCst);
    }
}
impl Events for FakeEvents {
    fn next(&mut self) -> FutureResult<'_, Option<(String, Vec<u8>)>> {
        Box::pin(async move { self.rx.recv().await.unwrap_or(Ok(None)) })
    }
}
impl Sink for Fake {
    fn subscribe<'a>(&'a self, topic: &'a str, qos: u8) -> FutureResult<'a, ()> {
        Box::pin(async move {
            self.calls
                .lock()
                .unwrap()
                .push(("subscribe".into(), topic.into(), vec![], qos));
            if *self.fail_sub.lock().unwrap() {
                Err(Error::Transport)
            } else {
                Ok(())
            }
        })
    }
    fn publish<'a>(&'a self, topic: &'a str, payload: Vec<u8>, qos: u8) -> FutureResult<'a, ()> {
        Box::pin(async move {
            self.calls
                .lock()
                .unwrap()
                .push(("publish".into(), topic.into(), payload.clone(), qos));
            if topic.ends_with("/account/bind")
                && let Some(code) = *self.reply.lock().unwrap()
            {
                let v: Value = serde_json::from_slice(&payload).unwrap();
                self.send(
                    "/sys/pk/dn/app/down/account/bind_reply",
                    &serde_json::to_vec(&json!({"id":v["id"],"code":code})).unwrap(),
                );
            }
            Ok(())
        })
    }
}
fn factory(sequence: Vec<Result<Fake>>) -> (Factory, Arc<AtomicUsize>) {
    let q = Arc::new(Mutex::new(VecDeque::from(sequence)));
    let count = Arc::new(AtomicUsize::new(0));
    let n = count.clone();
    (
        Arc::new(move |_| {
            n.fetch_add(1, Ordering::SeqCst);
            let next = q
                .lock()
                .unwrap()
                .pop_front()
                .expect("unexpected MQTT connection");
            Box::pin(async move { Ok(next?.connection()) })
        }),
        count,
    )
}
fn opts() -> StreamOptions {
    StreamOptions {
        tls_config: Some(
            build_tls(Some(
                std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/broker-ca.pem"),
            ))
            .unwrap(),
        ),
        reconnect: None,
        ..Default::default()
    }
}
fn mqtt(f: &Fake, handler: Handler) -> MqttTransport {
    let (factory, _) = factory(vec![Ok(f.clone())]);
    MqttTransport::new("pk", "dn", "dn&pk", factory, opts(), handler)
}
async fn wait_for(mut f: impl FnMut() -> bool) {
    tokio::time::timeout(Duration::from_secs(2), async {
        while !f() {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
}
fn push(items: Value) -> Vec<u8> {
    serde_json::to_vec(&json!({"params":{"deviceName":"PB01","items":items}})).unwrap()
}
const TOPIC: &str = "/sys/pk/dn/thing/properties";
pub async fn run(group: &str, name: &str) {
    let b = base(name);
    match group {
        "mqtt_auth" => match b {
            "test_authinfo_sign_is_hex_lower_hmac_sha1" => {
                let info = auth::mqtt_auth_info(
                    1779157013115,
                    "cdf9bxg7",
                    "m7yr2bp9zoxn9aj5e6yvbgsl7v3rxqdk",
                );
                let canonical = format!(
                    "appKey{}clientIdcdf9bxg7deviceSnm7yr2bp9zoxn9aj5e6yvbgsl7v3rxqdktimestamp1779157013115",
                    crate::constants::APP_KEY
                );
                assert_eq!(info["clientId"], "cdf9bxg7");
                assert_eq!(info["deviceSn"], "m7yr2bp9zoxn9aj5e6yvbgsl7v3rxqdk");
                assert_eq!(info["timestamp"], "1779157013115");
                assert_eq!(
                    info["sign"],
                    auth::hex(&auth::hmac_sha1(
                        crate::constants::APP_SECRET.as_bytes(),
                        canonical.as_bytes()
                    ))
                );
            }
            "test_authinfo_random_values_are_well_formed" => {
                let i = random_auth_info(1779157013115);
                for (k, n) in [("clientId", 8), ("deviceSn", 32)] {
                    let s = i[k].as_str().unwrap();
                    assert_eq!(s.len(), n);
                    assert!(
                        s.bytes()
                            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
                    );
                }
            }
            "test_mqtt_password_sign" => {
                let pk = "a4HiUVpAgt9";
                let dn = "deviceabc1234567890";
                let ds = "secret_xyz";
                let c = format!("clientId{dn}&{pk}deviceName{dn}productKey{pk}");
                let expected =
                    auth::hex(&auth::hmac_sha1(ds.as_bytes(), c.as_bytes())).to_uppercase();
                let sign = mqtt_credentials(pk, dn, ds, US_GATEWAY).3;
                assert_eq!(sign, expected);
                assert_eq!(sign.len(), 40);
                assert_eq!(sign, sign.to_uppercase());
            }
            _ => panic!("{name}"),
        },
        "mqtt_auth_derive" => {
            let w = Wire::default();
            w.json(match b{"test_derive_raises_on_non_200_code"=>json!({"code":403,"message":"denied"}),"test_derive_raises_when_data_not_object"=>json!({"code":200,"data":"oops"}),"test_derive_raises_on_empty_triple_fields"=>json!({"code":200,"data":{"productKey":"pk","deviceName":"","deviceSecret":"s"}}),_=>json!({"code":200,"data":{"productKey":"pk1","deviceName":"dn1","deviceSecret":"secret"}})});
            let host = if b.contains("regional") {
                "eu-central-1.api-iot.aliyuncs.com"
            } else {
                US_GATEWAY
            };
            let r = derive_mqtt_credentials(&w.transport(), host).await;
            if b.contains("raises") {
                assert!(matches!(r, Err(Error::Api(_))));
            } else {
                let c = r.unwrap();
                assert_eq!(c.username, "dn1&pk1");
                assert_eq!(
                    c.client_id,
                    "dn1&pk1|securemode=2,signmethod=hmacsha1,ext=1|"
                );
                assert_eq!(
                    c.host,
                    if b.contains("regional") {
                        "pk1.iot-as-mqtt.eu-central-1.aliyuncs.com"
                    } else {
                        "pk1.iot-as-mqtt.us-east-1.aliyuncs.com"
                    }
                );
                assert_eq!(c.port, 1883);
                assert_eq!(c.product_key, "pk1");
                assert_eq!(c.device_name, "dn1");
                assert_eq!(
                    c.password,
                    auth::hex(&auth::hmac_sha1(
                        b"secret",
                        b"clientIddn1&pk1deviceNamedn1productKeypk1"
                    ))
                    .to_uppercase()
                );
                assert_eq!(w.requests.lock().unwrap()[0].url().host_str(), Some(host));
            }
        }
        "mqtt_transport_flow" => flow(b).await,
        "mqtt_transport" => tls(b).await,
        "status_stream" => status(b).await,
        _ => panic!("unmapped {group} {name}"),
    }
}
async fn flow(b: &str) {
    let f = Fake::new();
    let captured = Arc::new(Mutex::new(Vec::new()));
    let c = captured.clone();
    let handler: Handler = Arc::new(move |t, p| {
        c.lock().unwrap().push((t, p));
        Box::pin(async { Ok(()) })
    });
    let mut t = mqtt(&f, handler);
    match b {
        "test_topic_prefix" => assert_eq!(t.topic_prefix, "/sys/pk/dn"),
        "test_require_client_raises_when_disconnected" => {
            assert!(matches!(t.require_client(), Err(Error::Transport)))
        }
        "test_subscribe_and_publish_delegate_to_client" => {
            t.connect().await.unwrap();
            t.subscribe("topic/#", 1).await.unwrap();
            t.publish("topic", b"payload".to_vec(), 1).await.unwrap();
            assert_eq!(
                *f.calls.lock().unwrap(),
                vec![
                    ("subscribe".into(), "topic/#".into(), vec![], 1),
                    ("publish".into(), "topic".into(), b"payload".to_vec(), 1)
                ]
            );
        }
        "test_bind_account_success_resolves_on_matching_reply"
        | "test_bind_account_raises_on_non_200_reply"
        | "test_bind_account_times_out" => {
            *f.reply.lock().unwrap() = if b.contains("times_out") {
                None
            } else if b.contains("non_200") {
                Some(500)
            } else {
                Some(200)
            };
            t.connect().await.unwrap();
            let r = t.bind_account("iot-token", Duration::from_millis(10)).await;
            if b.contains("times_out") {
                assert!(matches!(r, Err(Error::Timeout)));
            } else if b.contains("non_200") {
                assert!(matches!(r, Err(Error::Api(500))));
            } else {
                r.unwrap();
                let calls = f.calls.lock().unwrap();
                let v: Value = serde_json::from_slice(&calls[0].2).unwrap();
                assert!(!v["id"].as_str().unwrap().is_empty());
            }
            assert!(t.pending.lock().unwrap().is_empty());
        }
        "test_try_dispatch_reply_routes_matching_id" => {
            let (tx, rx) = oneshot::channel();
            t.pending.lock().unwrap().insert("abc".into(), tx);
            assert!(t.try_dispatch_reply(br#"{"id":"abc","code":200}"#));
            assert_eq!(rx.await.unwrap(), json!({"id":"abc","code":200}));
        }
        "test_try_dispatch_reply_ignores_unparsable_or_unmatched" => {
            for p in [
                b"not json".as_slice(),
                br#"{"no_id":1}"#,
                br#"{"id":"unknown"}"#,
            ] {
                assert!(!t.try_dispatch_reply(p));
            }
        }
        "test_try_dispatch_reply_ignores_already_done_future" => {
            let (tx, rx) = oneshot::channel();
            drop(rx);
            t.pending.lock().unwrap().insert("done".into(), tx);
            assert!(!t.try_dispatch_reply(br#"{"id":"done"}"#));
        }
        "test_disconnect_cancels_task_and_exits_client"
        | "test_disconnect_suppresses_mqtt_error_on_exit" => {
            t.connect().await.unwrap();
            if b.contains("suppresses") {
                f.loss(Error::Mqtt);
            }
            t.disconnect().await;
            assert!(!t.is_connected());
            assert_eq!(f.live.load(Ordering::SeqCst), 0);
        }
        "test_disconnect_is_idempotent" => {
            t.disconnect().await;
            t.disconnect().await;
            assert!(!t.is_connected());
        }
        "test_connect_is_noop_when_already_connected" => {
            t.connect().await.unwrap();
            let before = t.require_client().unwrap().clone();
            t.connect().await.unwrap();
            assert!(Arc::ptr_eq(&before, t.require_client().unwrap()));
        }
        "test_dispatch_loop_propagates_cancellation"
        | "test_dispatch_loop_does_not_signal_on_cancellation" => {
            t.connect().await.unwrap();
            t.disconnect().await;
            assert!(t.lost.try_recv().is_err());
            assert_eq!(f.live.load(Ordering::SeqCst), 0);
        }
        "test_dispatch_loop_routes_property_message_to_handler"
        | "test_dispatch_loop_converts_bytearray_payload" => {
            t.connect().await.unwrap();
            let payload = if b.contains("bytearray") {
                b"abc".to_vec()
            } else {
                br#"{"x": 1}"#.to_vec()
            };
            f.send(TOPIC, &payload);
            wait_for(|| !captured.lock().unwrap().is_empty()).await;
            assert_eq!(*captured.lock().unwrap(), vec![(TOPIC.into(), payload)]);
        }
        "test_dispatch_loop_routes_reply_before_handler" => {
            let (tx, rx) = oneshot::channel();
            t.pending.lock().unwrap().insert("rid".into(), tx);
            t.connect().await.unwrap();
            f.send(
                "/sys/pk/dn/app/down/account/bind_reply",
                br#"{"id":"rid","code":200}"#,
            );
            assert_eq!(rx.await.unwrap(), json!({"id":"rid","code":200}));
            assert!(captured.lock().unwrap().is_empty());
        }
        "test_dispatch_loop_swallows_handler_exception" => {
            let count = Arc::new(AtomicUsize::new(0));
            let n = count.clone();
            t = mqtt(
                &f,
                Arc::new(move |_, _| {
                    n.fetch_add(1, Ordering::SeqCst);
                    Box::pin(async { Err(Error::InvalidInput("handler failed")) })
                }),
            );
            t.connect().await.unwrap();
            f.send(TOPIC, b"{}");
            wait_for(|| count.load(Ordering::SeqCst) == 1).await;
            assert!(t.lost.try_recv().is_err());
        }
        "test_dispatch_loop_logs_and_exits_on_mqtt_error"
        | "test_dispatch_loop_signals_connection_lost_on_mqtt_error"
        | "test_dispatch_loop_signals_connection_lost_when_stream_ends" => {
            t.connect().await.unwrap();
            if b.contains("stream_ends") {
                f.end();
                assert!(matches!(t.lost.recv().await, Some(Error::Transport)));
            } else {
                f.loss(Error::Mqtt);
                assert!(matches!(t.lost.recv().await, Some(Error::Mqtt)));
            }
            assert!(t.lost.try_recv().is_err());
        }
        _ => panic!("unmapped {b}"),
    }
    t.disconnect().await;
}
async fn tls(b: &str) {
    let f = Fake::new();
    let (factory, _) = factory(vec![Ok(f.clone())]);
    let mut options = StreamOptions::default();
    if b.contains("prebuilt") {
        options = opts();
    }
    let prebuilt = options.tls_config.clone();
    let mut t = MqttTransport::new(
        "pk",
        "dn",
        "user",
        factory,
        options,
        Arc::new(|_, _| Box::pin(async { Ok(()) })),
    );
    match b {
        "test_build_tls_context_always_validates" => {
            let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
            let mut roots = rustls::RootCertStore::empty();
            roots.add(cert.cert.der().clone()).unwrap();
            let verify = crate::stream::verifier(roots).unwrap();
            let now = rustls::pki_types::UnixTime::now();
            let wrong = "wrong.example.com".try_into().unwrap();
            assert!(
                verify
                    .verify_server_cert(cert.cert.der(), &[], &wrong, &[], now)
                    .is_err()
            );
            let untrusted =
                crate::stream::verifier(crate::stream::root_store(None).unwrap()).unwrap();
            let right = "localhost".try_into().unwrap();
            assert!(
                untrusted
                    .verify_server_cert(cert.cert.der(), &[], &right, &[], now)
                    .is_err()
            );
        }
        "test_build_tls_context_trusts_bundled_broker_root"
        | "test_build_tls_context_keeps_broker_root_alongside_custom_bundle" => {
            let path =
                std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/broker-ca.pem");
            let custom = crate::stream::root_store(Some(path.clone())).unwrap();
            let roots = crate::stream::root_store(if b.contains("custom") {
                Some(path)
            } else {
                None
            })
            .unwrap();
            assert!(custom.roots.iter().all(|r| roots.roots.contains(r)));
            assert!(roots.roots.iter().any(|r| {
                r.subject
                    .as_ref()
                    .windows(b"GlobalSign Root CA".len())
                    .any(|w| w == b"GlobalSign Root CA")
            }));
            if b.contains("custom") {
                assert!(roots.roots.iter().all(|r| custom.roots.contains(r)));
            }
        }
        "test_init_does_not_build_context_eagerly" => assert!(t.tls.is_none()),
        "test_init_accepts_prebuilt_context" => assert!(Arc::ptr_eq(
            t.tls.as_ref().unwrap(),
            prebuilt.as_ref().unwrap()
        )),
        "test_connect_builds_context_in_executor" => {
            t.connect().await.unwrap();
            assert!(t.tls.is_some());
        }
        "test_connect_uses_prebuilt_context" => {
            t.connect().await.unwrap();
            assert!(Arc::ptr_eq(
                t.tls.as_ref().unwrap(),
                prebuilt.as_ref().unwrap()
            ));
        }
        _ => panic!("unmapped {b}"),
    }
    t.disconnect().await;
}
async fn status(b: &str) {
    if b.contains("handle_message")
        || b.contains("fire_typed")
        || b == "test_flatten_skips_non_mapping_entries"
    {
        dispatch(b);
        return;
    }
    if b.starts_with("test_reconnect_policy") {
        if b.contains("rejects") {
            for p in [
                ReconnectPolicy {
                    max_attempts: 0,
                    ..Default::default()
                },
                ReconnectPolicy {
                    multiplier: 0.5,
                    ..Default::default()
                },
            ] {
                assert!(p.validate().is_err());
            }
            assert!(Duration::try_from_secs_f64(-1.).is_err());
        } else {
            let p = ReconnectPolicy {
                initial_delay: Duration::from_secs(1),
                multiplier: 2.,
                max_delay: Duration::from_secs(5),
                ..Default::default()
            };
            assert_eq!(
                (1..5).map(|i| p.delay_for(i).as_secs()).collect::<Vec<_>>(),
                vec![1, 2, 4, 5]
            );
        }
        return;
    }
    let first = Fake::new();
    let second = Fake::new();
    let reconnect = b.contains("reconnect")
        || b.contains("second_drop")
        || b == "test_drop_while_stopping_is_not_retried";
    let mut sequence = vec![Ok(first.clone())];
    if b.contains("retries_until") {
        sequence.push(Err(Error::Transport));
    }
    if b.contains("gives_up") {
        sequence.extend([Err(Error::Transport), Err(Error::Transport)]);
    } else if reconnect || b.contains("restart") {
        sequence.push(Ok(second.clone()));
    }
    if b.contains("subscribe_fails") {
        *first.fail_sub.lock().unwrap() = true;
    }
    if b.contains("bind_fails") {
        *first.reply.lock().unwrap() = Some(500);
    }
    let (factory, count) = factory(sequence);
    let w = Wire::default();
    for _ in 0..4 {
        w.json(json!({"code":200,"data":{"productKey":"pk","deviceName":"dn","deviceSecret":"secret"}}));
    }
    let mut transport = w.transport();
    transport.mqtt_factory = Some(factory);
    let mut options = opts();
    if reconnect {
        options.reconnect = Some(ReconnectPolicy {
            max_attempts: 2,
            initial_delay: if b.contains("pending") {
                Duration::from_secs(30)
            } else {
                Duration::ZERO
            },
            max_delay: Duration::from_secs(30),
            ..Default::default()
        });
    }
    let mut s = StatusStream::new(transport, session("iot-token"), options).unwrap();
    match b {
        "test_start_connects_subscribes_binds" => {
            s.start().await.unwrap();
            s.start().await.unwrap();
            assert_eq!(count.load(Ordering::SeqCst), 1);
            let c = first.calls.lock().unwrap();
            assert_eq!(
                c[0],
                (
                    "subscribe".into(),
                    "/sys/pk/dn/app/down/#".into(),
                    vec![],
                    1
                )
            );
            assert_eq!(c[1].1, "/sys/pk/dn/app/up/account/bind");
            let body: Value = serde_json::from_slice(&c[1].2).unwrap();
            assert_eq!(body["params"]["iotToken"], "iot-token");
        }
        "test_stop_disconnects_and_is_idempotent" | "test_run_forever_returns_after_stop" => {
            s.start().await.unwrap();
            s.stop().await;
            s.stop().await;
            s.run_forever().await.unwrap();
            wait_for(|| first.live.load(Ordering::SeqCst) == 0).await;
            assert!(!s.is_running());
        }
        "test_start_disconnects_when_subscribe_fails"
        | "test_start_disconnects_when_bind_fails" => {
            assert!(s.start().await.is_err());
            assert!(!s.is_running());
            assert_eq!(first.live.load(Ordering::SeqCst), 0);
        }
        "test_run_forever_raises_when_connection_lost"
        | "test_run_forever_reraises_transport_error_unwrapped" => {
            s.start().await.unwrap();
            first.loss(if b.contains("unwrapped") {
                Error::Transport
            } else {
                Error::Mqtt
            });
            let e = s.run_forever().await.unwrap_err();
            if b.contains("unwrapped") {
                assert!(matches!(e, Error::Transport));
            } else {
                assert!(matches!(e, Error::Mqtt));
            }
            assert_eq!(count.load(Ordering::SeqCst), 1);
        }
        "test_restart_after_connection_loss_clears_the_error" => {
            s.start().await.unwrap();
            first.loss(Error::Mqtt);
            assert!(s.run_forever().await.is_err());
            s.stop().await;
            s.start().await.unwrap();
            s.stop().await;
            s.run_forever().await.unwrap();
            assert_eq!(count.load(Ordering::SeqCst), 2);
        }
        "test_context_manager_starts_and_stops" => {
            s.start().await.unwrap();
            assert_eq!(first.live.load(Ordering::SeqCst), 1);
            drop(s);
            wait_for(|| first.live.load(Ordering::SeqCst) == 0).await;
            return;
        }
        "test_reconnects_after_connection_loss"
        | "test_reconnect_retries_until_the_broker_comes_back"
        | "test_second_drop_does_not_stack_reconnect_tasks" => {
            s.start().await.unwrap();
            first.loss(Error::Mqtt);
            if b.contains("second_drop") {
                let _ = first.tx.send(Err(Error::Mqtt));
            }
            let expected = if b.contains("retries_until") { 3 } else { 2 };
            wait_for(|| {
                count.load(Ordering::SeqCst) == expected && second.calls.lock().unwrap().len() == 2
            })
            .await;
            assert_eq!(first.live.load(Ordering::SeqCst), 0);
            assert!(s.is_running());
            second.send(TOPIC, &push(json!({"silentMode":{"value":1}})));
            assert_eq!(
                s.next().await.unwrap().unwrap().changes["silent_mode"],
                true
            );
        }
        "test_reconnect_gives_up_and_notifies_the_consumer" => {
            s.start().await.unwrap();
            first.loss(Error::Mqtt);
            assert!(s.run_forever().await.is_err());
            assert_eq!(count.load(Ordering::SeqCst), 3);
            assert_eq!(first.live.load(Ordering::SeqCst), 0);
        }
        "test_stop_cancels_a_pending_reconnect" => {
            s.start().await.unwrap();
            first.loss(Error::Mqtt);
            wait_for(|| first.live.load(Ordering::SeqCst) == 0).await;
            assert!(s.is_running());
            s.stop().await;
            assert!(!s.is_running());
            s.run_forever().await.unwrap();
            assert_eq!(count.load(Ordering::SeqCst), 1);
        }
        "test_drop_while_stopping_is_not_retried" => {
            s.start().await.unwrap();
            s.stop().await;
            let _ = first.tx.send(Err(Error::Mqtt));
            tokio::task::yield_now().await;
            assert_eq!(count.load(Ordering::SeqCst), 1);
            assert!(!s.is_running());
        }
        _ => panic!("unmapped {b}"),
    }
    s.stop().await;
}
fn dispatch(b: &str) {
    let h = Handlers::default();
    let captured = Arc::new(Mutex::new(Vec::new()));
    let cap = captured.clone();
    h.on_change(move |u| {
        cap.lock().unwrap().push(u.clone());
        Ok(())
    });
    match b {
        "test_handle_message_ignores_non_property_topic" => {
            h.handle_message("/sys/pk/dn/app/down/event/other", &push(json!({})));
            assert!(captured.lock().unwrap().is_empty());
        }
        "test_handle_message_drops_unparsable_payload" => {
            h.handle_message(TOPIC, b"not json");
            assert!(captured.lock().unwrap().is_empty());
        }
        "test_handle_message_skips_empty_changes" => {
            h.handle_message(TOPIC, br#"{"params":{}}"#);
            h.handle_message(TOPIC, &push(json!({})));
            assert!(captured.lock().unwrap().is_empty());
        }
        "test_handle_message_dispatches_and_fires_on_change" => {
            let silent = Arc::new(Mutex::new(Vec::new()));
            let c = silent.clone();
            h.on_property("silent_mode", move |d, _, v| {
                c.lock()
                    .unwrap()
                    .push((d.to_string(), v.as_bool().unwrap()));
                Ok(())
            });
            h.handle_message(TOPIC, &push(json!({"silentMode":{"value":1}})));
            assert_eq!(*silent.lock().unwrap(), vec![("PB01".into(), true)]);
            assert_eq!(captured.lock().unwrap().len(), 1);
        }
        "test_handle_message_swallows_handler_exception" => {
            h.on_property("silent_mode", |_, _, _| {
                Err(Error::InvalidInput("handler failed"))
            });
            h.handle_message(TOPIC, &push(json!({"silentMode":{"value":1}})));
            assert_eq!(captured.lock().unwrap().len(), 1);
        }
        "test_fire_typed_bool_events" | "test_fire_typed_int_str_and_state_events" => {
            let sink = Arc::new(Mutex::new(serde_json::Map::new()));
            let (items, expected) = if b.contains("bool") {
                (
                    json!({"silentMode":{"value":1},"childLockOnOff":{"value":0},"autoLevel":{"value":1},"youngCatMode":{"value":1},"cleanCfg":{"value":{"active":1}},"catLeft":{"value":{"kitten":1,"stayTime":9,"needClean":1}},"room_of_bin":{"value":2}}),
                    json!({"silent_mode":true,"child_lock":false,"auto_level":true,"young_cat_mode":true,"cleaning_enabled":true,"cat_present":true,"needs_cleaning":true,"bucket_full":true}),
                )
            } else {
                (
                    json!({"Sand":{"value":{"percent":73}},"catLeft":{"value":{"kitten":0,"stayTime":12,"needClean":0}},"latestAddSandTime":{"value":"2026-05-31 10:00:00"},"actionLog":{"value":"cleanNow"},"bucketStatus":{"value":2}}),
                    json!({"sand_percent":73,"cat_stay_seconds":12,"last_sand_added":"2026-05-31 10:00:00","last_action":"cleanNow","operating_state":2}),
                )
            };
            for key in expected.as_object().unwrap().keys() {
                let c = sink.clone();
                h.on_property(key, move |d, k, v| {
                    assert_eq!(d, "PB01");
                    c.lock().unwrap().insert(k.into(), v.clone());
                    Ok(())
                });
            }
            h.handle_message(TOPIC, &push(items));
            assert_eq!(json!(*sink.lock().unwrap()), expected);
        }
        "test_fire_typed_unknown_goes_to_on_unknown" => {
            let unknown = Arc::new(Mutex::new(Vec::new()));
            let c = unknown.clone();
            h.on_unknown(move |d, k, v| {
                c.lock()
                    .unwrap()
                    .push((d.to_string(), k.to_string(), v.clone()));
                Ok(())
            });
            h.handle_message(TOPIC, &push(json!({"Reboot":{"value":1}})));
            assert_eq!(
                *unknown.lock().unwrap(),
                vec![("PB01".into(), "Reboot".into(), json!(1))]
            );
        }
        "test_flatten_skips_non_mapping_entries" => {
            let u=StatusUpdate::from_push(&json!({"params":{"deviceName":"PB01","items":{"silentMode":{"value":1},"junk":5}}})).unwrap();
            assert_eq!(json!(u.changes), json!({"silent_mode":true}));
        }
        "test_fire_typed_without_handlers_is_noop" => {
            let h = Handlers::default();
            h.handle_message(TOPIC,&push(json!({"silentMode":{"value":1},"Sand":{"value":{"percent":5}},"latestAddSandTime":{"value":"x"},"bucketStatus":{"value":0},"catLeft":{"value":{"kitten":1,"stayTime":1,"needClean":0}},"Reboot":{"value":1}})));
        }
        _ => panic!("unmapped {b}"),
    }
}
