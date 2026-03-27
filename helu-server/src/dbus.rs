use zbus::{Connection, Result as ZbusResult};
use helu_common::dbus::AuthProxy;
use tracing::debug;

pub struct HeluDbusClient {
    connection: Connection,
}

impl HeluDbusClient {
    pub async fn new(bus_type: &str) -> ZbusResult<Self> {
        let connection = if bus_type == "system" {
            Connection::system().await?
        } else {
            Connection::session().await?
        };

        Ok(Self { connection })
    }

    pub async fn trigger_auth(&self, username: &str, method: &str) -> anyhow::Result<(bool, String)> {
        debug!("Triggering D-Bus auth for user '{}' with method '{}'", username, method);
        let proxy = AuthProxy::new(&self.connection).await?;

        let result = proxy.authenticate(username, method).await?;

        Ok(result)
    }

    pub async fn trigger_auth_with_credential(&self, username: &str, method: &str, credential: &str) -> anyhow::Result<(bool, String)> {
        debug!("Triggering D-Bus auth with credential for user '{}' with method '{}'", username, method);
        let proxy = AuthProxy::new(&self.connection).await?;

        let result = proxy.authenticate_with_credential(username, method, credential).await?;

        Ok(result)
    }
}
