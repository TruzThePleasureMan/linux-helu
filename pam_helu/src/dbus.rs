use anyhow::{anyhow, Result};
use std::process::Command;
use std::time::Duration;
use tokio::time::sleep;
use zbus::Connection;
use log::{info, error};
use helu_common::dbus::AuthProxy;
use zbus::fdo::DBusProxy;

pub async fn helud_available() -> bool {
    // Try system bus first
    let conn = match Connection::system().await {
        Ok(c) => c,
        Err(_) => {
            // Fallback to session bus for development
            match Connection::session().await {
                Ok(c) => c,
                Err(_) => return false,
            }
        }
    };

    let proxy = match DBusProxy::new(&conn).await {
        Ok(p) => p,
        Err(_) => return false,
    };

    // Check if the auth service is registered
    proxy.name_has_owner("net.helu.Auth".try_into().unwrap()).await.unwrap_or(false)
}

pub async fn check_ui_ready() -> bool {
    let conn = match Connection::session().await {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to connect to session bus for UI check: {}", e);
            return false;
        }
    };

    let proxy = match DBusProxy::new(&conn).await {
        Ok(p) => p,
        Err(e) => {
            error!("Failed to create DBusProxy: {}", e);
            return false;
        }
    };

    if proxy.name_has_owner("net.helu.UI".try_into().unwrap()).await.unwrap_or(false) {
        return true;
    }

    info!("UI not running, attempting to start via systemctl...");
    let _ = Command::new("systemctl")
        .args(["--user", "start", "helu-ui"])
        .status();

    // Poll for up to 3 seconds
    for _ in 0..30 {
        sleep(Duration::from_millis(100)).await;
        if proxy.name_has_owner("net.helu.UI".try_into().unwrap()).await.unwrap_or(false) {
            info!("UI started successfully");
            return true;
        }
    }

    error!("UI failed to start within 3 seconds");
    false
}

pub async fn call_authenticate(username: &str, method: &str) -> Result<(bool, String)> {
    let conn = match Connection::system().await {
        Ok(c) => c,
        Err(_) => {
            match Connection::session().await {
                Ok(c) => c,
                Err(e) => return Err(anyhow!("Failed to connect to D-Bus: {}", e)),
            }
        }
    };

    let proxy = match AuthProxy::new(&conn).await {
        Ok(p) => p,
        Err(e) => return Err(anyhow!("Failed to create AuthProxy: {}", e)),
    };

    let result = tokio::time::timeout(
        Duration::from_secs(30),
        proxy.authenticate(username, method),
    )
    .await;

    match result {
        Ok(Ok(res)) => Ok(res),
        Ok(Err(e)) => Err(anyhow!("D-Bus error calling authenticate: {}", e)),
        Err(_) => Err(anyhow!("Timeout waiting for helud authentication")),
    }
}

pub async fn call_authenticate_with_credential(username: &str, method: &str, credential: &str) -> Result<(bool, String)> {
    let conn = match Connection::system().await {
        Ok(c) => c,
        Err(_) => {
            match Connection::session().await {
                Ok(c) => c,
                Err(e) => return Err(anyhow!("Failed to connect to D-Bus: {}", e)),
            }
        }
    };

    let proxy = match AuthProxy::new(&conn).await {
        Ok(p) => p,
        Err(e) => return Err(anyhow!("Failed to create AuthProxy: {}", e)),
    };

    let result = tokio::time::timeout(
        Duration::from_secs(30),
        proxy.authenticate_with_credential(username, method, credential),
    )
    .await;

    match result {
        Ok(Ok(res)) => Ok(res),
        Ok(Err(e)) => Err(anyhow!("D-Bus error calling authenticate_with_credential: {}", e)),
        Err(_) => Err(anyhow!("Timeout waiting for helud authentication")),
    }
}
