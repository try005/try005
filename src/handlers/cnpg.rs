use crate::error::Result;
use crate::models::cnpg::{CreateClusterRequest, UpdateClusterRequest};
use crate::models::ListQuery;
use crate::resources::cnpg::CnpgManager;
use crate::resources::ResourceManager;
use axum::{
    extract::{Json, Path, Query},
    response::Json as ResponseJson,
};
use kube::Client;
use serde_json::Value;

pub async fn create_cluster(Json(payload): Json<CreateClusterRequest>) -> Result<ResponseJson<Value>> {
    let client = Client::try_default().await?;
    let manager = CnpgManager;
    let result = manager.create(client, payload).await?;
    Ok(ResponseJson(result))
}

pub async fn get_cluster(Path((namespace, name)): Path<(String, String)>) -> Result<ResponseJson<Value>> {
    let client = Client::try_default().await?;
    let manager = CnpgManager;
    let cluster = manager.get(client, &namespace, &name).await?;
    Ok(ResponseJson(serde_json::to_value(cluster)?))
}

pub async fn list_clusters(Query(params): Query<ListQuery>) -> Result<ResponseJson<Value>> {
    let client = Client::try_default().await?;
    let namespace = params.namespace.as_deref().unwrap_or("default");
    let manager = CnpgManager;
    let result = manager.list(client, namespace).await?;
    Ok(ResponseJson(result))
}

pub async fn update_cluster(
    Path((namespace, name)): Path<(String, String)>,
    Json(payload): Json<UpdateClusterRequest>,
) -> Result<ResponseJson<Value>> {
    let client = Client::try_default().await?;
    let manager = CnpgManager;
    let result = manager.update(client, &namespace, &name, payload).await?;
    Ok(ResponseJson(result))
}

pub async fn delete_cluster(Path((namespace, name)): Path<(String, String)>) -> Result<ResponseJson<Value>> {
    let client = Client::try_default().await?;
    let manager = CnpgManager;
    let result = manager.delete(client, &namespace, &name).await?;
    Ok(ResponseJson(result))
}