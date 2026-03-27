use pamsm::{Pam, PamError, PamLibExt};
use log::{error, info};
use crate::dbus::call_authenticate_with_credential;
use tokio::runtime::Runtime;

pub fn terminal_pin_fallback(pamh: &Pam, username: &str) -> PamError {
    info!("Falling back to terminal PIN collection for {}", username);

    let pin = match pamh.get_authtok(Some("Helu PIN: ")) {
        Ok(Some(p)) => {
            match p.to_str() {
                Ok(s) => s.to_string(),
                Err(_) => {
                    error!("Invalid UTF-8 in PIN");
                    return PamError::AUTH_ERR;
                }
            }
        },
        Ok(None) => {
            error!("Empty PIN provided");
            return PamError::AUTH_ERR;
        },
        Err(e) => {
            error!("Failed to collect PIN: {:?}", e);
            return PamError::AUTH_ERR;
        }
    };

    let rt = match Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            error!("Failed to create tokio runtime for fallback: {}", e);
            return PamError::AUTH_ERR;
        }
    };

    match rt.block_on(call_authenticate_with_credential(username, "pin", &pin)) {
        Ok((true, msg)) => {
            info!("PIN authentication success: {}", msg);
            PamError::SUCCESS
        }
        Ok((false, msg)) => {
            info!("PIN authentication failed: {}", msg);
            PamError::AUTH_ERR
        }
        Err(e) => {
            error!("Error calling authenticate for PIN: {}", e);
            PamError::AUTHINFO_UNAVAIL
        }
    }
}
