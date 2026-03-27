pub mod face;
pub mod fingerprint;
pub mod pin;
pub mod fido2;

use anyhow::Result;
use std::collections::HashMap;

pub trait AuthMethod {
    fn name(&self) -> &'static str;
    fn authenticate(&self, username: &str) -> Result<bool>;
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
            // Try Face -> Fingerprint -> PIN
            if self.methods.get("face").is_some_and(|m| m.authenticate(username).unwrap_or(false)) { return Ok(true); }
            if self.methods.get("fingerprint").is_some_and(|m| m.authenticate(username).unwrap_or(false)) { return Ok(true); }
            if self.methods.get("pin").is_some_and(|m| m.authenticate(username).unwrap_or(false)) { return Ok(true); }
            Ok(false)
        } else if let Some(m) = self.methods.get(method) {
            m.authenticate(username)
        } else {
            anyhow::bail!("Method not found")
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
