//! Python-compatible private state files and optional Android session import.
use crate::{
    Error, LoginResult, Result, auth,
    constants::{APP_KEY, APP_SECRET},
};
use aes_gcm::{
    Aes128Gcm, Nonce,
    aead::{Aead, KeyInit},
};
use base64::{Engine, engine::general_purpose::STANDARD as B64};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Clone, Serialize, Deserialize)]
pub struct SavedSession {
    pub session: LoginResult,
    pub base_url: String,
}
#[derive(Default, Deserialize)]
pub struct Credentials {
    pub email: String,
    pub password: String,
}
pub fn default_state_dir() -> Result<PathBuf> {
    Ok(PathBuf::from(std::env::var_os("HOME").ok_or(Error::State)?)
        .join(".local/state/neakasa-api"))
}
pub fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}
/// Atomic replacement avoids truncated sessions; secrets never become world-readable.
pub fn save_session(dir: &Path, session: &LoginResult, base_url: &str) -> Result<()> {
    fs::create_dir_all(dir)?;
    #[cfg(unix)]
    fs::set_permissions(dir, fs::Permissions::from_mode(0o700))?;
    let path = dir.join(format!(".session-{}.tmp", uuid::Uuid::new_v4()));
    let result = (|| -> Result<()> {
        let mut opts = OpenOptions::new();
        opts.create_new(true).write(true);
        #[cfg(unix)]
        opts.mode(0o600);
        let mut file = opts.open(&path)?;
        serde_json::to_writer(
            &mut file,
            &SavedSession {
                session: session.clone(),
                base_url: base_url.into(),
            },
        )?;
        file.write_all(b"\n")?;
        file.sync_all()?;
        fs::rename(&path, dir.join("session.json"))?;
        fs::File::open(dir)?.sync_all()?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(path);
    }
    result
}
async fn adb_read(path: &str) -> Result<String> {
    let output = tokio::time::timeout(
        std::time::Duration::from_secs(20),
        tokio::process::Command::new("adb")
            .kill_on_drop(true)
            .args(["-s", "127.0.0.1:6520", "shell", "cat", path])
            .output(),
    )
    .await
    .map_err(|_| Error::Timeout)??;
    if !output.status.success() {
        return Err(Error::State);
    }
    String::from_utf8(output.stdout).map_err(|_| Error::InvalidResponse)
}
fn prefs(xml: &str) -> Result<HashMap<String, String>> {
    let doc = roxmltree::Document::parse(xml).map_err(|_| Error::InvalidResponse)?;
    Ok(doc
        .root_element()
        .children()
        .filter_map(|n| {
            Some((
                n.attribute("name")?.to_string(),
                n.text().unwrap_or("").to_string(),
            ))
        })
        .collect())
}
pub fn decode_android_session(user_xml: &str, secure_xml: &str) -> Result<SavedSession> {
    let p = prefs(user_xml)?;
    let secure = prefs(secure_xml)?;
    let raw = B64
        .decode(p.get("USER_OBJ").ok_or(Error::InvalidResponse)?)
        .map_err(|_| Error::Crypto)?;
    let user: Value = serde_json::from_slice(&raw)?;
    let mut session = LoginResult::from_api(&user)?;
    let base_url = p
        .get("CurrentBaseRegionURL")
        .ok_or(Error::InvalidResponse)?
        .trim_end_matches('/')
        .to_string();
    let key = auth::hex(&auth::hmac_sha1(
        APP_SECRET.as_bytes(),
        format!("safe-key-level-21{APP_KEY}").as_bytes(),
    ));
    let bytes = B64
        .decode(
            secure
                .get("KEY_IOT_CREDENTIAL_DATAv2")
                .ok_or(Error::InvalidResponse)?,
        )
        .map_err(|_| Error::Crypto)?;
    if bytes.len() < 28 {
        return Err(Error::Crypto);
    }
    let cipher = Aes128Gcm::new_from_slice(&key.as_bytes()[..16]).map_err(|_| Error::Crypto)?;
    let plain = cipher
        .decrypt(Nonce::from_slice(&bytes[..12]), &bytes[12..])
        .map_err(|_| Error::Crypto)?;
    let creds: Value = serde_json::from_slice(&plain)?;
    if creds["identity"] != user["userInfo"]["aliIdentityId"] || creds["identity"].is_null() {
        return Err(Error::InvalidResponse);
    }
    session.iot_token = creds["iotToken"]
        .as_str()
        .filter(|s| !s.is_empty())
        .ok_or(Error::NotAuthenticated)?
        .into();
    Ok(SavedSession { session, base_url })
}
pub async fn import_android() -> Result<SavedSession> {
    let user =
        adb_read("/data/data/com.jhkj.neakasa/shared_prefs/com.jhkj.neakasa_prefs.xml").await?;
    let secure = adb_read("/data/data/com.jhkj.neakasa/shared_prefs/SecureDataStore.xml").await?;
    decode_android_session(&user, &secure)
}
