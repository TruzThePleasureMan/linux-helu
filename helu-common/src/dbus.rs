use crate::types::AuthState;

/// The proxy trait for the UI to talk to the Daemon
#[zbus::proxy(
    interface = "net.helu.Auth",
    default_service = "net.helu.Auth",
    default_path = "/net/helu/Auth"
)]
pub trait Auth {
    fn authenticate(&self, username: &str, method: &str) -> zbus::Result<(bool, String)>;
    fn enroll(&self, username: &str, method: &str) -> zbus::Result<bool>;
    fn list_methods(&self, username: &str) -> zbus::Result<Vec<String>>;
    fn status(&self) -> zbus::Result<(String, Vec<String>)>;

    // UI listens to these signals
    #[zbus(signal)]
    fn auth_requested(&self, username: &str) -> zbus::Result<()>;
    #[zbus(signal)]
    fn auth_success(&self, username: &str) -> zbus::Result<()>;
    #[zbus(signal)]
    fn auth_failure(&self, username: &str, reason: &str) -> zbus::Result<()>;

    // Auth state changed. Uses a tagged struct to workaround zvariant 4 struct field count requirement.
    #[zbus(signal)]
    fn auth_state_changed(&self, state: AuthState) -> zbus::Result<()>;
}

/// The proxy trait for the Daemon to talk to the UI
#[zbus::proxy(
    interface = "net.helu.UI",
    default_service = "net.helu.UI",
    default_path = "/net/helu/UI"
)]
pub trait UI {
    #[zbus(signal)]
    fn pin_submitted(&self, username: &str, pin: &str) -> zbus::Result<()>;
    #[zbus(signal)]
    fn ui_ready(&self) -> zbus::Result<()>;
}
