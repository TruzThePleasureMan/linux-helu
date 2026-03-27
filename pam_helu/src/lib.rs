pub mod dbus;
pub mod fallback;

use pamsm::{PamError, PamFlags, PamServiceModule, Pam, PamLibExt};
use tokio::runtime::Runtime;
use log::{info, error};
use crate::dbus::{helud_available, check_ui_ready, call_authenticate};
use crate::fallback::terminal_pin_fallback;

struct PamHelu;

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
        // Initialize syslog with LOG_AUTH
        syslog::init(syslog::Facility::LOG_AUTH, log::LevelFilter::Info, Some("pam_helu")).ok();

        info!("pam_helu auth attempt started");

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

        // 1. Check if helud is running on D-Bus
        let is_helud_running = rt.block_on(helud_available());
        if !is_helud_running {
            error!("helud not running, falling back to terminal PIN immediately");
            return terminal_pin_fallback(&pamh, &user);
        }

        // 2. Check UI readiness
        let is_ui_ready = rt.block_on(check_ui_ready());
        if !is_ui_ready {
            error!("UI not ready, falling back to terminal PIN");
            return terminal_pin_fallback(&pamh, &user);
        }

        info!("Calling helud: net.helu.Auth.Authenticate({}, auto)", user);

        // 3. Call helud authenticate with method "auto"
        let result = rt.block_on(call_authenticate(&user, "auto"));

        match result {
            Ok((true, msg)) => {
                info!("Helud auth success: {}", msg);
                PamError::SUCCESS
            }
            Ok((false, msg)) => {
                info!("Helud auth failed: {}", msg);
                PamError::AUTH_ERR
            }
            Err(e) => {
                error!("Helud D-Bus call failed: {}", e);
                PamError::AUTH_ERR
            }
        }
    }
}

pamsm::pam_module!(PamHelu);
