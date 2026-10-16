//! TLS-verified MQTT property updates with bounded exponential reconnection.
use crate::{
    Error, LoginResult, Result, StatusUpdate, auth, error::unwrap_iot, transport::Transport,
};
use rumqttc::{AsyncClient, Event, EventLoop, MqttOptions, Packet, QoS, TlsConfiguration};
use serde_json::{Value, json};
use std::{collections::VecDeque, path::PathBuf, sync::Arc, time::Duration};
use tokio::{sync::mpsc, task::JoinHandle};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct ReconnectPolicy {
    pub max_attempts: usize,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub multiplier: f64,
}
impl Default for ReconnectPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            multiplier: 2.0,
        }
    }
}
impl ReconnectPolicy {
    pub fn validate(&self) -> Result<()> {
        if self.max_attempts == 0 || !self.multiplier.is_finite() || self.multiplier < 1.0 {
            Err(Error::InvalidInput("invalid reconnect policy"))
        } else {
            Ok(())
        }
    }
    pub fn delay_for(&self, attempt: usize) -> Duration {
        let seconds = (self.initial_delay.as_secs_f64()
            * self.multiplier.powf(attempt.saturating_sub(1) as f64))
        .min(self.max_delay.as_secs_f64());
        Duration::from_secs_f64(if seconds.is_finite() {
            seconds
        } else {
            self.max_delay.as_secs_f64()
        })
    }
}
#[derive(Clone)]
pub struct StreamOptions {
    pub reconnect: Option<ReconnectPolicy>,
    pub ca_certs: Option<PathBuf>,
    pub tls_config: Option<Arc<rustls::ClientConfig>>,
    pub timeout: Duration,
    /// Optional custom broker address; TLS verification still applies.
    pub broker_endpoint: Option<(String, u16)>,
}
impl Default for StreamOptions {
    fn default() -> Self {
        Self {
            reconnect: Some(ReconnectPolicy::default()),
            ca_certs: None,
            tls_config: None,
            timeout: Duration::from_secs(20),
            broker_endpoint: None,
        }
    }
}

/// Pull-based Rust equivalent of Python's callback stream. Dropping it stops all tasks.
pub struct StatusStream {
    receiver: mpsc::Receiver<Result<StatusUpdate>>,
    task: Option<JoinHandle<()>>,
}
impl StatusStream {
    pub(crate) async fn connect(
        transport: Transport,
        login: LoginResult,
        options: StreamOptions,
    ) -> Result<Self> {
        if login.iot_token.is_empty() {
            return Err(Error::NotAuthenticated);
        }
        if let Some(policy) = &options.reconnect {
            policy.validate()?;
        }
        let tls = if let Some(config) = &options.tls_config {
            config.clone()
        } else {
            let path = options.ca_certs.clone();
            tokio::task::spawn_blocking(move || build_tls(path))
                .await
                .map_err(|_| Error::Mqtt)??
        };
        let initial = open(&transport, &login, &options, tls.clone()).await?;
        let (sender, receiver) = mpsc::channel(64);
        let task = tokio::spawn(async move {
            let mut current = initial;
            loop {
                let failure = loop {
                    if let Some(update) = current.pending.pop_front() {
                        if sender.send(Ok(update)).await.is_err() {
                            return;
                        }
                        continue;
                    }
                    match current.events.poll().await {
                        Ok(Event::Incoming(Packet::Publish(p))) => {
                            if let Ok(v) = serde_json::from_slice::<Value>(&p.payload)
                                && let Some(update) = StatusUpdate::from_push(&v)
                                && sender.send(Ok(update)).await.is_err()
                            {
                                return;
                            }
                        }
                        Ok(_) => {}
                        Err(_) => break Error::Mqtt,
                    }
                };
                let Some(policy) = &options.reconnect else {
                    let _ = sender.send(Err(failure)).await;
                    return;
                };
                // Drop the old socket before bootstrapping and binding a fresh session.
                drop(current);
                let mut replacement = None;
                for attempt in 1..=policy.max_attempts {
                    tokio::time::sleep(policy.delay_for(attempt)).await;
                    if sender.is_closed() {
                        return;
                    }
                    if let Ok(next) = open(&transport, &login, &options, tls.clone()).await {
                        replacement = Some(next);
                        break;
                    }
                }
                match replacement {
                    Some(next) => current = next,
                    None => {
                        let _ = sender.send(Err(Error::Mqtt)).await;
                        return;
                    }
                }
            }
        });
        Ok(Self {
            receiver,
            task: Some(task),
        })
    }
    pub async fn next(&mut self) -> Option<Result<StatusUpdate>> {
        self.receiver.recv().await
    }
    pub async fn stop(&mut self) {
        if let Some(task) = self.task.take() {
            task.abort();
            let _ = task.await;
        }
        self.receiver.close();
    }
}
impl Drop for StatusStream {
    fn drop(&mut self) {
        if let Some(task) = &self.task {
            task.abort();
        }
    }
}

