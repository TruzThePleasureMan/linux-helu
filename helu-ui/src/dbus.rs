use zbus::Connection;
use helu_common::types::AuthState;
use futures_lite::stream::StreamExt;

pub enum UiEvent {
    StateChange(AuthState),
}

use helu_common::dbus::AuthProxy;

pub async fn listen_to_daemon(tx: async_channel::Sender<UiEvent>) -> zbus::Result<()> {
    let connection = Connection::session().await?;
    let proxy = AuthProxy::new(&connection).await?;
    let mut stream = proxy.receive_auth_state_changed().await?;

    while let Some(signal) = stream.next().await {
        let args = signal.args()?;
        let _ = tx.send_blocking(UiEvent::StateChange(args.state().clone()));
    }

    Ok(())
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
