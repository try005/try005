mod error;
mod handlers;
mod models;
mod resources;
mod utils;

use axum::{
    routing::{delete, get, post, put},
    Router,
};
use handlers::{cnpg, health};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    
    let app = Router::new()
        // Health check
        .route("/health", get(health::health_check))
        
        // CNPG routes
        .route("/cnpg/clusters", post(cnpg::create_cluster))
        .route("/cnpg/clusters", get(cnpg::list_clusters))
        .route("/cnpg/clusters/:namespace/:name", get(cnpg::get_cluster))
        .route("/cnpg/clusters/:namespace/:name", put(cnpg::update_cluster))
        .route("/cnpg/clusters/:namespace/:name", delete(cnpg::delete_cluster))
        
        // Legacy routes for backward compatibility (will be deprecated)
        .route("/clusters", post(cnpg::create_cluster))
        .route("/clusters", get(cnpg::list_clusters))
        .route("/clusters/:namespace/:name", get(cnpg::get_cluster))
        .route("/clusters/:namespace/:name", put(cnpg::update_cluster))
        .route("/clusters/:namespace/:name", delete(cnpg::delete_cluster))
        
        // Future routes will be added here:
        // .route("/kubevirt/vms", post(kubevirt::create_vm))
        // .route("/strimzi/kafka", post(strimzi::create_kafka))
        // .route("/cluster-api/clusters", post(capi::create_cluster))
        
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    
    tracing::info!("K8s Resource Manager listening on {}", listener.local_addr().unwrap());
    tracing::info!("API endpoints:");
    tracing::info!("  Health: GET /health");
    tracing::info!("  CNPG Clusters: /cnpg/clusters");
    tracing::info!("  Legacy CNPG: /clusters (deprecated)");
    
    axum::serve(listener, app).await.unwrap();
}