use super::AuthMethod;
use anyhow::Result;

pub struct Fido2Auth {
    config: crate::config::Fido2Config,
}

impl Fido2Auth {
    pub fn new(config: crate::config::Fido2Config) -> Self {
        Self { config }
    }
}

impl AuthMethod for Fido2Auth {
    fn name(&self) -> &'static str {
        "fido2"
    }

    fn authenticate(&self, _username: &str) -> Result<bool> {
        if self.config.mock_hardware {
            tracing::info!("Mocking FIDO2 auth: Success");
            return Ok(true);
        }
        Ok(false)
    }

    fn enroll(&mut self, _username: &str) -> Result<bool> {
        if self.config.mock_hardware {
            tracing::info!("Mocking FIDO2 enroll: Success");
            return Ok(true);
        }
        Ok(false)
    }
}
