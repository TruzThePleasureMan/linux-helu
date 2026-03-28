use anyhow::Result;
use zbus::proxy;
use futures::StreamExt;
use tracing::info;

#[proxy(
    interface = "net.reactivated.Fprint.Manager",
    default_service = "net.reactivated.Fprint",
    default_path = "/net/reactivated/Fprint/Manager"
)]
trait FprintManager {
    fn get_default_device(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;
}

#[proxy(
    interface = "net.reactivated.Fprint.Device",
    default_service = "net.reactivated.Fprint"
)]
trait FprintDevice {
    fn claim(&self, username: &str) -> zbus::Result<()>;
    fn release(&self) -> zbus::Result<()>;
    fn enroll_start(&self, finger: &str) -> zbus::Result<()>;
    fn enroll_stop(&self) -> zbus::Result<()>;

    #[zbus(signal)]
    fn enroll_status(&self, result: &str, done: bool) -> zbus::Result<()>;
}

// Reuse the guard pattern
struct FprintdDeviceGuard<'a> {
    device: &'a FprintDeviceProxy<'a>,
}

impl<'a> Drop for FprintdDeviceGuard<'a> {
    fn drop(&mut self) {
        let _ = futures::executor::block_on(self.device.release());
    }
}

pub async fn enroll_fingerprint(username: &str) -> Result<()> {
    let conn = zbus::Connection::system().await?;

    let manager = FprintManagerProxy::new(&conn).await?;
    let device_path = manager.get_default_device().await?;

    let device = FprintDeviceProxy::builder(&conn)
        .path(device_path)?
        .build()
        .await?;

    device.claim(username).await?;

    let _guard = FprintdDeviceGuard { device: &device };

    let mut enroll_stream = device.receive_enroll_status().await?;

    device.enroll_start("right-index-finger").await?;
    info!("Started fingerprint enrollment for {} (right-index-finger)", username);

    // Assume 5 total stages as a default since fprintd might not emit total easily in basic API
    let total_stages = 5;
    let mut current_stage = 0;

    while let Some(signal) = enroll_stream.next().await {
        let args: EnrollStatusArgs = match signal.args() {
            Ok(a) => a,
            Err(_) => continue,
        };

        let status_str = args.result();
        let done = args.done();

        info!("Fingerprint enroll status: result={}, done={}", status_str, done);

        match *status_str {
            "enroll-stage-passed" => {
                current_stage += 1;
                // Emit D-Bus signal on session bus for UI
                if let Ok(session_conn) = zbus::Connection::session().await {
                    // Match the method signature added to DBus
                    let _ = session_conn.emit_signal(
                        Option::<&str>::None,
                        "/net/helu/Auth",
                        "net.helu.Auth",
                        "EnrollProgress",
                        &(username, current_stage, total_stages)
                    ).await;
                }
            }
            "enroll-completed" => {
                let _ = device.enroll_stop().await;
                return Ok(());
            }
            "enroll-failed" | "enroll-unknown-error" => {
                let _ = device.enroll_stop().await;
                anyhow::bail!("Fingerprint enrollment failed: {}", status_str);
            }
            _ => {}
        }

        if *done {
            break;
        }
    }

    let _ = device.enroll_stop().await;
    anyhow::bail!("Fingerprint enrollment stream ended unexpectedly")
}