struct Connection {
    events: EventLoop,
    _client: AsyncClient,
    pending: VecDeque<StatusUpdate>,
}
async fn open(
    transport: &Transport,
    login: &LoginResult,
    options: &StreamOptions,
    tls: Arc<rustls::ClientConfig>,
) -> Result<Connection> {
    let cid = Uuid::new_v4().simple().to_string();
    let sn = Uuid::new_v4().simple().to_string();
    let triple = unwrap_iot(
        transport
            .iot(
                &login.iot_host,
                "/app/aepauth/handle",
                "1.0.0",
                None,
                json!({"authInfo":auth::mqtt_auth_info(auth::now_ms(),&cid[..8],&sn)}),
                "en-US",
            )
            .await?,
    )?;
    let pk = triple["productKey"]
        .as_str()
        .filter(|s| !s.is_empty())
        .ok_or(Error::InvalidResponse)?;
    let dn = triple["deviceName"]
        .as_str()
        .filter(|s| !s.is_empty())
        .ok_or(Error::InvalidResponse)?;
    let secret = triple["deviceSecret"]
        .as_str()
        .filter(|s| !s.is_empty())
        .ok_or(Error::InvalidResponse)?;
    let (host, client_id, username, password) = mqtt_credentials(pk, dn, secret, &login.iot_host);
    let (host, port) = options.broker_endpoint.clone().unwrap_or((host, 1883));
    let mut mqtt = MqttOptions::new(client_id, host, port);
    mqtt.set_credentials(&username, password)
        .set_keep_alive(Duration::from_secs(60))
        .set_clean_session(true)
        .set_transport(rumqttc::Transport::tls_with_config(
            TlsConfiguration::Rustls(tls),
        ));
    let (client, mut events) = AsyncClient::new(mqtt, 8);
    let prefix = format!("/sys/{pk}/{dn}");
    client
        .subscribe(format!("{prefix}/app/down/#"), QoS::AtLeastOnce)
        .await
        .map_err(|_| Error::Mqtt)?;
    let request_id = Uuid::new_v4().simple().to_string();
    let bind = json!({"id":request_id,"system":{"version":"1.0","time":auth::now_ms().to_string()},"request":{"clientId":username},"params":{"iotToken":login.iot_token}});
    let mut pending = VecDeque::new();
    tokio::time::timeout(options.timeout, async {
        let mut sent = false;
        loop {
            match events.poll().await.map_err(|_| Error::Mqtt)? {
                Event::Incoming(Packet::SubAck(ack)) => {
                    if ack
                        .return_codes
                        .iter()
                        .any(|code| matches!(code, rumqttc::SubscribeReasonCode::Failure))
                    {
                        return Err(Error::Mqtt);
                    }
                    if !sent {
                        client
                            .publish(
                                format!("{prefix}/app/up/account/bind"),
                                QoS::AtLeastOnce,
                                false,
                                serde_json::to_vec(&bind)?,
                            )
                            .await
                            .map_err(|_| Error::Mqtt)?;
                        sent = true;
                    }
                }
                Event::Incoming(Packet::Publish(p)) => {
                    if let Ok(v) = serde_json::from_slice::<Value>(&p.payload) {
                        if v["id"].as_str() == Some(&request_id) {
                            unwrap_iot(v)?;
                            return Ok(());
                        }
                        if let Some(update) = StatusUpdate::from_push(&v)
                            && pending.len() < 64
                        {
                            pending.push_back(update);
                        }
                    }
                }
                _ => {}
            }
        }
    })
    .await
    .map_err(|_| Error::Timeout)??;
    Ok(Connection {
        events,
        _client: client,
        pending,
    })
}
pub fn mqtt_credentials(
    pk: &str,
    dn: &str,
    secret: &str,
    gateway: &str,
) -> (String, String, String, String) {
    let region = gateway
        .split(".api-iot.aliyuncs.com")
        .next()
        .filter(|s| !s.is_empty())
        .unwrap_or("us-east-1");
    let bare = format!("{dn}&{pk}");
    let canonical = format!("clientId{bare}deviceName{dn}productKey{pk}");
    let password =
        auth::hex(&auth::hmac_sha1(secret.as_bytes(), canonical.as_bytes())).to_uppercase();
    (
        format!("{pk}.iot-as-mqtt.{region}.aliyuncs.com"),
        format!("{bare}|securemode=2,signmethod=hmacsha1,ext=1|"),
        bare,
        password,
    )
}
fn build_tls(path: Option<PathBuf>) -> Result<Arc<rustls::ClientConfig>> {
    let mut roots = rustls::RootCertStore::empty();
    if let Some(path) = path {
        let bytes = std::fs::read(path)?;
        for cert in rustls_pemfile::certs(&mut bytes.as_slice()) {
            roots
                .add(cert.map_err(|_| Error::Crypto)?)
                .map_err(|_| Error::Crypto)?;
        }
    } else {
        for cert in rustls_native_certs::load_native_certs().certs {
            let _ = roots.add(cert);
        }
    }
    for cert in rustls_pemfile::certs(&mut include_bytes!("broker-ca.pem").as_slice()) {
        roots
            .add(cert.map_err(|_| Error::Crypto)?)
            .map_err(|_| Error::Crypto)?;
    }
    let config = rustls::ClientConfig::builder_with_provider(Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_safe_default_protocol_versions()
    .map_err(|_| Error::Crypto)?
    .with_root_certificates(roots)
    .with_no_client_auth();
    Ok(Arc::new(config))
}
