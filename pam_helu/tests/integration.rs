use zbus::Connection;
use helu_common::dbus::AuthProxy;
use std::time::Duration;
use tokio::time::sleep;

// A simple mock struct for helud auth proxy
struct MockHeluAuth;

#[zbus::interface(name = "net.helu.Auth")]
impl MockHeluAuth {
    async fn authenticate(&self, username: String, method: String) -> (bool, String) {
        if method == "pin" && username == "testuser" {
            (true, "Mock PIN auth success".to_string())
        } else {
            (false, "Mock auth failed".to_string())
        }
    }

    async fn authenticate_with_credential(
        &self,
        username: String,
        method: String,
        credential: String,
    ) -> (bool, String) {
        if method == "pin" && credential == "1234" && username == "testuser" {
            (true, "Mock PIN credential auth success".to_string())
        } else {
            (false, "Mock auth failed".to_string())
        }
    }

    async fn enroll(&self, _username: String, _method: String) -> bool {
        false
    }

    async fn list_methods(&self, _username: String) -> Vec<String> {
        vec!["pin".to_string()]
    }

    async fn status(&self) -> (String, Vec<String>) {
        ("test".to_string(), vec!["pin".to_string()])
    }

    #[zbus(signal)]
    async fn auth_requested(
        ctxt: &zbus::SignalContext<'_>,
        username: &str,
        method: &str,
    ) -> zbus::Result<()> {
        Ok(())
    }

    #[zbus(signal)]
    async fn auth_success(
        ctxt: &zbus::SignalContext<'_>,
        username: &str,
        method: &str,
    ) -> zbus::Result<()> {
        Ok(())
    }

    #[zbus(signal)]
    async fn auth_failure(
        ctxt: &zbus::SignalContext<'_>,
        username: &str,
        reason: &str,
    ) -> zbus::Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn test_dbus_fallback_logic() {
    // 1. Setup mock D-Bus connection on session bus
    let conn = Connection::session().await.unwrap();

    // Register the mock interface
    conn.object_server()
        .at("/net/helu/Auth", MockHeluAuth)
        .await
        .unwrap();

    conn.request_name("net.helu.Auth").await.unwrap();

    // Let D-Bus propagate name
    sleep(Duration::from_millis(50)).await;

    // 2. Test call_authenticate_with_credential logic from fallback.rs
    let proxy = AuthProxy::new(&conn).await.unwrap();

    let (success, msg) = proxy.authenticate_with_credential("testuser", "pin", "1234").await.unwrap();
    assert!(success, "Authentication should succeed for testuser/1234");
    assert_eq!(msg, "Mock PIN credential auth success");

    let (success, _) = proxy.authenticate_with_credential("testuser", "pin", "wrong").await.unwrap();
    assert!(!success, "Authentication should fail for testuser/wrong");

    let (success, _) = proxy.authenticate_with_credential("otheruser", "pin", "1234").await.unwrap();
    assert!(!success, "Authentication should fail for otheruser/1234");
}
