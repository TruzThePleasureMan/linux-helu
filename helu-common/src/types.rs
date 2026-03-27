use serde::{Deserialize, Serialize};
use zbus::zvariant::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub enum AuthMethod {
    Face,
    Fingerprint,
    Fido2,
    Pin,
    Auto,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AuthState {
    pub state: String, // "Idle", "Scanning", "Success", "Failure", "Fallback", "FidoPrompt"
    pub reason: String, // empty if none
    pub retry_count: u32, // 0 if none
}
