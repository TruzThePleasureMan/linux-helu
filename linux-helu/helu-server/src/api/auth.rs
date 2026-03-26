use actix_web::{post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Deserialize)]
pub struct AuthRequest {
    pub username: String,
    pub token: Option<String>,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub success: bool,
    pub challenge: Option<String>,
    pub jwt: Option<String>,
    pub message: String,
}

#[post("/auth")]
pub async fn authenticate(req: web::Json<AuthRequest>) -> impl Responder {
    info!("Auth request for {}", req.username);

    // Skeleton implementation
    if req.token.is_none() {
        // Return a challenge
        return HttpResponse::Ok().json(AuthResponse {
            success: true,
            challenge: Some("dummy-challenge-1234".to_string()),
            jwt: None,
            message: "Challenge issued".to_string(),
        });
    }

    // Verify token
    HttpResponse::Ok().json(AuthResponse {
        success: true,
        challenge: None,
        jwt: Some("dummy-jwt-for-now".to_string()),
        message: "Authentication successful".to_string(),
    })
}
