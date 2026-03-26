use tauri::{AppHandle, Emitter, Manager};
use zbus::{Connection, proxy};
use futures_lite::stream::StreamExt;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, error};

pub struct HeluState {
    pub is_authenticating: bool,
    pub current_user: Option<String>,
}

impl HeluState {
    pub fn new() -> Self {
        Self {
            is_authenticating: false,
            current_user: None,
        }
    }
}

#[proxy(
    interface = "net.helu.Auth",
    default_service = "net.helu.Auth",
    default_path = "/net/helu/Auth"
)]
trait HeluAuth {
    #[zbus(signal)]
    async fn auth_requested(&self, username: &str, method: &str) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn auth_success(&self, username: &str, method: &str) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn auth_failure(&self, username: &str, reason: String) -> zbus::Result<()>;
}

#[derive(serde::Serialize, Clone)]
struct AuthEventPayload {
    username: String,
    method: String,
    status: String,
    message: Option<String>,
}

pub async fn start_dbus_listener(
    app_handle: AppHandle,
    state: Arc<Mutex<HeluState>>,
) -> zbus::Result<()> {
    // Try system bus first, then session
    let conn = match Connection::system().await {
        Ok(c) => c,
        Err(_) => Connection::session().await?,
    };

    let proxy = HeluAuthProxy::new(&conn).await?;
    info!("D-Bus listener started");

    let mut auth_requested_stream = proxy.receive_auth_requested().await?;
    let mut auth_success_stream = proxy.receive_auth_success().await?;
    let mut auth_failure_stream = proxy.receive_auth_failure().await?;

    loop {
        tokio::select! {
            Some(signal) = auth_requested_stream.next() => {
                let args = signal.args()?;
                info!("Auth requested for {} via {}", args.username(), args.method());

                let mut st = state.lock().await;
                st.is_authenticating = true;
                st.current_user = Some(args.username().to_string());

                let payload = AuthEventPayload {
                    username: args.username().to_string(),
                    method: args.method().to_string(),
                    status: "requested".to_string(),
                    message: None,
                };
                let _ = app_handle.emit("auth-event", payload);

                // For Wayland/X11 we would unhide/focus the window here
                if let Some(window) = app_handle.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            Some(signal) = auth_success_stream.next() => {
                let args = signal.args()?;
                info!("Auth success for {}", args.username());

                let mut st = state.lock().await;
                st.is_authenticating = false;

                let payload = AuthEventPayload {
                    username: args.username().to_string(),
                    method: args.method().to_string(),
                    status: "success".to_string(),
                    message: None,
                };
                let _ = app_handle.emit("auth-event", payload);

                // Auto-hide after success
                if let Some(window) = app_handle.get_webview_window("main") {
                    tokio::spawn(async move {
                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                        let _ = window.hide();
                    });
                }
            }
            Some(signal) = auth_failure_stream.next() => {
                let args = signal.args()?;
                info!("Auth failure for {}: {}", args.username(), args.reason());

                let payload = AuthEventPayload {
                    username: args.username().to_string(),
                    method: "".to_string(),
                    status: "failure".to_string(),
                    message: Some(args.reason().to_string()),
                };
                let _ = app_handle.emit("auth-event", payload);
            }
        }
    }
}
