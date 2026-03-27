use serde::{Deserialize, Serialize};
use zbus::zvariant::Type;

// zvariant does not support enums where some variants have fields and some don't.
// Making them all have an empty string to satisfy the Type macro if needed, or
// use a Struct instead. The easiest is making the empty ones have empty Strings.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub enum AuthResult {
    Success(String),
    Failure(String),
    NotEnrolled(String),
    Error(String),
}

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
    pub username: String, // Added to map DBus signals correctly
    pub state: String, // "Idle", "Scanning", "Success", "Failure", "Fallback", "FidoPrompt"
    pub reason: String, // empty if none
    pub retry_count: u32, // 0 if none
}
