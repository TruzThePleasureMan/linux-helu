use zbus::interface;
use zbus::object_server::SignalEmitter;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::config::Config;
use crate::auth::AuthManager;
use tracing::{info, error};

pub struct HeluAuth {
    #[allow(dead_code)]
    config: Arc<Config>,
    auth_manager: Arc<Mutex<AuthManager>>,
}

impl HeluAuth {
    pub fn new(config: Arc<Config>, auth_manager: Arc<Mutex<AuthManager>>) -> Self {
        Self {
            config,
            auth_manager,
        }
    }
}

#[interface(name = "net.helu.Auth")]
impl HeluAuth {
    async fn authenticate(
        &self,
        #[zbus(signal_context)] ctxt: SignalEmitter<'_>,
        username: String,
        method: String,
    ) -> (bool, String) {
        info!("Auth request for {} via {}", username, method);

        let manager = self.auth_manager.clone();

        // Emit AuthRequested signal
        if let Err(e) = HeluAuth::auth_requested(&ctxt, &username, &method).await {
            error!("Failed to emit AuthRequested signal: {}", e);
        }

        let mut mgr = manager.lock().await;
        let result = mgr.authenticate(&username, &method).await;

        match result {
            Ok(true) => {
                if let Err(e) = HeluAuth::auth_success(&ctxt, &username, &method).await {
                    error!("Failed to emit AuthSuccess signal: {}", e);
                }
                (true, "Authentication successful".to_string())
            }
            Ok(false) => {
                if let Err(e) = HeluAuth::auth_failure(&ctxt, &username, "Authentication failed".to_string()).await {
                    error!("Failed to emit AuthFailure signal: {}", e);
                }
                (false, "Authentication failed".to_string())
            }
            Err(e) => {
                let msg = format!("Error during auth: {}", e);
                if let Err(e2) = HeluAuth::auth_failure(&ctxt, &username, msg.clone()).await {
                    error!("Failed to emit AuthFailure signal: {}", e2);
                }
                (false, msg)
            }
        }
    }

    async fn authenticate_with_credential(
        &self,
        #[zbus(signal_context)] ctxt: SignalEmitter<'_>,
        username: String,
        method: String,
        credential: String,
    ) -> (bool, String) {
        info!("Auth request for {} via {} with credential", username, method);

        let manager = self.auth_manager.clone();

        // Emit AuthRequested signal
        if let Err(e) = HeluAuth::auth_requested(&ctxt, &username, &method).await {
            error!("Failed to emit AuthRequested signal: {}", e);
        }

        let mut mgr = manager.lock().await;
        let result = mgr.authenticate_with_credential(&username, &method, &credential).await;

        match result {
            Ok(true) => {
                if let Err(e) = HeluAuth::auth_success(&ctxt, &username, &method).await {
                    error!("Failed to emit AuthSuccess signal: {}", e);
                }
                (true, "Authentication successful".to_string())
            }
            Ok(false) => {
                if let Err(e) = HeluAuth::auth_failure(&ctxt, &username, "Authentication failed".to_string()).await {
                    error!("Failed to emit AuthFailure signal: {}", e);
                }
                (false, "Authentication failed".to_string())
            }
            Err(e) => {
                let msg = format!("Error during auth: {}", e);
                if let Err(e2) = HeluAuth::auth_failure(&ctxt, &username, msg.clone()).await {
                    error!("Failed to emit AuthFailure signal: {}", e2);
                }
                (false, msg)
            }
        }
    }

    async fn enroll(&self, username: String, method: String) -> bool {
        info!("Enrollment request for {} via {}", username, method);
        let mut mgr = self.auth_manager.lock().await;
        mgr.enroll(&username, &method).await.unwrap_or(false)
    }

    async fn list_methods(&self, username: String) -> Vec<String> {
        let mgr = self.auth_manager.lock().await;
        mgr.list_methods(&username)
    }

    async fn status(&self) -> (String, Vec<String>) {
        let mgr = self.auth_manager.lock().await;
        (
            env!("CARGO_PKG_VERSION").to_string(),
            mgr.loaded_methods()
        )
    }

    // Signals
    #[zbus(signal)]
    async fn auth_requested(ctxt: &SignalEmitter<'_>, username: &str, method: &str) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn auth_success(ctxt: &SignalEmitter<'_>, username: &str, method: &str) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn auth_failure(ctxt: &SignalEmitter<'_>, username: &str, reason: String) -> zbus::Result<()>;
}
