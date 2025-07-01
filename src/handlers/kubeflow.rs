use crate::error::{AppError, Result};
use crate::models::kubeflow::{CreateNotebookRequest, UpdateNotebookRequest};
use crate::models::ListQuery;
use crate::resources::kubeflow::KubeflowManager;
use crate::resources::ResourceManager;
use crate::utils::validation;
use axum::{
    extract::{Json, Path, Query},
    response::Json as ResponseJson,
};
use kube::Client;
use serde_json::Value;

pub async fn create_notebook(Json(payload): Json<CreateNotebookRequest>) -> Result<ResponseJson<Value>> {
    // Validate input
    validation::validate_resource_name(&payload.name)?;
    validation::validate_image_name(&payload.image)?;
    
    if let Some(ref namespace) = payload.namespace {
        validation::validate_namespace(namespace)?;
    }
    
    if let Some(ref cpu_request) = payload.cpu_request {
        validation::validate_cpu_resource(cpu_request)?;
    }
    
    if let Some(ref cpu_limit) = payload.cpu_limit {
        validation::validate_cpu_resource(cpu_limit)?;
    }
    
    if let Some(ref memory_request) = payload.memory_request {
        validation::validate_memory_resource(memory_request)?;
    }
    
    if let Some(ref memory_limit) = payload.memory_limit {
        validation::validate_memory_resource(memory_limit)?;
    }
    
    if let Some(ref workspace_size) = payload.workspace_volume_size {
        validation::validate_storage_size(workspace_size)?;
    }
    
    let client = Client::try_default()
        .await
        .map_err(|e| AppError::Config(format!("Failed to create Kubernetes client: {}", e)))?;
    
    let manager = KubeflowManager;
    let result = manager.create(client, payload).await?;
    
    tracing::info!(notebook_name = result.get("metadata").and_then(|m| m.get("name")).and_then(|n| n.as_str()).unwrap_or("unknown"), "Kubeflow notebook created successfully");
    
    Ok(ResponseJson(result))
}

pub async fn get_notebook(Path((namespace, name)): Path<(String, String)>) -> Result<ResponseJson<Value>> {
    // Validate input
    validation::validate_namespace(&namespace)?;
    validation::validate_resource_name(&name)?;
    
    let client = Client::try_default()
        .await
        .map_err(|e| AppError::Config(format!("Failed to create Kubernetes client: {}", e)))?;
    
    let manager = KubeflowManager;
    let notebook = manager.get(client, &namespace, &name).await?;
    
    Ok(ResponseJson(serde_json::to_value(notebook).map_err(|e| {
        AppError::Internal(format!("Failed to serialize notebook: {}", e))
    })?))
}

pub async fn list_notebooks(Query(params): Query<ListQuery>) -> Result<ResponseJson<Value>> {
    let namespace = params.namespace.as_deref().unwrap_or("default");
    
    // Validate namespace if provided
    if params.namespace.is_some() {
        validation::validate_namespace(namespace)?;
    }
    
    let client = Client::try_default()
        .await
        .map_err(|e| AppError::Config(format!("Failed to create Kubernetes client: {}", e)))?;
    
    let manager = KubeflowManager;
    let result = manager.list(client, namespace).await?;
    
    Ok(ResponseJson(result))
}

pub async fn update_notebook(
    Path((namespace, name)): Path<(String, String)>,
    Json(payload): Json<UpdateNotebookRequest>,
) -> Result<ResponseJson<Value>> {
    // Validate input
    validation::validate_namespace(&namespace)?;
    validation::validate_resource_name(&name)?;
    
    if let Some(ref image) = payload.image {
        validation::validate_image_name(image)?;
    }
    
    if let Some(ref cpu_request) = payload.cpu_request {
        validation::validate_cpu_resource(cpu_request)?;
    }
    
    if let Some(ref cpu_limit) = payload.cpu_limit {
        validation::validate_cpu_resource(cpu_limit)?;
    }
    
    if let Some(ref memory_request) = payload.memory_request {
        validation::validate_memory_resource(memory_request)?;
    }
    
    if let Some(ref memory_limit) = payload.memory_limit {
        validation::validate_memory_resource(memory_limit)?;
    }
    
    let client = Client::try_default()
        .await
        .map_err(|e| AppError::Config(format!("Failed to create Kubernetes client: {}", e)))?;
    
    let manager = KubeflowManager;
    let result = manager.update(client, &namespace, &name, payload).await?;
    
    tracing::info!(notebook_name = name, namespace = namespace, "Kubeflow notebook updated successfully");
    
    Ok(ResponseJson(result))
}

pub async fn delete_notebook(Path((namespace, name)): Path<(String, String)>) -> Result<ResponseJson<Value>> {
    // Validate input
    validation::validate_namespace(&namespace)?;
    validation::validate_resource_name(&name)?;
    
    let client = Client::try_default()
        .await
        .map_err(|e| AppError::Config(format!("Failed to create Kubernetes client: {}", e)))?;
    
    let manager = KubeflowManager;
    let result = manager.delete(client, &namespace, &name).await?;
    
    tracing::info!(notebook_name = name, namespace = namespace, "Kubeflow notebook deleted successfully");
    
    Ok(ResponseJson(result))
}