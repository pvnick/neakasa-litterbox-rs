use crate::{
    Error, Result,
    auth::{self, Headers},
};
use reqwest::{Client, RequestBuilder};
use serde_json::Value;
use std::time::{Duration, SystemTime};
use uuid::Uuid;

#[derive(Clone)]
pub struct Transport {
    http: Client,
}
impl Transport {
    pub fn new(timeout: Duration) -> Result<Self> {
        let http = Client::builder()
            .timeout(timeout)
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|_| Error::Transport)?;
        Ok(Self { http })
    }
    fn headers(mut request: RequestBuilder, headers: Headers) -> RequestBuilder {
        for (name, value) in headers {
            request = request.header(name, value);
        }
        request
    }
    async fn execute(request: RequestBuilder) -> Result<Value> {
        let response = request.send().await.map_err(|_| Error::Transport)?;
        let status = response.status();
        if !status.is_success() {
            return Err(Error::Http(status.as_u16()));
        }
        let value: Value = response.json().await.map_err(|_| Error::InvalidResponse)?;
        if !value.is_object() {
            return Err(Error::InvalidResponse);
        }
        Ok(value)
    }
    pub async fn get(
        &self,
        url: &str,
        params: &[(String, String)],
        headers: Headers,
    ) -> Result<Value> {
        Self::execute(Self::headers(self.http.get(url).query(params), headers)).await
    }
    pub async fn post_json(&self, url: &str, body: Value, mut headers: Headers) -> Result<Value> {
        headers.insert("Content-Type".into(), "application/json".into());
        Self::execute(Self::headers(self.http.post(url), headers).json(&body)).await
    }
    pub async fn iot(
        &self,
        host: &str,
        path: &str,
        version: &str,
        token: Option<&str>,
        payload: Value,
        language: &str,
    ) -> Result<Value> {
        let id = Uuid::new_v4().to_string();
        let body = serde_json::to_vec(&auth::iot_envelope(payload, version, token, language, &id))?;
        let path = format!("{path}?x-ca-request-id={id}");
        let headers = auth::iot_headers(
            &path,
            &body,
            auth::now_ms(),
            &Uuid::new_v4().to_string(),
            &httpdate::fmt_http_date(SystemTime::now()),
        );
        let base = gateway_url(host)?;
        Self::execute(Self::headers(
            self.http.post(format!("{base}{path}")).body(body),
            headers,
        ))
        .await
    }
    pub async fn oa(
        &self,
        host: &str,
        path: &str,
        body: Value,
        vid: Option<&str>,
    ) -> Result<Value> {
        let (canonical, wire) = auth::oa_body(&body)?;
        let mut headers = auth::oa_headers(
            path,
            &canonical,
            auth::now_ms() / 1000,
            &Uuid::new_v4().to_string(),
            &httpdate::fmt_http_date(SystemTime::now()),
        );
        if let Some(vid) = vid {
            headers.insert("vid".into(), vid.into());
        }
        Self::execute(Self::headers(
            self.http
                .post(format!("{}{path}", gateway_url(host)?))
                .body(wire),
            headers,
        ))
        .await
    }
}
/// Real gateways always use HTTPS. Loopback HTTP is permitted for offline tests.
pub(crate) fn gateway_url(host: &str) -> Result<String> {
    let url = if host.contains("://") {
        host.to_string()
    } else {
        format!("https://{host}")
    };
    validate_base_url(&url)?;
    Ok(url.trim_end_matches('/').into())
}
pub(crate) fn validate_base_url(url: &str) -> Result<()> {
    let u = url::Url::parse(url).map_err(|_| Error::InvalidInput("invalid endpoint"))?;
    if !u.username().is_empty()
        || u.password().is_some()
        || u.query().is_some()
        || u.fragment().is_some()
    {
        return Err(Error::InvalidInput("invalid endpoint"));
    }
    if u.scheme() == "https"
        || (u.scheme() == "http"
            && matches!(u.host_str(), Some("localhost" | "127.0.0.1" | "[::1]")))
    {
        Ok(())
    } else {
        Err(Error::InvalidInput("endpoint requires HTTPS"))
    }
}
