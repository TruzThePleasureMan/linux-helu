mod api;
mod config;
mod token;

use actix_web::{App, HttpServer, middleware};
use tracing::{info, Level};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    let conf = config::ServerConfig::load().unwrap_or_default();

    info!("Starting Helu Server on {}:{}", conf.bind_addr, conf.port);

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .service(api::auth::authenticate)
            .service(api::enroll::enroll)
            .service(api::admin::status)
    })
    .bind((conf.bind_addr.clone(), conf.port))?
    .run()
    .await
}
