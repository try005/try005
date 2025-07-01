use crate::error::{AppError, Result};
use crate::models::cnpg::{CreateClusterRequest, UpdateClusterRequest};
use crate::models::ListQuery;
use crate::resources::cnpg::CnpgManager;
use crate::resources::ResourceManager;
use crate::utils::validation;
use axum::{
    extract::{Json, Path, Query},
    response::Json as ResponseJson,
};
use kube::Client;
use serde_json::Value;

pub async fn create_cluster(Json(payload): Json<CreateClusterRequest>) -> Result<ResponseJson<Value>> {
    // Validate input
    validation::validate_resource_name(&payload.name)?;
    validation::validate_database_name(&payload.database_name)?;
    validation::validate_database_name(&payload.database_owner)?;
    validation::validate_instance_count(payload.instances)?;
    validation::validate_storage_size(&payload.storage_size)?;
    
    if let Some(ref namespace) = payload.namespace {
        validation::validate_namespace(namespace)?;
    }
    
    // Create Kubernetes client with timeout
    let client = Client::try_default()
        .await
        .map_err(|e| AppError::Config(format!("Failed to create Kubernetes client: {}", e)))?;
    
    let manager = CnpgManager;
    let result = manager.create(client, payload).await?;
    
    tracing::info!(cluster_name = result.get("metadata").and_then(|m| m.get("name")).and_then(|n| n.as_str()).unwrap_or("unknown"), "CNPG cluster created successfully");
    
    Ok(ResponseJson(result))
}

pub async fn get_cluster(Path((namespace, name)): Path<(String, String)>) -> Result<ResponseJson<Value>> {
    // Validate input
    validation::validate_namespace(&namespace)?;
    validation::validate_resource_name(&name)?;
    
    let client = Client::try_default()
        .await
        .map_err(|e| AppError::Config(format!("Failed to create Kubernetes client: {}", e)))?;
    
    let manager = CnpgManager;
    let cluster = manager.get(client, &namespace, &name).await?;
    
    Ok(ResponseJson(serde_json::to_value(cluster).map_err(|e| {
        AppError::Internal(format!("Failed to serialize cluster: {}", e))
    })?))
}

pub async fn list_clusters(Query(params): Query<ListQuery>) -> Result<ResponseJson<Value>> {
    let namespace = params.namespace.as_deref().unwrap_or("default");
    
    // Validate namespace if provided
    if params.namespace.is_some() {
        validation::validate_namespace(namespace)?;
    }
    
    let client = Client::try_default()
        .await
        .map_err(|e| AppError::Config(format!("Failed to create Kubernetes client: {}", e)))?;
    
    let manager = CnpgManager;
    let result = manager.list(client, namespace).await?;
    
    Ok(ResponseJson(result))
}

pub async fn update_cluster(
    Path((namespace, name)): Path<(String, String)>,
    Json(payload): Json<UpdateClusterRequest>,
) -> Result<ResponseJson<Value>> {
    // Validate input
    validation::validate_namespace(&namespace)?;
    validation::validate_resource_name(&name)?;
    
    if let Some(instances) = payload.instances {
        validation::validate_instance_count(instances)?;
    }
    
    let client = Client::try_default()
        .await
        .map_err(|e| AppError::Config(format!("Failed to create Kubernetes client: {}", e)))?;
    
    let manager = CnpgManager;
    let result = manager.update(client, &namespace, &name, payload).await?;
    
    tracing::info!(cluster_name = name, namespace = namespace, "CNPG cluster updated successfully");
    
    Ok(ResponseJson(result))
}

pub async fn delete_cluster(Path((namespace, name)): Path<(String, String)>) -> Result<ResponseJson<Value>> {
    // Validate input
    validation::validate_namespace(&namespace)?;
    validation::validate_resource_name(&name)?;
    
    let client = Client::try_default()
        .await
        .map_err(|e| AppError::Config(format!("Failed to create Kubernetes client: {}", e)))?;
    
    let manager = CnpgManager;
    let result = manager.delete(client, &namespace, &name).await?;
    
    tracing::info!(cluster_name = name, namespace = namespace, "CNPG cluster deleted successfully");
    
    Ok(ResponseJson(result))
}