use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use crate::state::AppState;

pub async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

pub async fn db_health(State(state): State<AppState>) -> impl IntoResponse {
    match state.driver.health().await {
        Ok(health) => {
            if health.is_alive {
                (StatusCode::OK, Json(health)).into_response()
            } else {
                (StatusCode::SERVICE_UNAVAILABLE, Json(health)).into_response()
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "status": "error",
                "error": e.to_string(),
            })),
        )
            .into_response(),
    }
}
