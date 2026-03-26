use super::AuthMethod;
use anyhow::Result;

pub struct FingerprintAuth {
    config: crate::config::FingerprintConfig,
}

impl FingerprintAuth {
    pub fn new(config: crate::config::FingerprintConfig) -> Self {
        Self { config }
    }
}

impl AuthMethod for FingerprintAuth {
    fn name(&self) -> &'static str {
        "fingerprint"
    }

    fn authenticate(&self, _username: &str) -> Result<bool> {
        if self.config.mock_hardware {
            tracing::info!("Mocking fingerprint auth: Success");
            return Ok(true);
        }
        Ok(false)
    }

    fn enroll(&mut self, _username: &str) -> Result<bool> {
        if self.config.mock_hardware {
            tracing::info!("Mocking fingerprint enroll: Success");
            return Ok(true);
        }
        Ok(false)
    }
}
