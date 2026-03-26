use pamsm::{PamError, PamFlags, PamServiceModule, Pam, PamLibExt};
use tokio::runtime::Runtime;
use zbus::Connection;
use tracing::{info, error, Level};

struct PamHelu;

#[zbus::proxy(
    interface = "net.helu.Auth",
    default_service = "net.helu.Auth",
    default_path = "/net/helu/Auth"
)]
trait HeluAuth {
    async fn authenticate(&self, username: String, method: String) -> zbus::Result<(bool, String)>;
}

impl PamServiceModule for PamHelu {
    fn setcred(
        _pamh: Pam,
        _flags: PamFlags,
        _args: Vec<String>,
    ) -> PamError {
        PamError::SUCCESS
    }

    fn authenticate(
        pamh: Pam,
        _flags: PamFlags,
        _args: Vec<String>,
    ) -> PamError {
        let _ = tracing_subscriber::fmt()
            .with_max_level(Level::INFO)
            .try_init();

        info!("pam_helu triggered");

        let user = match pamh.get_user(None) {
            Ok(Some(u)) => match u.to_str() {
                Ok(s) => s.to_string(),
                Err(_) => {
                    error!("Invalid UTF-8 in username");
                    return PamError::USER_UNKNOWN;
                }
            },
            _ => {
                error!("Could not retrieve username from PAM");
                return PamError::USER_UNKNOWN;
            }
        };

        info!("Authenticating user: {}", user);

        let rt = match Runtime::new() {
            Ok(rt) => rt,
            Err(e) => {
                error!("Failed to create tokio runtime: {}", e);
                return PamError::AUTH_ERR;
            }
        };

        let result = rt.block_on(async {
            // First try system bus, then session bus as fallback (for dev)
            let conn_res = Connection::system().await;

            let conn = match conn_res {
                Ok(c) => {
                    info!("Connected to D-Bus (system)");
                    c
                },
                Err(_) => {
                    match Connection::session().await {
                        Ok(c) => {
                            info!("Connected to D-Bus (session) fallback");
                            c
                        },
                        Err(e) => {
                            error!("Failed to connect to D-Bus: {}", e);
                            return Err(PamError::AUTHINFO_UNAVAIL);
                        }
                    }
                }
            };

            let proxy = match HeluAuthProxy::new(&conn).await {
                Ok(p) => p,
                Err(e) => {
                    error!("Failed to create D-Bus proxy: {}", e);
                    return Err(PamError::AUTHINFO_UNAVAIL);
                }
            };

            match proxy.authenticate(user.clone(), "auto".to_string()).await {
                Ok((true, msg)) => {
                    info!("Helud auth success: {}", msg);
                    Ok(PamError::SUCCESS)
                }
                Ok((false, msg)) => {
                    info!("Helud auth failed: {}", msg);
                    Ok(PamError::AUTH_ERR)
                }
                Err(e) => {
                    error!("Helud D-Bus call failed: {}", e);
                    Ok(PamError::AUTHINFO_UNAVAIL)
                }
            }
        });

        result.unwrap_or_else(|e| e)
    }
}

pamsm::pam_module!(PamHelu);
