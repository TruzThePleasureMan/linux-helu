mod config;
mod dbus;
mod auth;

use clap::Parser;
use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Result;
use zbus::Connection;
use tracing::{info, Level};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    mock: bool,

    #[arg(short, long)]
    bus: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting Helud...");

    let cli = Cli::parse();
    let mut conf = config::Config::load()?;

    if cli.mock {
        conf.face.mock_hardware = true;
        conf.fingerprint.mock_hardware = true;
        conf.fido2.mock_hardware = true;
    }

    if let Some(bus) = cli.bus {
        conf.daemon.bus = bus;
    }

    let config = Arc::new(conf.clone());

    let mut auth_manager = auth::AuthManager::new();

    if conf.face.enabled {
        auth_manager.register_method(Box::new(auth::face::FaceAuth::new(conf.face.clone())));
    }
    if conf.fingerprint.enabled {
        auth_manager.register_method(Box::new(auth::fingerprint::FingerprintAuth::new(conf.fingerprint.clone())));
    }
    if conf.pin.enabled {
        auth_manager.register_method(Box::new(auth::pin::PinAuth::new(conf.pin.clone())));
    }
    if conf.fido2.enabled {
        auth_manager.register_method(Box::new(auth::fido2::Fido2Auth::new(conf.fido2.clone())));
    }

    let auth_manager = Arc::new(Mutex::new(auth_manager));
    let helu_auth = dbus::HeluAuth::new(config.clone(), auth_manager);

    let mut builder = zbus::connection::Builder::system()?;
    if config.daemon.bus == "session" {
        builder = zbus::connection::Builder::session()?;
    }

    let _connection = builder
        .name("net.helu.Auth")?
        .serve_at("/net/helu/Auth", helu_auth)?
        .build()
        .await?;

    info!("Helud D-Bus service running on {} bus.", config.daemon.bus);

    // Keep the daemon running
    std::future::pending::<()>().await;

    Ok(())
}
