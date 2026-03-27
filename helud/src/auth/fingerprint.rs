use super::AuthMethod;
use anyhow::Result;
use helu_common::types::AuthResult;
use zbus::proxy;
use tracing::{info, error, warn};
use tokio::time::{timeout, Duration};

#[proxy(
    interface = "net.reactivated.Fprint.Manager",
    default_service = "net.reactivated.Fprint",
    default_path = "/net/reactivated/Fprint/Manager"
)]
trait FprintManager {
    fn get_default_device(&self) -> zbus::Result<zbus::zvariant::ObjectPath<'static>>;
}

#[proxy(
    interface = "net.reactivated.Fprint.Device",
    default_service = "net.reactivated.Fprint"
)]
trait FprintDevice {
    fn claim(&self, username: &str) -> zbus::Result<()>;
    fn release(&self) -> zbus::Result<()>;
    fn verify_start(&self, finger: &str) -> zbus::Result<()>;
    fn verify_stop(&self) -> zbus::Result<()>;

    #[zbus(signal)]
    fn verify_status(&self, result: &str, done: bool) -> zbus::Result<()>;
}

struct FprintdDeviceGuard<'a> {
    device: &'a FprintDeviceProxy<'a>,
}

impl<'a> Drop for FprintdDeviceGuard<'a> {
    fn drop(&mut self) {
        let _ = futures::executor::block_on(self.device.release());
    }
}

pub struct FingerprintAuth {
    config: crate::config::FingerprintConfig,
}

impl FingerprintAuth {
    pub fn new(config: crate::config::FingerprintConfig) -> Self {
        Self { config }
    }

    pub async fn fprintd_available() -> bool {
        let conn = match zbus::Connection::system().await {
            Ok(c) => c,
            Err(_) => return false,
        };
        let manager = match FprintManagerProxy::new(&conn).await {
            Ok(m) => m,
            Err(_) => return false,
        };
        manager.get_default_device().await.is_ok()
    }

    pub async fn authenticate_fingerprint(&self, username: &str) -> AuthResult {
        if self.config.mock_hardware {
            info!("Mocking fingerprint auth: Success");
            return AuthResult::Success(String::new());
        }

        let conn = match zbus::Connection::system().await {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to connect to system bus: {}", e);
                return AuthResult::Error(format!("D-Bus error: {}", e));
            }
        };

        let manager = match FprintManagerProxy::new(&conn).await {
            Ok(m) => m,
            Err(e) => return AuthResult::Error(format!("Failed to get fprintd manager: {}", e)),
        };

        let device_path = match manager.get_default_device().await {
            Ok(p) => p,
            Err(_) => {
                warn!("No fingerprint reader found. This is Linux.");
                return AuthResult::NotEnrolled("No fingerprint reader found".to_string());
            }
        };

        let device = match FprintDeviceProxy::builder(&conn)
            .path(device_path)
            .unwrap()
            .build()
            .await
        {
            Ok(d) => d,
            Err(e) => return AuthResult::Error(format!("Failed to connect to device: {}", e)),
        };

        if let Err(e) = device.claim(username).await {
            error!("Failed to claim fingerprint device for {}: {}", username, e);
            return AuthResult::Error(format!("Failed to claim device: {}", e));
        }

        let _guard = FprintdDeviceGuard { device: &device };

        let mut verify_stream = match device.receive_verify_status().await {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to subscribe to VerifyStatus signal: {}", e);
                return AuthResult::Error(format!("Failed to subscribe to VerifyStatus: {}", e));
            }
        };

        if let Err(e) = device.verify_start("any").await {
            error!("Failed to start verification: {}", e);
            return AuthResult::Error(format!("Failed to start verification: {}", e));
        }

        let mut verify_result = AuthResult::Failure("fingerprint_unknown_error".to_string());

        // Wait for VerifyStatus signal with a timeout
        let timeout_duration = Duration::from_secs(15);
        let result = timeout(timeout_duration, async {
            use zbus::StreamExt;
            while let Some(signal) = verify_stream.next().await {
                let args = match signal.args() {
                    Ok(a) => a,
                    Err(_) => continue,
                };

                let status_str = args.result();
                let done = args.done();

                info!("Fingerprint verify status: result={}, done={}", status_str, done);

                if done {
                    return match status_str {
                        "verify-match" => AuthResult::Success(String::new()),
                        "verify-no-match" => AuthResult::Failure("fingerprint_no_match".to_string()),
                        "verify-unknown-error" => AuthResult::Error("fprintd error".to_string()),
                        _ => AuthResult::Failure(format!("fingerprint_error_{}", status_str)),
                    };
                }
            }
            AuthResult::Failure("fingerprint_stream_ended".to_string())
        }).await;

        let _ = device.verify_stop().await;

        match result {
            Ok(res) => {
                verify_result = res;
            }
            Err(_) => {
                error!("Fingerprint verification timed out after {}s", timeout_duration.as_secs());
                verify_result = AuthResult::Failure("fingerprint_timeout".to_string());
            }
        }

        verify_result
    }

    fn run_pipeline(&self, username: &str) -> Result<AuthResult> {
        let rt = tokio::runtime::Runtime::new()?;
        Ok(rt.block_on(self.authenticate_fingerprint(username)))
    }
}

impl AuthMethod for FingerprintAuth {
    fn name(&self) -> &'static str {
        "fingerprint"
    }

    fn authenticate(&self, username: &str) -> Result<bool> {
        match self.run_pipeline(username)? {
            AuthResult::Success(_) => Ok(true),
            AuthResult::Failure(reason) => {
                info!("Fingerprint Auth Failed: {}", reason);
                Ok(false)
            },
            AuthResult::NotEnrolled(msg) => {
                info!("Fingerprint Auth Not Enrolled: {}", msg);
                Ok(false)
            },
            AuthResult::Error(e) => {
                error!("Fingerprint Auth Error: {}", e);
                Ok(false)
            },
        }
    }

    fn authenticate_result(&self, username: &str) -> Result<AuthResult> {
        self.run_pipeline(username)
    }

    fn enroll(&mut self, username: &str) -> Result<bool> {
        if self.config.mock_hardware {
            info!("Mocking fingerprint enroll: Success");
            return Ok(true);
        }

        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            crate::enrollment::fingerprint::enroll_fingerprint(username).await.is_ok()
        })
    }
}
