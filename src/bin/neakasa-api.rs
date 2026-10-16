use clap::{Parser, Subcommand, ValueEnum};
use neakasa_litterbox_sdk::{
    Device, Error, NeakasaClient, Region, Result, StreamOptions, WeightUnit, auth,
    state::{self, Credentials, SavedSession},
};
use serde_json::{Value, json};
use std::{path::PathBuf, time::Duration};

#[derive(Parser)]
#[command(
    version,
    about = "Native Rust Neakasa API; compatible with the existing Python state files"
)]
struct Args {
    #[arg(long, global = true, conflicts_with = "import_session")]
    login: bool,
    #[arg(long, global = true)]
    import_session: bool,
    #[arg(long, global = true)]
    state_dir: Option<PathBuf>,
    #[command(subcommand)]
    command: Option<Command>,
}
#[derive(Subcommand, Default)]
enum Command {
    #[default]
    Cats,
    Status,
    History,
    /// Raw properties can include owner identifiers.
    Properties,
    Devices,
    Statistics {
        #[arg(long)]
        start: i64,
        #[arg(long)]
        end: i64,
        #[arg(long, default_value_t = 0, allow_hyphen_values = true)]
        zone_seconds: i32,
    },
    /// Stream property updates until Ctrl-C.
    Watch,
    /// Update only the specified cat's weight, then verify all profiles.
    SetCatWeight {
        #[arg(long)]
        cat_id: i64,
        #[arg(long)]
        weight: f64,
        #[arg(long, value_enum)]
        unit: Unit,
        #[arg(long, default_value_t = 0, allow_hyphen_values = true)]
        zone_seconds: i32,
    },
}
#[derive(Clone, Copy, ValueEnum)]
enum Unit {
    Kg,
    Lb,
}
#[tokio::main]
async fn main() {
    let args = Args::parse();
    if let Err(error) = run(args).await {
        println!(
            "{}",
            json!({"error":error.kind(),"code":error.code(),"hint":"If authentication expired, retry with --login."})
        );
        std::process::exit(1);
    }
}
async fn run(args: Args) -> Result<()> {
    let dir = match args.state_dir {
        Some(d) => d,
        None => state::default_state_dir()?,
    };
    let saved = if args.import_session {
        state::import_android().await?
    } else {
        state::read_json::<SavedSession>(&dir.join("session.json"))?
    };
    // Preserve the Python wrapper's endpoint guard when reading private state.
    if saved.base_url != Region::Us.web_url() {
        return Err(Error::InvalidInput("unexpected app API endpoint"));
    }
    let creds = if dir.join("credentials.json").exists() {
        state::read_json::<Credentials>(&dir.join("credentials.json"))?
    } else {
        Credentials::default()
    };
    let mut client = NeakasaClient::with_endpoint(
        creds.email,
        creds.password,
        &saved.base_url,
        Duration::from_secs(20),
    )?;
    client.resume_session(saved.session);
    if args.login {
        client.login(None).await?;
        state::save_session(
            &dir,
            client.login_result().ok_or(Error::NotAuthenticated)?,
            &saved.base_url,
        )?;
    }
    let device = Device::from_api(&state::read_json::<Value>(&dir.join("device.json"))?);
    if device.device_name.is_empty() || device.iot_id.is_empty() {
        return Err(Error::InvalidInput("invalid saved device"));
    }
    client.register_device(device.clone());
    let result = execute(&mut client, &device, args.command.unwrap_or_default()).await;
    // Persist refreshes even when the original operation fails; never expose token data.
    let save = state::save_session(
        &dir,
        client.login_result().ok_or(Error::NotAuthenticated)?,
        &saved.base_url,
    );
    match result {
        Ok(Some(value)) => {
            save?;
            println!("{}", serde_json::to_string_pretty(&value)?);
            Ok(())
        }
        Ok(None) => {
            save?;
            Ok(())
        }
        Err(error) => {
            let _ = save;
            Err(error)
        }
    }
}
async fn execute(
    client: &mut NeakasaClient,
    device: &Device,
    command: Command,
) -> Result<Option<Value>> {
    let name = &device.device_name;
    let mut row = json!({"device_name":name,"product":device.product_name});
    match command {
        Command::Cats => {
            row["cats"] = json!(
                client
                    .list_cats(name)
                    .await?
                    .into_iter()
                    .map(|c| json!({"name":c.name,"weight":c.weight,"unit":c.unit}))
                    .collect::<Vec<_>>()
            );
        }
        Command::Status => {
            row["status"] = json!(client.get_status(name).await?);
        }
        Command::History => {
            let now = (auth::now_ms() / 1000) as i64;
            row["records"] = json!(client.get_toilet_records(name, now - 86400, now).await?);
        }
        Command::Properties => {
            row["properties"] = client.get_properties(name).await?;
        }
        Command::Devices => return Ok(Some(json!(client.list_devices().await?))),
        Command::Statistics {
            start,
            end,
            zone_seconds,
        } => {
            row["statistics"] = json!(
                client
                    .get_toilet_statistics(name, start, end, zone_seconds)
                    .await?
            );
        }
        Command::Watch => {
            let mut stream = client.watch_status(StreamOptions::default()).await?;
            eprintln!("Connected; waiting for property updates (Ctrl-C to stop).");
            loop {
                tokio::select! {
                    _=tokio::signal::ctrl_c()=>{stream.stop().await;break;},
                    update=stream.next()=>{match update{Some(result)=>println!("{}",serde_json::to_string(&result?)?),None=>break,}}
                }
            }
            return Ok(None);
        }
        Command::SetCatWeight {
            cat_id,
            weight,
            unit,
            zone_seconds,
        } => {
            let unit = match unit {
                Unit::Kg => WeightUnit::Kg,
                Unit::Lb => WeightUnit::Lb,
            };
            let before = client.list_cats(name).await?;
            let target = before
                .iter()
                .find(|c| c.id == cat_id)
                .ok_or(Error::InvalidInput("cat ID not in this device's profiles"))?;
            client
                .set_cat_weight(name, cat_id, weight, unit, zone_seconds)
                .await?;
            let after = client.list_cats(name).await?;
            let mut expected = before.clone();
            for cat in &mut expected {
                if cat.id == cat_id {
                    cat.weight = weight;
                    cat.unit = unit.as_str().into();
                }
            }
            let mut actual = after.clone();
            expected.sort_by_key(|c| c.id);
            actual.sort_by_key(|c| c.id);
            if expected != actual {
                return Err(Error::InvalidResponse);
            }
            row["updated"] =
                json!({"name":target.name,"weight":weight,"unit":unit,"verified":true});
        }
    }
    Ok(Some(json!([row])))
}
