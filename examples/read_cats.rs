use neakasa_litterbox_sdk::{
    Device, Error, NeakasaClient, Result,
    state::{self, Credentials, SavedSession},
};
use serde_json::Value;
use std::time::Duration;

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
async fn run() -> Result<()> {
    let dir = state::default_state_dir()?;
    let saved: SavedSession = state::read_json(&dir.join("session.json"))?;
    let credentials: Credentials = state::read_json(&dir.join("credentials.json"))?;
    let device = Device::from_api(&state::read_json::<Value>(&dir.join("device.json"))?);
    let mut client = NeakasaClient::with_endpoint(
        credentials.email,
        credentials.password,
        &saved.base_url,
        Duration::from_secs(20),
    )?;
    client.resume_session(saved.session);
    client.register_device(device.clone());
    let result = client.list_cats(&device.device_name).await;
    state::save_session(
        &dir,
        client.login_result().ok_or(Error::NotAuthenticated)?,
        &saved.base_url,
    )?;
    for cat in result? {
        println!("{} (ID {}): {} {}", cat.name, cat.id, cat.weight, cat.unit);
    }
    Ok(())
}
