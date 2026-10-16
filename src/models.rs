use crate::{Error, Result, auth};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub const US_GATEWAY: &str = "us-east-1.api-iot.aliyuncs.com";
fn default_gateway() -> String {
    US_GATEWAY.into()
}
pub(crate) fn int(v: &Value) -> i64 {
    v.as_i64()
        .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
        .unwrap_or(0)
}
pub(crate) fn float(v: &Value) -> f64 {
    v.as_f64()
        .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
        .filter(|n| n.is_finite())
        .unwrap_or(0.0)
}
pub(crate) fn string(v: &Value) -> String {
    v.as_str().unwrap_or("").into()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Region {
    Us,
    Eu,
    Ap,
}
impl Region {
    pub fn web_url(self) -> &'static str {
        match self {
            Self::Us => "https://usapi.neakasapet.com/api",
            Self::Eu => "https://euapi.neakasa.com/api",
            Self::Ap => "https://ap.neakasa.com/api",
        }
    }
}

macro_rules! numeric_enum {
    ($name:ident, $default:ident, $( $variant:ident = $number:expr ),+ $(,)?) => {
        #[derive(Clone,Copy,Debug,Serialize,Deserialize,PartialEq,Eq)]
        #[serde(from="i64",into="i64")]
        pub enum $name { $( $variant = $number ),+ }
        impl From<i64> for $name { fn from(n:i64)->Self { match n { $( $number => Self::$variant, )+ _=>Self::$default } } }
        impl From<$name> for i64 { fn from(v:$name)->Self { v as i64 } }
    }
}
numeric_enum!(DeviceRole, Shared, Owner = 1, Shared = 2);
numeric_enum!(CatGender, Unknown, Unknown = 0, Male = 1, Female = 2);
numeric_enum!(RecordType, Other, CatVisit = 1, CleanCycle = 2, Other = 3);
numeric_enum!(
    OperatingState,
    Unknown,
    Unknown = -1,
    Idle = 0,
    Restoring = 1,
    Cleaning = 2,
    Leveling = 3,
    CatAppears = 5
);
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum WeightUnit {
    #[serde(rename = "kg")]
    Kg,
    #[serde(rename = "lb")]
    Lb,
}
impl WeightUnit {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Kg => "kg",
            Self::Lb => "lb",
        }
    }
}

