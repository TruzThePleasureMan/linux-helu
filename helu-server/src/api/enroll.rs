use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
use tracing::info;

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct EnrollRequest {
    pub username: String,
    pub method: String,
    pub payload: String,
}

#[post("/enroll")]
pub async fn enroll(req: web::Json<EnrollRequest>) -> impl Responder {
    info!("Enrollment request for {} using {}", req.username, req.method);

    // Skeleton implementation
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": format!("Successfully enrolled {} for {}", req.method, req.username)
    }))
}
