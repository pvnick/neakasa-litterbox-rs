use crate::*;
use serde_json::{Value, json};
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    time::Duration,
};
type ResponseQueue = Arc<Mutex<VecDeque<crate::transport::WireResponse>>>;
#[derive(Clone, Default)]
pub struct Wire {
    pub requests: Arc<Mutex<Vec<reqwest::Request>>>,
    pub responses: ResponseQueue,
}
impl Wire {
    pub fn json(&self, v: Value) {
        self.responses
            .lock()
            .unwrap()
            .push_back(Ok((200, serde_json::to_vec(&v).unwrap())));
    }
    pub fn error(&self, e: crate::transport::IoFailure) {
        self.responses.lock().unwrap().push_back(Err(e));
    }
    pub fn transport(&self) -> transport::Transport {
        let mut t = transport::Transport::new(Duration::from_secs(1)).unwrap();
        self.attach(&mut t);
        t
    }
    pub fn attach(&self, t: &mut transport::Transport) {
        let w = self.clone();
        t.mock = Some(Arc::new(move |r| {
            w.requests.lock().unwrap().push(r);
            w.responses
                .lock()
                .unwrap()
                .pop_front()
                .expect("unexpected network request")
        }));
    }
    pub fn count(&self) -> usize {
        self.requests.lock().unwrap().len()
    }
    pub fn body(&self, n: usize) -> Value {
        serde_json::from_slice(
            self.requests.lock().unwrap()[n]
                .body()
                .unwrap()
                .as_bytes()
                .unwrap(),
        )
        .unwrap()
    }
    pub fn path(&self, n: usize) -> String {
        self.requests.lock().unwrap()[n].url().path().into()
    }
    pub fn query(&self, n: usize) -> std::collections::HashMap<String, String> {
        self.requests.lock().unwrap()[n]
            .url()
            .query_pairs()
            .into_owned()
            .collect()
    }
}
pub fn session(token: &str) -> LoginResult {
    LoginResult {
        user_id: "400068852".into(),
        user_token: "utoken".into(),
        aes_key: "0123456789abcdef".into(),
        aes_iv: "abcdef0123456789".into(),
        user_info: UserInfo {
            user_id: 42,
            user_name: "user@example.com".into(),
            ali_user_id: 400068852,
            ali_authentication_token: "ali-auth-token".into(),
        },
        issued_at: 1700000000.,
        iot_token: token.into(),
        iot_host: US_GATEWAY.into(),
    }
}
pub fn client(w: &Wire, authed: bool) -> NeakasaClient {
    let mut c = NeakasaClient::new("user@example.com", "pw", Region::Us).unwrap();
    w.attach(&mut c.transport);
    if authed {
        c.resume_session(session("iot-token"));
        c.register_device(Device::from_api(
            &json!({"deviceName":"PB01","iotId":"iot-1","productKey":"pk","owned":1}),
        ));
    }
    c
}
pub fn handshake() -> Vec<Value> {
    vec![
        json!({"code":200,"data":{"oaApiGatewayEndpoint":"oa.example.com","apiGatewayEndpoint":"api.example.com"}}),
        json!({"success":"true","data":{"successful":"true","vid":"vid-123"}}),
        json!({"success":"true","data":{"successful":"true","data":{"loginSuccessResult":{"sid":"sid-456"}}}}),
        json!({"code":200,"data":{"iotToken":"iot-token-789"}}),
    ]
}
pub fn rest_login() -> Value {
    json!({"code":0,"data":{"loginToken":"","userInfo":{"userId":42,"userName":"user@example.com","aliUserId":400068852,"aliAuthenticationToken":"ali-auth-token"}}})
}
pub fn base(name: &str) -> &str {
    name.split('[').next().unwrap()
}
pub fn param(name: &str) -> &str {
    name.split_once('[')
        .map(|(_, s)| s.trim_end_matches(']'))
        .unwrap_or("")
}
