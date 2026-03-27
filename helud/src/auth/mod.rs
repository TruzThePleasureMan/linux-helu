pub mod face;
pub mod fingerprint;
pub mod pin;
pub mod fido2;

use anyhow::Result;
use std::collections::HashMap;
use tracing::info;

use helu_common::types::AuthResult;

pub trait AuthMethod {
    fn name(&self) -> &'static str;
    fn authenticate(&self, username: &str) -> Result<bool>;
    #[allow(dead_code)]
    fn authenticate_result(&self, username: &str) -> Result<AuthResult> {
        match self.authenticate(username) {
            Ok(true) => Ok(AuthResult::Success(String::new())),
            Ok(false) => Ok(AuthResult::Failure("Authentication failed".to_string())),
            Err(e) => Ok(AuthResult::Error(e.to_string())),
        }
    }
    fn authenticate_with_credential(&self, username: &str, _credential: &str) -> Result<bool> {
        // By default, fall back to standard authenticate if credential is not expected
        self.authenticate(username)
    }
    fn enroll(&mut self, username: &str) -> Result<bool>;
}

pub struct AuthManager {
    methods: HashMap<&'static str, Box<dyn AuthMethod + Send + Sync>>,
}

impl AuthManager {
    pub fn new() -> Self {
        Self {
            methods: HashMap::new(),
        }
    }

    pub fn register_method(&mut self, method: Box<dyn AuthMethod + Send + Sync>) {
        self.methods.insert(method.name(), method);
    }

    pub async fn authenticate(&mut self, username: &str, method: &str) -> Result<bool> {
        if method == "auto" {
            // Try Face -> Fingerprint -> FIDO2 -> PIN
            if self.methods.get("face").is_some_and(|m| m.authenticate(username).unwrap_or(false)) { return Ok(true); }
            if self.methods.get("fingerprint").is_some_and(|m| m.authenticate(username).unwrap_or(false)) { return Ok(true); }
            if self.methods.get("fido2").is_some_and(|m| m.authenticate(username).unwrap_or(false)) { return Ok(true); }
            if self.methods.get("pin").is_some_and(|m| m.authenticate(username).unwrap_or(false)) { return Ok(true); }
            Ok(false)
        } else if let Some(m) = self.methods.get(method) {
            m.authenticate(username)
        } else {
            anyhow::bail!("Method not found")
        }
    }

    pub async fn authenticate_with_credential(&mut self, username: &str, method: &str, credential: &str) -> Result<bool> {
        info!("Authenticating {} via {} with credential", username, method);

        if let Some(m) = self.methods.get(method) {
            m.authenticate_with_credential(username, credential)
        } else {
            Err(anyhow::anyhow!("Unknown auth method: {}", method))
        }
    }

    pub async fn enroll(&mut self, username: &str, method: &str) -> Result<bool> {
        if let Some(m) = self.methods.get_mut(method) {
            m.enroll(username)
        } else {
            anyhow::bail!("Method not found")
        }
    }

    pub fn list_methods(&self, _username: &str) -> Vec<String> {
        self.methods.keys().map(|k| k.to_string()).collect()
    }

    pub fn loaded_methods(&self) -> Vec<String> {
        self.methods.keys().map(|k| k.to_string()).collect()
    }
}
