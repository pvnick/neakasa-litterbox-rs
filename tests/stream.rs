use bytes::BytesMut;
use neakasa_litterbox_sdk::*;
use rumqttc::mqttbytes::{self, QoS, v4::*};
use serde_json::{Value, json};
use std::{sync::Arc, time::Duration};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use tokio_rustls::{TlsAcceptor, server::TlsStream};
use wiremock::{Mock, MockServer, ResponseTemplate, matchers::path};

struct Peer {
    socket: TlsStream<TcpStream>,
    buffer: BytesMut,
}
impl Peer {
    async fn read(&mut self) -> Packet {
        loop {
            match Packet::read(&mut self.buffer, 65536) {
                Ok(p) => return p,
                Err(mqttbytes::Error::InsufficientBytes(_)) => {
                    assert!(self.socket.read_buf(&mut self.buffer).await.unwrap() > 0);
                }
                Err(e) => panic!("invalid MQTT: {e:?}"),
            }
        }
    }
    async fn send(&mut self, p: Packet) {
        let mut b = BytesMut::new();
        p.write(&mut b, 65536).unwrap();
        self.socket.write_all(&b).await.unwrap();
    }
    async fn publish(&mut self, v: Value) {
        self.send(Packet::Publish(Publish::new(
            "/sys/pk/dn/app/down/test",
            QoS::AtMostOnce,
            serde_json::to_vec(&v).unwrap(),
        )))
        .await;
    }
    async fn bind_success(&mut self, id: &str) {
        let mut p = Publish::new(
            "/sys/pk/dn/app/down/account/bind",
            QoS::AtLeastOnce,
            serde_json::to_vec(&json!({"id":id,"code":200})).unwrap(),
        );
        p.pkid = 42;
        self.send(Packet::Publish(p)).await;
        loop {
            if let Packet::PubAck(ack) = self.read().await
                && ack.pkid == 42
            {
                break;
            }
        }
    }
    async fn wait_closed(&mut self) {
        let mut b = [0; 1024];
        loop {
            match self.socket.read(&mut b).await {
                Ok(0) | Err(_) => break,
                Ok(_) => {}
            }
        }
    }
}
struct Setup {
    server: MockServer,
    listener: TcpListener,
    acceptor: TlsAcceptor,
    client: NeakasaClient,
    options: StreamOptions,
}
async fn setup(reconnect: bool) -> Setup {
    let server = MockServer::start().await;
    Mock::given(path("/app/aepauth/handle"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"code":200,"data":{"productKey":"pk","deviceName":"dn","deviceSecret":"secret"}})))
        .mount(&server).await;
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let config = rustls::ServerConfig::builder_with_provider(provider.clone())
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_no_client_auth()
        .with_single_cert(
            vec![cert.cert.der().clone()],
            rustls::pki_types::PrivatePkcs8KeyDer::from(cert.key_pair.serialize_der()).into(),
        )
        .unwrap();
    let mut roots = rustls::RootCertStore::empty();
    roots.add(cert.cert.der().clone()).unwrap();
    let tls = rustls::ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_root_certificates(roots)
        .with_no_client_auth();
    let options = StreamOptions {
        broker_endpoint: Some(("localhost".into(), listener.local_addr().unwrap().port())),
        tls_config: Some(Arc::new(tls)),
        timeout: Duration::from_millis(500),
        reconnect: reconnect.then_some(ReconnectPolicy {
            max_attempts: 2,
            initial_delay: Duration::from_millis(1),
            ..Default::default()
        }),
        ..Default::default()
    };
    let fixture: Value = serde_json::from_str(include_str!("fixtures/android.json")).unwrap();
    let mut login = LoginResult::from_api(&fixture["user"]).unwrap();
    login.iot_host = server.uri();
    login.iot_token = "test-iot".into();
    let mut client = NeakasaClient::new("", "", Region::Us).unwrap();
    client.resume_session(login);
    Setup {
        server,
        listener,
        acceptor: TlsAcceptor::from(Arc::new(config)),
        client,
        options,
    }
}
async fn accept_bind(listener: &TcpListener, acceptor: &TlsAcceptor) -> (Peer, String) {
    let (socket, _) = listener.accept().await.unwrap();
    let mut peer = Peer {
        socket: acceptor.accept(socket).await.unwrap(),
        buffer: BytesMut::new(),
    };
    let Packet::Connect(c) = peer.read().await else {
        panic!("expected CONNECT")
    };
    assert_eq!(c.client_id, "dn&pk|securemode=2,signmethod=hmacsha1,ext=1|");
    let login = c.login.unwrap();
    assert_eq!(login.username, "dn&pk");
    assert_eq!(
        login.password,
        stream::mqtt_credentials("pk", "dn", "secret", "us-east-1.api-iot.aliyuncs.com").3
    );
    peer.send(Packet::ConnAck(ConnAck::new(
        ConnectReturnCode::Success,
        false,
    )))
    .await;
    let Packet::Subscribe(s) = peer.read().await else {
        panic!("expected SUBSCRIBE")
    };
    assert_eq!(s.filters[0].path, "/sys/pk/dn/app/down/#");
    peer.send(Packet::SubAck(SubAck::new(
        s.pkid,
        vec![SubscribeReasonCode::Success(QoS::AtLeastOnce)],
    )))
    .await;
    let Packet::Publish(p) = peer.read().await else {
        panic!("expected account bind")
    };
    assert_eq!(p.topic, "/sys/pk/dn/app/up/account/bind");
    let body: Value = serde_json::from_slice(&p.payload).unwrap();
    assert_eq!(body["params"]["iotToken"], "test-iot");
    peer.send(Packet::PubAck(PubAck::new(p.pkid))).await;
    (peer, body["id"].as_str().unwrap().into())
}
#[tokio::test]
async fn verified_tls_bind_updates_reconnect_and_idempotent_stop() {
    let Setup {
        server,
        listener,
        acceptor,
        client,
        options,
    } = setup(true).await;
    let broker = tokio::spawn(async move {
        for round in 1..=2 {
            let (mut peer, id) = accept_bind(&listener, &acceptor).await;
            // A push arriving before bind acknowledgement must not disappear.
            peer.publish(
                json!({"params":{"deviceName":"TEST01","items":{"autoLevel":{"value":round}}}}),
            )
            .await;
            // Ignore unrelated response IDs.
            peer.publish(json!({"id":"unrelated","code":401})).await;
            if round == 1 {
                peer.bind_success(&id).await;
            } else {
                peer.publish(json!({"id":id,"code":200,"data":{}})).await;
                // Stop/drop must close the socket.
                peer.wait_closed().await;
            }
        }
    });
    tokio::time::timeout(Duration::from_secs(5), async {
        let mut stream = client.watch_status(options).await.unwrap();
        for _ in 1..=2 {
            let update = stream.next().await.unwrap().unwrap();
            assert_eq!(update.device_name, "TEST01");
            assert!(!update.changes.is_empty());
        }
        stream.stop().await;
        stream.stop().await;
        assert!(stream.next().await.is_none());
        broker.await.unwrap();
    })
    .await
    .unwrap();
    assert_eq!(server.received_requests().await.unwrap().len(), 2);
}
#[tokio::test]
async fn account_bind_rejection_returns_redacted_session_error() {
    let Setup {
        server: _server,
        listener,
        acceptor,
        client,
        options,
    } = setup(false).await;
    let broker = tokio::spawn(async move {
        let (mut peer, id) = accept_bind(&listener, &acceptor).await;
        peer.publish(json!({"id":id,"code":401,"message":"sensitive"}))
            .await;
        peer.wait_closed().await;
    });
    assert!(matches!(
        client.watch_status(options).await,
        Err(Error::SessionExpired(401))
    ));
    broker.await.unwrap();
}
#[tokio::test]
async fn missing_bind_ack_times_out() {
    let Setup {
        server: _server,
        listener,
        acceptor,
        client,
        options,
    } = setup(false).await;
    let broker = tokio::spawn(async move {
        let (_peer, _) = accept_bind(&listener, &acceptor).await;
        std::future::pending::<()>().await;
    });
    assert!(matches!(
        client.watch_status(options).await,
        Err(Error::Timeout)
    ));
    broker.abort();
}
#[tokio::test]
async fn broker_disconnect_is_reported_when_reconnect_disabled() {
    let Setup {
        server: _server,
        listener,
        acceptor,
        client,
        options,
    } = setup(false).await;
    let broker = tokio::spawn(async move {
        let (mut peer, id) = accept_bind(&listener, &acceptor).await;
        peer.bind_success(&id).await;
    });
    let mut stream = client.watch_status(options).await.unwrap();
    assert!(matches!(
        tokio::time::timeout(Duration::from_secs(2), stream.next())
            .await
            .unwrap(),
        Some(Err(Error::Mqtt))
    ));
    assert!(stream.next().await.is_none());
    broker.await.unwrap();
}
