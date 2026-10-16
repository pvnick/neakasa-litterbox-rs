//! TLS-verified MQTT property updates with bounded exponential reconnection.
use crate::{
    Error, LoginResult, Result, StatusUpdate, auth, error::unwrap_iot, transport::Transport,
};

use serde_json::{Value, json};
use std::{path::PathBuf, sync::Arc, time::Duration};
use tokio::{sync::mpsc, task::JoinHandle};

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

/// A lazily constructed stream; `start` and `stop` are idempotent.
/// It supports pull consumption and optional callbacks. Dropping cancels its tasks.
pub struct StatusStream {
    receiver: mpsc::Receiver<Result<StatusUpdate>>,
    task: Option<JoinHandle<()>>,
    cancel: Option<tokio::sync::oneshot::Sender<()>>,
    transport: Transport,
    login: LoginResult,
    options: StreamOptions,
    handlers: Arc<Handlers>,
}
type ChangeHandler = Arc<dyn Fn(&StatusUpdate) -> Result<()> + Send + Sync>;
type PropertyHandler = Arc<dyn Fn(&str, &str, &Value) -> Result<()> + Send + Sync>;
#[derive(Default)]
pub struct Handlers {
    changes: std::sync::Mutex<Vec<ChangeHandler>>,
    properties: std::sync::Mutex<Vec<(Option<String>, PropertyHandler)>>,
}
impl Handlers {
    pub fn on_change(
        &self,
        callback: impl Fn(&StatusUpdate) -> Result<()> + Send + Sync + 'static,
    ) {
        self.changes.lock().unwrap().push(Arc::new(callback));
    }
    pub fn on_property(
        &self,
        key: &str,
        callback: impl Fn(&str, &str, &Value) -> Result<()> + Send + Sync + 'static,
    ) {
        self.properties
            .lock()
            .unwrap()
            .push((Some(key.into()), Arc::new(callback)));
    }
    pub fn on_unknown(
        &self,
        callback: impl Fn(&str, &str, &Value) -> Result<()> + Send + Sync + 'static,
    ) {
        self.properties
            .lock()
            .unwrap()
            .push((None, Arc::new(callback)));
    }
    pub fn dispatch(&self, update: &StatusUpdate) {
        let properties = self.properties.lock().unwrap().clone();
        let changes = self.changes.lock().unwrap().clone();
        for (key, value) in &update.changes {
            let known = matches!(
                key.as_str(),
                "silent_mode"
                    | "child_lock"
                    | "auto_level"
                    | "young_cat_mode"
                    | "cleaning_enabled"
                    | "cat_present"
                    | "needs_cleaning"
                    | "bucket_full"
                    | "sand_percent"
                    | "cat_stay_seconds"
                    | "last_sand_added"
                    | "last_action"
                    | "operating_state"
            );
            for (filter, cb) in properties.iter() {
                if (filter.as_deref() == Some(key) || (filter.is_none() && !known))
                    && cb(&update.device_name, key, value).is_err()
                {
                    log::warn!("status property handler failed");
                }
            }
        }
        for cb in changes.iter() {
            if cb(update).is_err() {
                log::warn!("status change handler failed");
            }
        }
    }
    pub fn handle_message(&self, topic: &str, payload: &[u8]) -> Option<StatusUpdate> {
        if !topic.ends_with("/thing/properties") {
            return None;
        }
        let value = serde_json::from_slice::<Value>(payload).ok()?;
        let update = StatusUpdate::from_push(&value)?;
        if update.changes.is_empty() {
            return None;
        }
        self.dispatch(&update);
        Some(update)
    }
}
impl StatusStream {
    pub(crate) fn new(
        transport: Transport,
        login: LoginResult,
        options: StreamOptions,
    ) -> Result<Self> {
        if login.iot_token.is_empty() {
            return Err(Error::NotAuthenticated);
        }
        if let Some(p) = &options.reconnect {
            p.validate()?;
        }
        let (_, receiver) = mpsc::channel(64);
        Ok(Self {
            transport,
            login,
            options,
            receiver,
            task: None,
            cancel: None,
            handlers: Arc::new(Handlers::default()),
        })
    }
    pub fn handlers(&self) -> Arc<Handlers> {
        self.handlers.clone()
    }
    pub fn is_running(&self) -> bool {
        self.task.as_ref().is_some_and(|t| !t.is_finished())
    }
    pub(crate) async fn connect(
        transport: Transport,
        login: LoginResult,
        options: StreamOptions,
    ) -> Result<Self> {
        let mut s = Self::new(transport, login, options)?;
        s.start().await?;
        Ok(s)
    }
    pub async fn start(&mut self) -> Result<()> {
        if self.is_running() {
            return Ok(());
        }
        let transport = self.transport.clone();
        let login = self.login.clone();
        let options = self.options.clone();
        let handlers = self.handlers.clone();
        let (sender, receiver) = mpsc::channel(64);
        let mut current = open(
            &transport,
            &login,
            &options,
            handlers.clone(),
            sender.clone(),
        )
        .await?;
        self.receiver = receiver;
        let (cancel, cancelled) = tokio::sync::oneshot::channel();
        self.cancel = Some(cancel);
        self.task = Some(tokio::spawn(async move {
            tokio::select! {
              _=cancelled=>{},
              _=async {
              loop {
                  let failure = current.lost.recv().await.unwrap_or(Error::Transport);
                  current.disconnect().await;
                  let Some(policy) = &options.reconnect else {
                      let _ = sender.send(Err(failure)).await;
                      return;
                  };
                  let mut replacement = None;
                  for attempt in 1..=policy.max_attempts {
                      tokio::time::sleep(policy.delay_for(attempt)).await;
                      if sender.is_closed() {
                          return;
                      }
                      if let Ok(next) = open(
                          &transport,
                          &login,
                          &options,
                          handlers.clone(),
                          sender.clone(),
                      )
                      .await
                      {
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
              }=>{},
            }
            current.disconnect().await;
        }));
        Ok(())
    }
    pub async fn next(&mut self) -> Option<Result<StatusUpdate>> {
        self.receiver.recv().await
    }
    pub async fn run_forever(&mut self) -> Result<()> {
        while let Some(result) = self.next().await {
            result?;
        }
        Ok(())
    }
    pub async fn stop(&mut self) {
        if let Some(cancel) = self.cancel.take() {
            let _ = cancel.send(());
        }
        if let Some(t) = self.task.take() {
            let _ = t.await;
        }
        self.receiver.close();
        while self.receiver.try_recv().is_ok() {}
    }
}
impl Drop for StatusStream {
    fn drop(&mut self) {
        if let Some(t) = &self.task {
            t.abort();
        }
    }
}
async fn open(
    transport: &Transport,
    login: &LoginResult,
    options: &StreamOptions,
    handlers: Arc<Handlers>,
    sender: mpsc::Sender<Result<StatusUpdate>>,
) -> Result<crate::mqtt::MqttTransport> {
    let creds = derive_mqtt_credentials(transport, &login.iot_host).await?;
    let factory = crate::mqtt::real_factory(creds.clone(), options.broker_endpoint.clone());
    #[cfg(test)]
    let factory = transport.mqtt_factory.clone().unwrap_or(factory);
    let handler: crate::mqtt::Handler = Arc::new(move |topic: String, payload: Vec<u8>| {
        let handlers = handlers.clone();
        let sender = sender.clone();
        Box::pin(async move {
            if let Some(update) = handlers.handle_message(&topic, &payload) {
                sender
                    .send(Ok(update))
                    .await
                    .map_err(|_| Error::Transport)?;
            }
            Ok(())
        })
    });
    let mut t = crate::mqtt::MqttTransport::new(
        &creds.product_key,
        &creds.device_name,
        &creds.username,
        factory,
        options.clone(),
        handler,
    );
    let result = async {
        t.connect().await?;
        t.subscribe(&format!("{}/app/down/#", t.topic_prefix), 1)
            .await?;
        t.bind_account(&login.iot_token, options.timeout).await
    }
    .await;
    if let Err(e) = result {
        t.disconnect().await;
        return Err(e);
    }
    Ok(t)
}
#[derive(Clone)]
pub struct MqttCredentials {
    pub host: String,
    pub port: u16,
    pub client_id: String,
    pub username: String,
    pub password: String,
    pub product_key: String,
    pub device_name: String,
}
pub fn random_auth_info(now_ms: u64) -> Value {
    let cid = uuid::Uuid::new_v4().simple().to_string();
    let sn = uuid::Uuid::new_v4().simple().to_string();
    auth::mqtt_auth_info(now_ms, &cid[..8], &sn)
}
pub async fn derive_mqtt_credentials(
    transport: &Transport,
    gateway: &str,
) -> Result<MqttCredentials> {
    let triple = unwrap_iot(
        transport
            .iot(
                gateway,
                "/app/aepauth/handle",
                "1.0.0",
                None,
                json!({"authInfo":random_auth_info(auth::now_ms())}),
                "en-US",
            )
            .await?,
    )?;
    let field = |key: &str| {
        triple[key]
            .as_str()
            .filter(|s| !s.is_empty())
            .ok_or(Error::Api(-1))
    };
    let pk = field("productKey")?;
    let dn = field("deviceName")?;
    let secret = field("deviceSecret")?;
    let (host, client_id, username, password) = mqtt_credentials(pk, dn, secret, gateway);
    Ok(MqttCredentials {
        host,
        port: 1883,
        client_id,
        username,
        password,
        product_key: pk.into(),
        device_name: dn.into(),
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
pub(crate) fn root_store(path: Option<PathBuf>) -> Result<rustls::RootCertStore> {
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
    Ok(roots)
}
pub(crate) fn verifier(
    roots: rustls::RootCertStore,
) -> Result<Arc<rustls::client::WebPkiServerVerifier>> {
    rustls::client::WebPkiServerVerifier::builder_with_provider(
        Arc::new(roots),
        Arc::new(rustls::crypto::ring::default_provider()),
    )
    .build()
    .map_err(|_| Error::Crypto)
}
pub(crate) fn build_tls(path: Option<PathBuf>) -> Result<Arc<rustls::ClientConfig>> {
    let verify = verifier(root_store(path)?)?;
    let config = rustls::ClientConfig::builder_with_provider(Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_safe_default_protocol_versions()
    .map_err(|_| Error::Crypto)?
    .with_webpki_verifier(verify)
    .with_no_client_auth();
    Ok(Arc::new(config))
}
