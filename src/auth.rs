//! Byte-compatible REST, Aliyun IoT, and OpenAccount signing.
use crate::{Error, Result, constants::*};
use aes::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit, block_padding::NoPadding};
use base64::{Engine, engine::general_purpose::STANDARD as B64};
use hmac::{Hmac, Mac};
use md5::{Digest, Md5};
use serde_json::{Value, json};
use sha1::Sha1;
use sha2::Sha256;
use std::{
    collections::BTreeMap,
    time::{SystemTime, UNIX_EPOCH},
};

const BOOT_KEY: &[u8; 16] = b"3J74PRUE5TKPJP32";
const BOOT_IV: &[u8; 16] = b"QB8GC2X6WK39FF93";
pub type Headers = BTreeMap<String, String>;
pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
pub fn hmac_sha1(key: &[u8], body: &[u8]) -> Vec<u8> {
    let mut mac = <Hmac<Sha1> as Mac>::new_from_slice(key).expect("HMAC accepts any key length");
    mac.update(body);
    mac.finalize().into_bytes().to_vec()
}
pub fn hmac_sha256(key: &[u8], body: &[u8]) -> Vec<u8> {
    let mut mac = <Hmac<Sha256> as Mac>::new_from_slice(key).expect("HMAC accepts any key length");
    mac.update(body);
    mac.finalize().into_bytes().to_vec()
}
pub fn md5_double_hex(value: &str) -> String {
    hex(&Md5::digest(hex(&Md5::digest(value.as_bytes())).as_bytes()))
}
pub fn aes_encrypt(value: &str, key: &[u8], iv: &[u8]) -> Result<String> {
    let mut bytes = value.as_bytes().to_vec();
    bytes.resize(bytes.len().div_ceil(16) * 16, 0);
    let cipher =
        cbc::Encryptor::<aes::Aes128>::new_from_slices(key, iv).map_err(|_| Error::Crypto)?;
    Ok(B64.encode(cipher.encrypt_padded_vec_mut::<NoPadding>(&bytes)))
}
pub fn boot_encrypt(value: &str) -> Result<String> {
    aes_encrypt(value, BOOT_KEY, BOOT_IV)
}
pub fn decrypt_login_token(value: &str) -> Result<String> {
    let bytes = B64
        .decode(value.replace(' ', "+"))
        .map_err(|_| Error::Crypto)?;
    let cipher = cbc::Decryptor::<aes::Aes128>::new_from_slices(BOOT_KEY, BOOT_IV)
        .map_err(|_| Error::Crypto)?;
    let plain = cipher
        .decrypt_padded_vec_mut::<NoPadding>(&bytes)
        .map_err(|_| Error::Crypto)?;
    String::from_utf8(plain)
        .map(|s| s.trim_end_matches('\0').to_string())
        .map_err(|_| Error::Crypto)
}
pub fn session_token(token: &str, key: &str, iv: &str, timestamp_ms: u64) -> Result<String> {
    aes_encrypt(
        &format!("{token}@{timestamp_ms}"),
        key.as_bytes(),
        iv.as_bytes(),
    )
}
pub fn rest_headers(language: &str, timestamp_sec: u64) -> Headers {
    let sign = B64
        .encode(hmac_sha256(
            APP_SECRET.as_bytes(),
            format!("{APP_KEY}{timestamp_sec}").as_bytes(),
        ))
        .to_uppercase();
    [
        ("appId", APP_KEY.to_string()),
        ("sign", sign.clone()),
        ("request-id", sign),
        ("timestamp", timestamp_sec.to_string()),
        ("version", "226".into()),
        ("versionString", "2.2.6".into()),
        ("brand", "Generic".into()),
        ("model", "neakasa-litterbox-sdk".into()),
        ("Accept-Language", language.into()),
        ("Content-Type", "application/x-www-form-urlencoded".into()),
        ("Charset", "UTF-8".into()),
        ("Accept", "*/*".into()),
        ("User-Agent", USER_AGENT.into()),
    ]
    .into_iter()
    .map(|(k, v)| (k.into(), v))
    .collect()
}
pub fn authenticated_headers(
    login: &crate::LoginResult,
    language: &str,
    ms: u64,
) -> Result<Headers> {
    let mut headers = rest_headers(language, ms / 1000);
    headers.remove("appId");
    headers.remove("sign");
    headers.insert("uid".into(), boot_encrypt(&login.user_id)?);
    headers.insert(
        "token".into(),
        session_token(&login.user_token, &login.aes_key, &login.aes_iv, ms)?,
    );
    Ok(headers)
}
pub fn iot_headers(path: &str, body: &[u8], ms: u64, nonce: &str, date: &str) -> Headers {
    let accept = "application/json; charset=utf-8";
    let content_type = "application/octet-stream; charset=utf-8";
    let digest = B64.encode(Md5::digest(body));
    let canonical = format!(
        "POST\n{accept}\n{digest}\n{content_type}\n{date}\nx-ca-key:{APP_KEY}\nx-ca-nonce:{nonce}\nx-ca-signature-method:HmacSHA1\nx-ca-timestamp:{ms}\n{path}"
    );
    let mut h = common_gateway_headers(
        accept,
        content_type,
        date,
        nonce,
        &ms.to_string(),
        "HmacSHA1",
        B64.encode(hmac_sha1(APP_SECRET.as_bytes(), canonical.as_bytes())),
    );
    h.insert("content-md5".into(), digest);
    h.insert("ca_version".into(), "1".into());
    h
}
pub fn oa_headers(path: &str, canonical_body: &str, sec: u64, nonce: &str, date: &str) -> Headers {
    let accept = "application/json";
    let ct = "application/x-www-form-urlencoded";
    let canonical = format!(
        "POST\n{accept}\n\n{ct}\n{date}\nx-ca-key:{APP_KEY}\nx-ca-nonce:{nonce}\nx-ca-signature-method:HmacSHA256\nx-ca-timestamp:{sec}\n{path}?{canonical_body}"
    );
    common_gateway_headers(
        accept,
        ct,
        date,
        nonce,
        &sec.to_string(),
        "HmacSHA256",
        B64.encode(hmac_sha256(APP_SECRET.as_bytes(), canonical.as_bytes())),
    )
}
fn common_gateway_headers(
    accept: &str,
    ct: &str,
    date: &str,
    nonce: &str,
    ts: &str,
    method: &str,
    signature: String,
) -> Headers {
    [
        ("accept", accept),
        ("content-type", ct),
        ("date", date),
        ("x-ca-key", APP_KEY),
        ("x-ca-nonce", nonce),
        ("x-ca-timestamp", ts),
        ("x-ca-signature-method", method),
        ("x-ca-signature", &signature),
        (
            "x-ca-signature-headers",
            "x-ca-nonce,x-ca-timestamp,x-ca-key,x-ca-signature-method",
        ),
        ("user-agent", USER_AGENT),
    ]
    .into_iter()
    .map(|(k, v)| (k.into(), v.into()))
    .collect()
}
pub fn iot_envelope(
    payload: Value,
    version: &str,
    token: Option<&str>,
    language: &str,
    id: &str,
) -> Value {
    let mut request = json!({"apiVer":version,"language":language});
    if let Some(token) = token {
        request["iotToken"] = json!(token);
    }
    json!({"id":id,"version":"1.0","request":request,"params":payload})
}
pub fn oa_body(body: &Value) -> Result<(String, String)> {
    let fields = body.as_object().ok_or(Error::InvalidResponse)?;
    let mut canonical = Vec::new();
    let mut wire = url::form_urlencoded::Serializer::new(String::new());
    for (key, value) in fields {
        let v = serde_json::to_string(value)?;
        canonical.push(format!("{key}={v}"));
        wire.append_pair(key, &v);
    }
    Ok((canonical.join("&"), wire.finish()))
}
pub fn mqtt_auth_info(ms: u64, client_id: &str, device_sn: &str) -> Value {
    let sign = format!("appKey{APP_KEY}clientId{client_id}deviceSn{device_sn}timestamp{ms}");
    json!({"clientId":client_id,"deviceSn":device_sn,"timestamp":ms.to_string(),"sign":hex(&hmac_sha1(APP_SECRET.as_bytes(),sign.as_bytes()))})
}
