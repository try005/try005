use axum::{
    extract::{Path, Query, Json},
    http::StatusCode,
    response::Json as ResponseJson,
    routing::{get, post, put, delete},
    Router,
};
use kube::{Api, Client, CustomResource};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber;
use schemars::JsonSchema;

#[derive(Debug, thiserror::Error)]
enum AppError {
    #[error("Kubernetes error: {0}")]
    Kube(#[from] kube::Error),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_yaml::Error),
    #[error("Cluster not found: {0}")]
    NotFound(String),
    #[error("Invalid request: {0}")]
    BadRequest(String),
}

impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Kube(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
            AppError::Serde(err) => (StatusCode::BAD_REQUEST, err.to_string()),
        };
        
        let body = ResponseJson(serde_json::json!({
            "error": error_message
        }));
        
        (status, body).into_response()
    }
}

#[derive(CustomResource, Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[kube(group = "postgresql.cnpg.io", version = "v1", kind = "Cluster")]
#[kube(namespaced)]
pub struct ClusterSpec {
    pub instances: i32,
    pub postgresql: PostgreSQLConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bootstrap: Option<BootstrapConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage: Option<StorageConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monitoring: Option<MonitoringConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct PostgreSQLConfig {
    pub parameters: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct BootstrapConfig {
    #[serde(rename = "initdb")]
    pub initdb: Option<InitDBConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct InitDBConfig {
    pub database: String,
    pub owner: String,
    pub secret: SecretConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct SecretConfig {
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct StorageConfig {
    pub size: String,
    #[serde(rename = "storageClass")]
    pub storage_class: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct MonitoringConfig {
    #[serde(rename = "enablePodMonitor")]
    pub enable_pod_monitor: bool,
    #[serde(rename = "disableDefaultQueries")]
    pub disable_default_queries: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateClusterRequest {
    pub name: String,
    pub namespace: Option<String>,
    pub instances: i32,
    pub database_name: String,
    pub database_owner: String,
    pub secret_name: String,
    pub storage_size: String,
    pub storage_class: Option<String>,
    pub postgresql_parameters: Option<HashMap<String, String>>,
    pub monitoring_enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateClusterRequest {
    pub instances: Option<i32>,
    pub postgresql_parameters: Option<HashMap<String, String>>,
    pub monitoring_enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub namespace: Option<String>,
}

type Result<T> = std::result::Result<T, AppError>;

async fn create_cluster(Json(payload): Json<CreateClusterRequest>) -> Result<ResponseJson<serde_json::Value>> {
    let client = Client::try_default().await?;
    let namespace = payload.namespace.as_deref().unwrap_or("default");
    
    let cluster_spec = ClusterSpec {
        instances: payload.instances,
        postgresql: PostgreSQLConfig {
            parameters: payload.postgresql_parameters.unwrap_or_default(),
        },
        bootstrap: Some(BootstrapConfig {
            initdb: Some(InitDBConfig {
                database: payload.database_name,
                owner: payload.database_owner,
                secret: SecretConfig {
                    name: payload.secret_name,
                },
            }),
        }),
        storage: Some(StorageConfig {
            size: payload.storage_size,
            storage_class: payload.storage_class,
        }),
        monitoring: payload.monitoring_enabled.map(|enabled| MonitoringConfig {
            enable_pod_monitor: enabled,
            disable_default_queries: false,
        }),
    };
    
    let cluster = Cluster {
        metadata: ObjectMeta {
            name: Some(payload.name.clone()),
            namespace: Some(namespace.to_string()),
            ..Default::default()
        },
        spec: cluster_spec,
    };
    
    let clusters: Api<Cluster> = Api::namespaced(client, namespace);
    let created = clusters.create(&Default::default(), &cluster).await?;
    
    Ok(ResponseJson(serde_json::json!({
        "message": "Cluster created successfully",
        "name": created.metadata.name,
        "namespace": created.metadata.namespace
    })))
}

async fn get_cluster(Path((namespace, name)): Path<(String, String)>) -> Result<ResponseJson<Cluster>> {
    let client = Client::try_default().await?;
    let clusters: Api<Cluster> = Api::namespaced(client, &namespace);
    
    match clusters.get(&name).await {
        Ok(cluster) => Ok(ResponseJson(cluster)),
        Err(kube::Error::Api(err)) if err.code == 404 => {
            Err(AppError::NotFound(format!("Cluster '{}' not found in namespace '{}'", name, namespace)))
        }
        Err(e) => Err(AppError::Kube(e)),
    }
}

async fn list_clusters(Query(params): Query<ListQuery>) -> Result<ResponseJson<serde_json::Value>> {
    let client = Client::try_default().await?;
    let namespace = params.namespace.as_deref().unwrap_or("default");
    let clusters: Api<Cluster> = Api::namespaced(client, namespace);
    
    let cluster_list = clusters.list(&Default::default()).await?;
    
    let clusters_info: Vec<serde_json::Value> = cluster_list
        .items
        .iter()
        .map(|cluster| {
            serde_json::json!({
                "name": cluster.metadata.name,
                "namespace": cluster.metadata.namespace,
                "instances": cluster.spec.instances,
                "creation_timestamp": cluster.metadata.creation_timestamp
            })
        })
        .collect();
    
    Ok(ResponseJson(serde_json::json!({
        "clusters": clusters_info,
        "count": clusters_info.len()
    })))
}

async fn update_cluster(
    Path((namespace, name)): Path<(String, String)>,
    Json(payload): Json<UpdateClusterRequest>,
) -> Result<ResponseJson<serde_json::Value>> {
    let client = Client::try_default().await?;
    let clusters: Api<Cluster> = Api::namespaced(client, &namespace);
    
    let mut cluster = match clusters.get(&name).await {
        Ok(cluster) => cluster,
        Err(kube::Error::Api(err)) if err.code == 404 => {
            return Err(AppError::NotFound(format!(
                "Cluster '{}' not found in namespace '{}'",
                name, namespace
            )));
        }
        Err(e) => return Err(AppError::Kube(e)),
    };
    
    if let Some(instances) = payload.instances {
        cluster.spec.instances = instances;
    }
    
    if let Some(parameters) = payload.postgresql_parameters {
        cluster.spec.postgresql.parameters = parameters;
    }
    
    if let Some(monitoring_enabled) = payload.monitoring_enabled {
        cluster.spec.monitoring = Some(MonitoringConfig {
            enable_pod_monitor: monitoring_enabled,
            disable_default_queries: false,
        });
    }
    
    let updated = clusters.replace(&name, &Default::default(), &cluster).await?;
    
    Ok(ResponseJson(serde_json::json!({
        "message": "Cluster updated successfully",
        "name": updated.metadata.name,
        "namespace": updated.metadata.namespace
    })))
}

async fn delete_cluster(Path((namespace, name)): Path<(String, String)>) -> Result<ResponseJson<serde_json::Value>> {
    let client = Client::try_default().await?;
    let clusters: Api<Cluster> = Api::namespaced(client, &namespace);
    
    match clusters.delete(&name, &Default::default()).await {
        Ok(_) => Ok(ResponseJson(serde_json::json!({
            "message": format!("Cluster '{}' deleted successfully", name)
        }))),
        Err(kube::Error::Api(err)) if err.code == 404 => {
            Err(AppError::NotFound(format!(
                "Cluster '{}' not found in namespace '{}'",
                name, namespace
            )))
        }
        Err(e) => Err(AppError::Kube(e)),
    }
}

async fn health_check() -> ResponseJson<serde_json::Value> {
    ResponseJson(serde_json::json!({
        "status": "healthy",
        "service": "cnpg-microservice"
    }))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/clusters", post(create_cluster))
        .route("/clusters", get(list_clusters))
        .route("/clusters/:namespace/:name", get(get_cluster))
        .route("/clusters/:namespace/:name", put(update_cluster))
        .route("/clusters/:namespace/:name", delete(delete_cluster))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    
    tracing::info!("CNPG Microservice listening on {}", listener.local_addr().unwrap());
    
    axum::serve(listener, app).await.unwrap();
}
