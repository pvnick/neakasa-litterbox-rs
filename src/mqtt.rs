//! MQTT transport with explicit lifecycle, reply correlation, and an injectable I/O boundary.
use crate::{Error, Result, auth, error::unwrap_iot};
use rumqttc::{AsyncClient, Event, Packet, QoS};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{
    sync::{mpsc, oneshot},
    task::JoinHandle,
};
pub type FutureResult<'a, T> = Pin<Box<dyn Future<Output = Result<T>> + Send + 'a>>;
pub trait Sink: Send + Sync {
    fn subscribe<'a>(&'a self, topic: &'a str, qos: u8) -> FutureResult<'a, ()>;
    fn publish<'a>(&'a self, topic: &'a str, payload: Vec<u8>, qos: u8) -> FutureResult<'a, ()>;
}
pub trait Events: Send {
    fn next(&mut self) -> FutureResult<'_, Option<(String, Vec<u8>)>>;
}
pub type Connection = (Arc<dyn Sink>, Box<dyn Events>);
pub type Factory =
    Arc<dyn Fn(Arc<rustls::ClientConfig>) -> FutureResult<'static, Connection> + Send + Sync>;
type Pending = Arc<Mutex<HashMap<String, oneshot::Sender<Value>>>>;
pub type Handler = Arc<dyn Fn(String, Vec<u8>) -> FutureResult<'static, ()> + Send + Sync>;
pub struct MqttTransport {
    pub topic_prefix: String,
    username: String,
    factory: Factory,
    options: crate::StreamOptions,
    pub(crate) tls: Option<Arc<rustls::ClientConfig>>,
    sink: Option<Arc<dyn Sink>>,
    task: Option<JoinHandle<()>>,
    pub(crate) pending: Pending,
    pub(crate) lost: mpsc::UnboundedReceiver<Error>,
    lost_sender: mpsc::UnboundedSender<Error>,
    handler: Handler,
}
impl MqttTransport {
    pub fn new(
        pk: &str,
        dn: &str,
        username: &str,
        factory: Factory,
        options: crate::StreamOptions,
        handler: Handler,
    ) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            topic_prefix: format!("/sys/{pk}/{dn}"),
            username: username.into(),
            factory,
            tls: options.tls_config.clone(),
            options,
            sink: None,
            task: None,
            pending: Default::default(),
            lost: rx,
            lost_sender: tx,
            handler,
        }
    }
    pub fn is_connected(&self) -> bool {
        self.sink.is_some()
    }
    pub async fn connect(&mut self) -> Result<()> {
        if self.is_connected() {
            return Ok(());
        }
        let tls = match &self.tls {
            Some(t) => t.clone(),
            None => {
                let path = self.options.ca_certs.clone();
                let t = tokio::task::spawn_blocking(move || crate::stream::build_tls(path))
                    .await
                    .map_err(|_| Error::Mqtt)??;
                self.tls = Some(t.clone());
                t
            }
        };
        let (sink, mut events) = (self.factory)(tls).await?;
        let pending = self.pending.clone();
        let handler = self.handler.clone();
        let lost = self.lost_sender.clone();
        self.task = Some(tokio::spawn(async move {
            let error = loop {
                match events.next().await {
                    Ok(Some((topic, payload))) => {
                        if !dispatch_reply(&pending, &payload)
                            && handler(topic, payload).await.is_err()
                        {
                            log::warn!("MQTT message handler failed");
                        }
                    }
                    Ok(None) => break Error::Transport,
                    Err(e) => break e,
                }
            };
            log::warn!("MQTT connection lost");
            let _ = lost.send(error);
        }));
        self.sink = Some(sink);
        Ok(())
    }
    pub(crate) fn require_client(&self) -> Result<&Arc<dyn Sink>> {
        self.sink.as_ref().ok_or(Error::Transport)
    }
    pub async fn subscribe(&self, topic: &str, qos: u8) -> Result<()> {
        self.require_client()?.subscribe(topic, qos).await
    }
    pub async fn publish(&self, topic: &str, payload: Vec<u8>, qos: u8) -> Result<()> {
        self.require_client()?.publish(topic, payload, qos).await
    }
    pub async fn bind_account(&mut self, token: &str, timeout: Duration) -> Result<()> {
        let id = uuid::Uuid::new_v4().simple().to_string();
        let (tx, rx) = oneshot::channel();
        self.pending.lock().unwrap().insert(id.clone(), tx);
        // Removal is guaranteed even when the caller cancels the bind future.
        struct Remove(Pending, String);
        impl Drop for Remove {
            fn drop(&mut self) {
                self.0.lock().unwrap().remove(&self.1);
            }
        }
        let _remove = Remove(self.pending.clone(), id.clone());
        let body = json!({"id":id,"system":{"version":"1.0","time":auth::now_ms().to_string()},"request":{"clientId":self.username},"params":{"iotToken":token}});
        self.publish(
            &format!("{}/app/up/account/bind", self.topic_prefix),
            serde_json::to_vec(&body)?,
            1,
        )
        .await?;
        tokio::select! {
         result=tokio::time::timeout(timeout,rx)=>{let reply=result.map_err(|_|Error::Timeout)?.map_err(|_|Error::Transport)?;unwrap_iot(reply)?;Ok(())},
         error=self.lost.recv()=>Err(error.unwrap_or(Error::Transport)),
        }
    }
    pub fn try_dispatch_reply(&self, payload: &[u8]) -> bool {
        dispatch_reply(&self.pending, payload)
    }
    pub async fn disconnect(&mut self) {
        if let Some(t) = self.task.take() {
            t.abort();
            let _ = t.await;
        }
        self.sink = None;
        self.pending.lock().unwrap().clear();
    }
}
impl Drop for MqttTransport {
    fn drop(&mut self) {
        if let Some(t) = &self.task {
            t.abort();
        }
    }
}
fn dispatch_reply(pending: &Pending, payload: &[u8]) -> bool {
    let Ok(v) = serde_json::from_slice::<Value>(payload) else {
        return false;
    };
    let Some(id) = v["id"].as_str() else {
        return false;
    };
    let sender = pending.lock().unwrap().remove(id);
    sender.is_some_and(|s| s.send(v).is_ok())
}
fn qos(n: u8) -> Result<QoS> {
    match n {
        0 => Ok(QoS::AtMostOnce),
        1 => Ok(QoS::AtLeastOnce),
        2 => Ok(QoS::ExactlyOnce),
        _ => Err(Error::InvalidInput("invalid QoS")),
    }
}
struct RealSink(AsyncClient);
impl Sink for RealSink {
    fn subscribe<'a>(&'a self, topic: &'a str, n: u8) -> FutureResult<'a, ()> {
        Box::pin(async move {
            self.0
                .subscribe(topic, qos(n)?)
                .await
                .map_err(|_| Error::Mqtt)
        })
    }
    fn publish<'a>(&'a self, topic: &'a str, payload: Vec<u8>, n: u8) -> FutureResult<'a, ()> {
        Box::pin(async move {
            self.0
                .publish(topic, qos(n)?, false, payload)
                .await
                .map_err(|_| Error::Mqtt)
        })
    }
}
struct RealEvents(rumqttc::EventLoop);
impl Events for RealEvents {
    fn next(&mut self) -> FutureResult<'_, Option<(String, Vec<u8>)>> {
        Box::pin(async move {
            loop {
                match self.0.poll().await.map_err(|_| Error::Mqtt)? {
                    Event::Incoming(Packet::Publish(p)) => {
                        return Ok(Some((p.topic, p.payload.to_vec())));
                    }
                    Event::Incoming(Packet::SubAck(a))
                        if a.return_codes
                            .iter()
                            .any(|c| matches!(c, rumqttc::SubscribeReasonCode::Failure)) =>
                    {
                        return Err(Error::Mqtt);
                    }
                    _ => {}
                }
            }
        })
    }
}
pub(crate) fn real_factory(
    credentials: crate::stream::MqttCredentials,
    endpoint: Option<(String, u16)>,
) -> Factory {
    Arc::new(move |tls| {
        let c = credentials.clone();
        let ep = endpoint.clone();
        Box::pin(async move {
            let (host, port) = ep.unwrap_or((c.host, c.port));
            let mut o = rumqttc::MqttOptions::new(c.client_id, host, port);
            o.set_credentials(c.username, c.password)
                .set_keep_alive(Duration::from_secs(60))
                .set_clean_session(true)
                .set_transport(rumqttc::Transport::tls_with_config(
                    rumqttc::TlsConfiguration::Rustls(tls),
                ));
            let (sink, events) = AsyncClient::new(o, 8);
            Ok((
                Arc::new(RealSink(sink)) as Arc<dyn Sink>,
                Box::new(RealEvents(events)) as Box<dyn Events>,
            ))
        })
    })
}
