mod error;
mod handlers;
mod models;
mod resources;
mod utils;

use axum::{
    routing::{delete, get, post, put},
    Router,
};
use handlers::{cnpg, health, kubeflow};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber;
use tokio::signal;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing with error handling
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"))
        )
        .try_init()
        .map_err(|e| format!("Failed to initialize tracing: {}", e))?;
    
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
        
        // Kubeflow routes
        .route("/kubeflow/notebooks", post(kubeflow::create_notebook))
        .route("/kubeflow/notebooks", get(kubeflow::list_notebooks))
        .route("/kubeflow/notebooks/:namespace/:name", get(kubeflow::get_notebook))
        .route("/kubeflow/notebooks/:namespace/:name", put(kubeflow::update_notebook))
        .route("/kubeflow/notebooks/:namespace/:name", delete(kubeflow::delete_notebook))
        
        // Future routes will be added here:
        // .route("/kubevirt/vms", post(kubevirt::create_vm))
        // .route("/strimzi/kafka", post(strimzi::create_kafka))
        // .route("/cluster-api/clusters", post(capi::create_cluster))
        
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());
    
    // Bind to the specified address with proper error handling
    let bind_addr = std::env::var("BIND_ADDRESS").unwrap_or_else(|_| "0.0.0.0:3000".to_string());
    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .map_err(|e| format!("Failed to bind to address '{}': {}", bind_addr, e))?;
    
    // Get the actual listening address safely
    let local_addr = listener
        .local_addr()
        .map_err(|e| format!("Failed to get local address: {}", e))?;
    
    tracing::info!("K8s Resource Manager listening on {}", local_addr);
    tracing::info!("API endpoints:");
    tracing::info!("  Health: GET /health");
    tracing::info!("  CNPG Clusters: /cnpg/clusters");
    tracing::info!("  Kubeflow Notebooks: /kubeflow/notebooks");
    tracing::info!("  Legacy CNPG: /clusters (deprecated)");
    
    // Start the server with graceful shutdown
    tracing::info!("Starting server...");
    
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| format!("Server error: {}", e))?;
    
    tracing::info!("Server shutdown complete");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        match signal::ctrl_c().await {
            Ok(()) => tracing::info!("Received Ctrl+C, initiating graceful shutdown"),
            Err(e) => tracing::error!("Failed to listen for Ctrl+C signal: {}", e),
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match signal::unix::signal(signal::unix::SignalKind::terminate()) {
            Ok(mut stream) => {
                stream.recv().await;
                tracing::info!("Received SIGTERM, initiating graceful shutdown");
            }
            Err(e) => tracing::error!("Failed to install SIGTERM handler: {}", e),
        }
    };

    #[cfg(not(unix))]
    let terminate = async {
        // On non-Unix systems, just wait indefinitely
        std::future::pending::<()>().await;
    };

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}