/// Contains authentication material; Debug is intentionally redacted.
#[derive(Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub user_id: i64,
    pub user_name: String,
    pub ali_user_id: i64,
    pub ali_authentication_token: String,
}
impl std::fmt::Debug for UserInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("UserInfo([redacted])")
    }
}
impl UserInfo {
    pub fn from_api(v: &Value) -> Self {
        Self {
            user_id: int(&v["userId"]),
            user_name: string(&v["userName"]),
            ali_user_id: int(&v["aliUserId"]),
            ali_authentication_token: string(&v["aliAuthenticationToken"]),
        }
    }
}
#[derive(Clone, Serialize, Deserialize)]
pub struct LoginResult {
    pub user_id: String,
    pub user_token: String,
    pub aes_key: String,
    pub aes_iv: String,
    pub user_info: UserInfo,
    pub issued_at: f64,
    #[serde(default)]
    pub iot_token: String,
    #[serde(default = "default_gateway")]
    pub iot_host: String,
}
impl std::fmt::Debug for LoginResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("LoginResult([redacted])")
    }
}
impl LoginResult {
    pub fn from_api(v: &Value) -> Result<Self> {
        let plain =
            auth::decrypt_login_token(v["loginToken"].as_str().ok_or(Error::InvalidResponse)?)?;
        let parts: Vec<_> = plain.split('@').collect();
        // The live service appends metadata after the four authentication fields.
        // Match Python's parser by ignoring those trailing fields.
        if parts.len() < 4 {
            return Err(Error::InvalidResponse);
        }
        let user_info = UserInfo::from_api(&v["userInfo"]);
        Ok(Self {
            user_id: if user_info.ali_user_id != 0 {
                user_info.ali_user_id.to_string()
            } else {
                parts[1].into()
            },
            user_token: parts[0].into(),
            aes_key: parts[2].into(),
            aes_iv: parts[3].into(),
            user_info,
            issued_at: auth::now_ms() as f64 / 1000.0,
            iot_token: String::new(),
            iot_host: default_gateway(),
        })
    }
    pub fn with_iot_session(mut self, token: String, host: String) -> Self {
        self.iot_token = token;
        self.iot_host = host;
        self
    }
    pub fn age_seconds(&self) -> f64 {
        auth::now_ms() as f64 / 1000.0 - self.issued_at
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Device {
    pub iot_id: String,
    pub product_key: String,
    pub product_name: String,
    pub device_name: String,
    pub category_key: String,
    pub category_name: String,
    pub net_type: String,
    pub role: DeviceRole,
    pub status: i64,
    pub bind_time: i64,
}
impl Device {
    pub fn from_api(v: &Value) -> Self {
        Self {
            iot_id: string(&v["iotId"]),
            product_key: string(&v["productKey"]),
            product_name: string(&v["productName"]),
            device_name: string(&v["deviceName"]),
            category_key: string(&v["categoryKey"]),
            category_name: string(&v["categoryName"]),
            net_type: string(&v["netType"]),
            role: if int(&v["owned"]) == 1 {
                DeviceRole::Owner
            } else {
                DeviceRole::Shared
            },
            status: int(&v["status"]),
            bind_time: int(&v["bindTime"]),
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Cat {
    pub id: i64,
    pub name: String,
    pub weight: f64,
    pub unit: String,
    pub avatar: String,
    pub birthday: String,
    pub variety: i64,
    pub gender: CatGender,
    pub sterilization: i64,
    pub enabled: i64,
    pub path: String,
}
impl Cat {
    pub fn from_api(v: &Value) -> Self {
        Self {
            id: int(&v["id"]),
            name: string(&v["name"]),
            weight: float(&v["weight"]),
            unit: string(&v["unit"]),
            avatar: string(&v["avatar"]),
            birthday: string(&v["birthday"]),
            variety: if v["variety"].is_null() {
                -1
            } else {
                int(&v["variety"])
            },
            gender: CatGender::from(int(&v["gender"])),
            sterilization: int(&v["sterilization"]),
            enabled: int(&v["enabled"]),
            path: string(&v["path"]),
        }
    }
    pub fn is_sterilized(&self) -> bool {
        self.sterilization == 1
    }
    pub fn is_enabled(&self) -> bool {
        self.enabled == 1
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ToiletRecord {
    pub record_id: i64,
    pub record_type: RecordType,
    pub cat_id: i64,
    pub start_time: i64,
    pub end_time: i64,
    pub weight: f64,
    pub unit: String,
    pub way: i64,
}
impl ToiletRecord {
    pub fn from_api(v: &Value) -> Self {
        Self {
            record_id: int(&v["record_id"]),
            record_type: RecordType::from(int(&v["type"])),
            cat_id: int(&v["cat_id"]),
            start_time: int(&v["start_time"]),
            end_time: int(&v["end_time"]),
            weight: float(&v["weight"]),
            unit: string(&v["unit"]),
            way: int(&v["way"]),
        }
    }
    pub fn duration_seconds(&self) -> i64 {
        self.end_time - self.start_time
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DailyStatistics {
    pub date: String,
    pub num: i64,
    pub weight: f64,
    pub unit: String,
    pub toilet_total_second: i64,
    pub weight_avg: f64,
}
impl DailyStatistics {
    pub fn from_api(v: &Value) -> Self {
        Self {
            date: string(&v["date"]),
            num: int(&v["num"]),
            weight: float(&v["weight"]),
            unit: string(&v["unit"]),
            toilet_total_second: int(&v["toilet_total_second"]),
            weight_avg: float(&v["weight_avg"]),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DeviceStatus {
    pub sand_percent: i64,
    pub cat_present: bool,
    pub cat_stay_seconds: i64,
    pub needs_cleaning: bool,
    pub bucket_full: bool,
    pub operating_state: OperatingState,
    pub last_sand_added: String,
    pub cleaning_enabled: bool,
    pub auto_level: bool,
    pub silent_mode: bool,
    pub child_lock: bool,
    pub young_cat_mode: bool,
    pub last_action: String,
    pub wifi_name: String,
    pub wifi_rssi: i64,
    pub ip_address: String,
    pub mac_address: String,
    pub firmware_version: String,
    pub hardware_version: String,
    pub updated_at: i64,
}
impl DeviceStatus {
    pub fn from_response(v: &Value) -> Self {
        let get = |k: &str| &v[k]["value"];
        Self {
            sand_percent: int(&get("Sand")["percent"]),
            cat_present: int(&get("catLeft")["kitten"]) == 1,
            cat_stay_seconds: int(&get("catLeft")["stayTime"]),
            needs_cleaning: int(&get("catLeft")["needClean"]) == 1,
            bucket_full: int(get("room_of_bin")) != 0,
            operating_state: OperatingState::from(int(get("bucketStatus"))),
            last_sand_added: string(get("latestAddSandTime")),
            cleaning_enabled: int(&get("cleanCfg")["active"]) == 1,
            auto_level: int(get("autoLevel")) == 1,
            silent_mode: int(get("silentMode")) == 1,
            child_lock: int(get("childLockOnOff")) == 1,
            young_cat_mode: int(get("youngCatMode")) == 1,
            last_action: string(get("actionLog")),
            wifi_name: string(&get("NetWorkStatus")["WiFi_Name"]),
            wifi_rssi: int(&get("NetWorkStatus")["WiFi_RSSI"]),
            ip_address: string(&get("NetWorkStatus")["IP_Addr"]),
            mac_address: string(&get("NetWorkStatus")["MAC_Addr"]),
            firmware_version: string(&get("DeviceVer")["FW_Version"]),
            hardware_version: string(&get("DeviceVer")["HW_Version"]),
            updated_at: int(get("timestamp")),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct StatusUpdate {
    pub device_name: String,
    pub changes: serde_json::Map<String, Value>,
    pub received_at: f64,
}
impl StatusUpdate {
    pub fn from_push(body: &Value) -> Option<Self> {
        let device_name = body["params"]["deviceName"].as_str()?.to_string();
        let items = body["params"]["items"].as_object()?;
        let mut changes = serde_json::Map::new();
        for (key, entry) in items {
            if !entry.is_object() {
                continue;
            }
            let v = &entry["value"];
            let bool_key = match key.as_str() {
                "silentMode" => Some("silent_mode"),
                "childLockOnOff" => Some("child_lock"),
                "autoLevel" => Some("auto_level"),
                "youngCatMode" => Some("young_cat_mode"),
                _ => None,
            };
            if let Some(k) = bool_key {
                changes.insert(k.into(), json!(v == 1));
                continue;
            }
            match key.as_str() {
                "room_of_bin" => {
                    changes.insert("bucket_full".into(), json!(v != 0));
                }
                "bucketStatus" if v.is_i64() => {
                    changes.insert(
                        "operating_state".into(),
                        json!(OperatingState::from(int(v))),
                    );
                }
                "latestAddSandTime" if v.is_string() => {
                    changes.insert("last_sand_added".into(), v.clone());
                }
                "actionLog" if v.is_string() => {
                    changes.insert("last_action".into(), v.clone());
                }
                "Sand" if v.is_object() => {
                    if v["percent"].is_i64() {
                        changes.insert("sand_percent".into(), v["percent"].clone());
                    }
                }
                "catLeft" if v.is_object() => {
                    changes.insert("cat_present".into(), json!(v["kitten"] == 1));
                    changes.insert("needs_cleaning".into(), json!(v["needClean"] == 1));
                    if v["stayTime"].is_i64() {
                        changes.insert("cat_stay_seconds".into(), v["stayTime"].clone());
                    }
                }
                "cleanCfg" if v.is_object() => {
                    changes.insert("cleaning_enabled".into(), json!(v["active"] == 1));
                }
                _ => {
                    changes.insert(key.clone(), v.clone());
                }
            }
        }
        Some(Self {
            device_name,
            changes,
            received_at: auth::now_ms() as f64 / 1000.0,
        })
    }
}
