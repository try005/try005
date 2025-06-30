use axum::response::Json as ResponseJson;
use serde_json::{json, Value};

pub async fn health_check() -> ResponseJson<Value> {
    ResponseJson(json!({
        "status": "healthy",
        "service": "k8s-resource-manager",
        "version": "0.1.0"
    }))
}