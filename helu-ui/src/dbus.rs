use zbus::Connection;
use helu_common::types::AuthState;
use futures_lite::stream::StreamExt;

pub enum UiEvent {
    StateChange(AuthState),
    AuthRequested { username: String },
    AuthSuccess { username: String },
    AuthFailure { username: String, reason: String },
}

use helu_common::dbus::AuthProxy;

pub async fn listen_to_daemon(tx: async_channel::Sender<UiEvent>) -> zbus::Result<()> {
    let connection = Connection::session().await?;
    let proxy = AuthProxy::new(&connection).await?;

    let mut state_stream = proxy.receive_auth_state_changed().await?;
    let mut req_stream = proxy.receive_auth_requested().await?;
    let mut succ_stream = proxy.receive_auth_success().await?;
    let mut fail_stream = proxy.receive_auth_failure().await?;

    loop {
        tokio::select! {
            Some(signal) = state_stream.next() => {
                if let Ok(args) = signal.args() {
                    let _ = tx.send_blocking(UiEvent::StateChange(args.state().clone()));
                }
            }
            Some(signal) = req_stream.next() => {
                if let Ok(args) = signal.args() {
                    let _ = tx.send_blocking(UiEvent::AuthRequested { username: args.username().to_string() });
                }
            }
            Some(signal) = succ_stream.next() => {
                if let Ok(args) = signal.args() {
                    let _ = tx.send_blocking(UiEvent::AuthSuccess { username: args.username().to_string() });
                }
            }
            Some(signal) = fail_stream.next() => {
                if let Ok(args) = signal.args() {
                    let _ = tx.send_blocking(UiEvent::AuthFailure {
                        username: args.username().to_string(),
                        reason: args.reason().to_string()
                    });
                }
            }
        }
    }
}

pub async fn emit_ui_ready() -> zbus::Result<()> {
    let connection = Connection::session().await?;
    connection.emit_signal(
        Option::<&str>::None,
        "/net/helu/UI",
        "net.helu.UI",
        "UIReady",
        &(),
    ).await
}

pub async fn submit_pin(username: &str, pin: &str) -> zbus::Result<()> {
    let connection = Connection::session().await?;
    connection.emit_signal(
        Option::<&str>::None,
        "/net/helu/UI",
        "net.helu.UI",
        "PinSubmitted",
        &(username, pin),
    ).await
}
