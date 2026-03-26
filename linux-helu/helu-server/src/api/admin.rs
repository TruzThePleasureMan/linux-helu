use actix_web::{get, HttpResponse, Responder};

#[get("/admin/status")]
pub async fn status() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "running",
        "version": env!("CARGO_PKG_VERSION")
    }))
}
