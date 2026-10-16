use crate::{
    Error, Result, auth,
    constants::{APP_KEY, USER_AGENT},
    error::{unwrap_iot, unwrap_rest},
    models::*,
    transport::Transport,
};
use serde_json::{Value, json};
use std::{collections::HashMap, time::Duration};

/// Async client. Login is explicit; a cached session can be resumed without network I/O.
pub struct NeakasaClient {
    pub(crate) transport: Transport,
    email: String,
    password: String,
    base_url: String,
    language: String,
    login: Option<LoginResult>,
    devices: HashMap<String, Device>,
    bootstrap_host: String,
}
impl NeakasaClient {
    pub fn new(
        email: impl Into<String>,
        password: impl Into<String>,
        region: Region,
    ) -> Result<Self> {
        Self::with_endpoint(email, password, region.web_url(), Duration::from_secs(20))
    }
    pub fn with_endpoint(
        email: impl Into<String>,
        password: impl Into<String>,
        base_url: &str,
        timeout: Duration,
    ) -> Result<Self> {
        crate::transport::validate_base_url(base_url)?;
        Ok(Self {
            transport: Transport::new(timeout)?,
            email: email.into(),
            password: password.into(),
            base_url: base_url.trim_end_matches('/').into(),
            language: "en".into(),
            login: None,
            devices: HashMap::new(),
            bootstrap_host: "cn-shanghai.api-iot.aliyuncs.com".into(),
        })
    }
    pub fn set_language(&mut self, language: impl Into<String>) {
        self.language = language.into();
    }
    /// Override bootstrap gateway for regional testing or a compatible gateway.
    pub fn set_bootstrap_host(&mut self, host: impl Into<String>) -> Result<()> {
        let host = host.into();
        crate::transport::gateway_url(&host)?;
        self.bootstrap_host = host;
        Ok(())
    }
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
    pub fn is_authenticated(&self) -> bool {
        self.login.is_some()
    }
    pub fn login_result(&self) -> Option<&LoginResult> {
        self.login.as_ref()
    }
    pub fn resume_session(&mut self, login: LoginResult) {
        self.login = Some(login);
    }
    pub fn register_device(&mut self, device: Device) {
        self.devices.insert(device.device_name.clone(), device);
    }
    pub(crate) fn require_session(&self) -> Result<&LoginResult> {
        self.login.as_ref().ok_or(Error::NotAuthenticated)
    }
    /// None forces a fresh login; Some resumes and bootstraps IoT only if missing.
    pub async fn login(&mut self, cached: Option<LoginResult>) -> Result<LoginResult> {
        self.login = cached;
        if self.login.is_none() {
            self.login_rest().await?;
        }
        if self.require_session()?.iot_token.is_empty() {
            self.authenticate_aliyun().await?;
        }
        Ok(self.require_session()?.clone())
    }
    async fn login_rest(&mut self) -> Result<()> {
        if self.email.is_empty() || self.password.is_empty() {
            return Err(Error::InvalidInput("saved credentials unavailable"));
        }
        let params = json!({"areaCode":"1","productId":"a123nCqsrQm3vEbt","userName":self.email,"phone":"","email":self.email,
        "password":auth::md5_double_hex(&self.password),"userAppVersion":"2.2.6","deviceNumber":"neakasa-litterbox-sdk",
        "deviceToken":"neakasa-litterbox-sdk","deviceType":"2","devSysVer":USER_AGENT});
        let result = self
            .transport
            .get(
                &format!("{}/login", self.base_url),
                &[("data".into(), serde_json::to_string(&params)?)],
                auth::rest_headers(&self.language, auth::now_ms() / 1000),
            )
            .await?;
        self.login = Some(LoginResult::from_api(&unwrap_rest(result)?)?);
        Ok(())
    }
    async fn authenticate_aliyun(&mut self) -> Result<()> {
        let auth_code = self
            .require_session()?
            .user_info
            .ali_authentication_token
            .clone();
        if auth_code.is_empty() {
            return Err(Error::InvalidInput("missing Aliyun authentication token"));
        }
        let language = format!("{}-US", self.language);
        let region = unwrap_iot(
            self.transport
                .iot(
                    &self.bootstrap_host,
                    "/living/account/region/get",
                    "1.0.2",
                    None,
                    json!({"authCode":auth_code,"type":"THIRD_AUTHCODE"}),
                    &language,
                )
                .await?,
        )?;
        let oa = nonempty(&region["oaApiGatewayEndpoint"])?;
        let gateway = nonempty(&region["apiGatewayEndpoint"])?;
        let connect=self.transport.oa(&oa,"/api/prd/connect.json",json!({"request":{"context":{"appKey":APP_KEY},"config":{"version":0,"lastModify":0},"device":{}}}),None).await?;
        check_oa(&connect)?;
        let vid = nonempty(&connect["data"]["vid"])?;
        let login=self.transport.oa(&oa,"/api/prd/loginbyoauth.json",json!({"loginByOauthRequest":{"authCode":auth_code,"oauthPlateform":23,"oauthAppKey":APP_KEY,"riskControlInfo":{}}}),Some(&vid)).await?;
        check_oa(&login)?;
        let sid = nonempty(&login["data"]["data"]["loginSuccessResult"]["sid"])?;
        let result = unwrap_iot(
            self.transport
                .iot(
                    &gateway,
                    "/account/createSessionByAuthCode",
                    "1.0.4",
                    None,
                    json!({"request":{"authCode":sid,"accountType":"OA_SESSION","appKey":APP_KEY}}),
                    &language,
                )
                .await?,
        )?;
        let token = nonempty(&result["iotToken"])?;
        let session = self.login.as_mut().ok_or(Error::NotAuthenticated)?;
        session.iot_token = token;
        session.iot_host = gateway;
        Ok(())
    }
    async fn refresh_iot(&mut self) -> Result<()> {
        if self.authenticate_aliyun().await.is_err() {
            self.login_rest().await?;
            self.authenticate_aliyun().await?;
        }
        Ok(())
    }
    async fn iot(&mut self, path: &str, version: &str, payload: Value) -> Result<Value> {
        for attempt in 0..2 {
            let session = self.require_session()?;
            if session.iot_token.is_empty() {
                return Err(Error::NotAuthenticated);
            }
            let result = unwrap_iot(
                self.transport
                    .iot(
                        &session.iot_host,
                        path,
                        version,
                        Some(&session.iot_token),
                        payload.clone(),
                        &format!("{}-US", self.language),
                    )
                    .await?,
            );
            if matches!(result, Err(Error::SessionExpired(_))) && attempt == 0 {
                self.refresh_iot().await?;
            } else {
                return result;
            }
        }
        Err(Error::SessionExpired(401))
    }
    async fn rest_get(&self, path: &str, mut params: Vec<(String, String)>) -> Result<Value> {
        let session = self.require_session()?;
        params.push(("user_id".into(), session.user_id.clone()));
        unwrap_rest(
            self.transport
                .get(
                    &format!("{}{path}", self.base_url),
                    &params,
                    auth::authenticated_headers(session, &self.language, auth::now_ms())?,
                )
                .await?,
        )
    }
    async fn rest_post(&self, path: &str, mut payload: Value) -> Result<Value> {
        let session = self.require_session()?;
        payload["user_id"] = json!(
            session
                .user_id
                .parse::<i64>()
                .map_err(|_| Error::InvalidInput("invalid session user ID"))?
        );
        unwrap_rest(
            self.transport
                .post_json(
                    &format!("{}{path}", self.base_url),
                    payload,
                    auth::authenticated_headers(session, &self.language, auth::now_ms())?,
                )
                .await?,
        )
    }
    pub async fn list_devices(&mut self) -> Result<Vec<Device>> {
        let data = self
            .iot("/uc/listBindingByAccount", "1.0.8", json!({}))
            .await?;
        let devices: Vec<_> = objects(&data["data"])?
            .iter()
            .map(|v| Device::from_api(v))
            .collect();
        self.devices = devices
            .iter()
            .cloned()
            .map(|d| (d.device_name.clone(), d))
            .collect();
        Ok(devices)
    }
    async fn resolve(&mut self, name: &str) -> Result<Device> {
        self.require_session()?;
        if !self.devices.contains_key(name) {
            self.list_devices().await?;
        }
        self.devices
            .get(name)
            .cloned()
            .ok_or(Error::InvalidInput("device not registered"))
    }
    async fn rest_params(&mut self, name: &str) -> Result<Vec<(String, String)>> {
        let device = self.resolve(name).await?;
        Ok(vec![
            ("device_name".into(), name.into()),
            ("bind_status".into(), i64::from(device.role).to_string()),
        ])
    }
    pub async fn list_cats(&mut self, name: &str) -> Result<Vec<Cat>> {
        let params = self.rest_params(name).await?;
        let value = self.rest_get("/catbox/cat/list", params).await?;
        Ok(objects(&value)?.iter().map(|v| Cat::from_api(v)).collect())
    }
    pub async fn get_toilet_records(
        &mut self,
        name: &str,
        start: i64,
        end: i64,
    ) -> Result<Vec<ToiletRecord>> {
        let mut params = self.rest_params(name).await?;
        params.extend([
            ("start_time".into(), start.to_string()),
            ("end_time".into(), end.to_string()),
        ]);
        let value = self.rest_get("/catbox/record", params).await?;
        Ok(objects(&value["record_list"])?
            .iter()
            .map(|v| ToiletRecord::from_api(v))
            .collect())
    }
    pub async fn get_toilet_statistics(
        &mut self,
        name: &str,
        start: i64,
        end: i64,
        zone_seconds: i32,
    ) -> Result<Vec<DailyStatistics>> {
        let mut params = self.rest_params(name).await?;
        params.extend([
            ("start_time".into(), start.to_string()),
            ("end_time".into(), end.to_string()),
            ("zone".into(), zone_seconds.to_string()),
        ]);
        let value = self.rest_get("/catbox/toilet/statistics", params).await?;
        Ok(objects(&value)?
            .iter()
            .map(|v| DailyStatistics::from_api(v))
            .collect())
    }
    /// Explicit unit; never replays an ambiguous mutation or overwrites other profile fields.
    pub async fn set_cat_weight(
        &mut self,
        name: &str,
        cat_id: i64,
        weight: f64,
        unit: WeightUnit,
        zone_seconds: i32,
    ) -> Result<()> {
        if cat_id <= 0 {
            return Err(Error::InvalidInput("cat_id must be positive"));
        }
        if !weight.is_finite() || weight <= 0.0 {
            return Err(Error::InvalidInput("weight must be finite and positive"));
        }
        let device = self.resolve(name).await?;
        self.rest_post("/catbox/cat/update",json!({"device_name":name,"bind_status":device.role,"id":cat_id,"weight":weight,"unit":unit,"scene":"catDetail","zone":zone_seconds})).await?;
        Ok(())
    }
    pub async fn get_properties(&mut self, name: &str) -> Result<Value> {
        let device = self.resolve(name).await?;
        let v = self
            .iot(
                "/thing/properties/get",
                "1.0.4",
                json!({"iotId":device.iot_id}),
            )
            .await?;
        if !v.is_object() {
            return Err(Error::InvalidResponse);
        }
        Ok(v)
    }
    pub async fn get_status(&mut self, name: &str) -> Result<DeviceStatus> {
        Ok(DeviceStatus::from_response(
            &self.get_properties(name).await?,
        ))
    }
    async fn service(&mut self, name: &str, identifier: &str, args: Value) -> Result<()> {
        let device = self.resolve(name).await?;
        self.iot(
            "/thing/service/invoke",
            "1.0.5",
            json!({"iotId":device.iot_id,"identifier":identifier,"args":args}),
        )
        .await?;
        Ok(())
    }
    async fn property(&mut self, name: &str, key: &str, value: Value) -> Result<()> {
        let device = self.resolve(name).await?;
        self.iot(
            "/thing/properties/set",
            "1.0.5",
            json!({"iotId":device.iot_id,"items":{key:value}}),
        )
        .await?;
        Ok(())
    }
    pub async fn start_clean(&mut self, name: &str) -> Result<()> {
        self.service(name, "cleanNow", json!({"bStartClean":1}))
            .await
    }
    pub async fn stop_clean(&mut self, name: &str) -> Result<()> {
        self.service(name, "cleanNow", json!({"bStartClean":0}))
            .await
    }
    pub async fn start_level(&mut self, name: &str) -> Result<()> {
        self.service(name, "sandLeveling", json!({"bStartLeveling":1}))
            .await
    }
    pub async fn stop_level(&mut self, name: &str) -> Result<()> {
        self.service(name, "sandLeveling", json!({"bStartLeveling":0}))
            .await
    }
    pub async fn set_auto_clean(&mut self, name: &str, enabled: bool) -> Result<()> {
        let p = self.get_properties(name).await?;
        let mut config = p["cleanCfg"]["value"].clone();
        if !config.is_object()
            || !config["cleanType"].is_number()
            || !config["cleanParam"].is_number()
        {
            return Err(Error::InvalidResponse);
        }
        config["active"] = json!(i32::from(enabled));
        self.property(name, "cleanCfg", config).await
    }
    pub async fn set_auto_level(&mut self, name: &str, enabled: bool) -> Result<()> {
        self.property(name, "autoLevel", json!(i32::from(enabled)))
            .await
    }
    pub async fn set_silent_mode(&mut self, name: &str, enabled: bool) -> Result<()> {
        self.property(name, "silentMode", json!(i32::from(enabled)))
            .await
    }
    pub async fn set_child_lock(&mut self, name: &str, enabled: bool) -> Result<()> {
        self.property(name, "childLockOnOff", json!(i32::from(enabled)))
            .await
    }
    pub async fn calibrate_sand(&mut self, name: &str, percent: u8) -> Result<()> {
        if !(1..=100).contains(&percent) {
            return Err(Error::InvalidInput("percent must be in 1..100"));
        }
        self.service(name, "sandAdj", json!({"percent":percent}))
            .await
    }
    pub async fn watch_status(&self, options: crate::StreamOptions) -> Result<crate::StatusStream> {
        crate::StatusStream::connect(
            self.transport.clone(),
            self.require_session()?.clone(),
            options,
        )
        .await
    }
}
fn nonempty(v: &Value) -> Result<String> {
    v.as_str()
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .ok_or(Error::InvalidResponse)
}
fn check_oa(v: &Value) -> Result<()> {
    if v["success"].as_str() != Some("true") || v["data"]["successful"].as_str() != Some("true") {
        Err(Error::Authentication(-1))
    } else {
        Ok(())
    }
}
fn objects(v: &Value) -> Result<Vec<&Value>> {
    Ok(v.as_array()
        .ok_or(Error::InvalidResponse)?
        .iter()
        .filter(|v| v.is_object())
        .collect())
}
