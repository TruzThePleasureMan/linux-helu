use clap::{Parser, Subcommand};
use anyhow::Result;
use zbus::{Connection, proxy};

#[proxy(
    interface = "net.helu.Auth",
    default_service = "net.helu.Auth",
    default_path = "/net/helu/Auth"
)]
trait HeluAuth {
    async fn authenticate(&self, username: String, method: String) -> zbus::Result<(bool, String)>;
    async fn enroll(&self, username: String, method: String) -> zbus::Result<bool>;
    async fn list_methods(&self, username: String) -> zbus::Result<Vec<String>>;
    async fn status(&self) -> zbus::Result<(String, Vec<String>)>;
}

#[derive(Parser)]
#[command(version, about = "Linux Helu command line tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Enroll a new biometric method
    Enroll {
        method: String,
        #[arg(short, long)]
        username: Option<String>,
    },
    /// Test authentication
    Test {
        #[arg(short, long)]
        username: Option<String>,
        #[arg(short, long, default_value = "auto")]
        method: String,
    },
    /// Show daemon status
    Status,
    /// Who am I?
    Whoami {
        #[arg(long)]
        headless: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // First try system bus, then session bus as fallback (for dev)
    let conn_res = Connection::system().await;

    let conn = match conn_res {
        Ok(c) => c,
        Err(_) => Connection::session().await?
    };

    let proxy = HeluAuthProxy::new(&conn).await?;

    let current_user = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());

    match &cli.command {
        Commands::Enroll { method, username } => {
            let user = username.as_ref().unwrap_or(&current_user);
            println!("Enrolling {} for {}...", method, user);
            let success = proxy.enroll(user.to_string(), method.to_string()).await?;
            if success {
                println!("Successfully enrolled {} for {}", method, user);
            } else {
                println!("Failed to enroll {} for {}", method, user);
            }
        }
        Commands::Test { username, method } => {
            let user = username.as_ref().unwrap_or(&current_user);
            println!("Testing auth for {} via {}...", user, method);
            let (success, msg) = proxy.authenticate(user.to_string(), method.to_string()).await?;
            if success {
                println!("Success: {}", msg);
            } else {
                println!("Failed: {}", msg);
            }
        }
        Commands::Status => {
            match proxy.status().await {
                Ok((version, methods)) => {
                    println!("Helud Version: {}", version);
                    println!("Loaded Methods: {}", methods.join(", "));

                    if let Ok(user_methods) = proxy.list_methods(current_user.clone()).await {
                        println!("Enrolled methods for {}: {}", current_user, user_methods.join(", "));
                    }
                }
                Err(e) => {
                    println!("Error getting status (is helud running?): {}", e);
                }
            }
        }
        Commands::Whoami { headless: _ } => {
            println!("Helu, {}", current_user);
        }
    }

    Ok(())
}
