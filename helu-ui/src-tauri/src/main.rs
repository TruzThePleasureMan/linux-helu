#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod bridge;

use tauri::Manager;
use tracing::{info, error, Level};
use bridge::{start_dbus_listener, HeluState};
use std::sync::Arc;
use zbus::Connection;
use zbus::proxy;

#[proxy(
    interface = "net.helu.Auth",
    default_service = "net.helu.Auth",
    default_path = "/net/helu/Auth"
)]
trait HeluAuth {
    async fn authenticate(&self, username: String, method: String) -> zbus::Result<(bool, String)>;
}

#[tauri::command]
async fn send_auth_response(
    _state: tauri::State<'_, Arc<tokio::sync::Mutex<HeluState>>>,
    success: bool,
    method: String,
) -> Result<(), String> {
    info!("UI sent auth response: success={}, method={}", success, method);

    // Call daemon back for the PIN
    let conn_res = Connection::system().await;

    let conn = match conn_res {
        Ok(c) => c,
        Err(_) => Connection::session().await.map_err(|e| e.to_string())?
    };

    let proxy = HeluAuthProxy::new(&conn).await.map_err(|e| e.to_string())?;

    // In a real system, the daemon handles the validation logic. We trigger auto again but setting a flag,
    // or just let the proxy authenticate pin.
    let user = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());

    if method == "pin" {
        // Send a pin to daemon to be verified
        std::env::set_var("HELU_MOCK_PIN", "1234");

        match proxy.authenticate(user, method.clone()).await {
            Ok((true, msg)) => {
                info!("Daemon auth success: {}", msg);
                Ok(())
            }
            Ok((false, msg)) => {
                info!("Daemon auth failed: {}", msg);
                Err(msg)
            }
            Err(e) => {
                error!("Daemon D-Bus call failed: {}", e);
                Err(e.to_string())
            }
        }
    } else {
        Ok(())
    }
}

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting Helu UI...");

    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle().clone();
            let state = Arc::new(tokio::sync::Mutex::new(HeluState::new()));
            app.manage(state.clone());

            // Start D-Bus listener in background
            tauri::async_runtime::spawn(async move {
                if let Err(e) = start_dbus_listener(app_handle, state).await {
                    error!("D-Bus listener failed: {}", e);
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![send_auth_response])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
