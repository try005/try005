use crate::error::{AppError, Result};
use crate::models::cnpg::*;
use crate::resources::ResourceManager;
use async_trait::async_trait;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::{Api, Client};
use serde_json::{json, Value};

pub struct CnpgManager;

#[async_trait]
impl ResourceManager for CnpgManager {
    type CreateRequest = CreateClusterRequest;
    type UpdateRequest = UpdateClusterRequest;
    type Resource = Cluster;

    async fn create(&self, client: Client, request: Self::CreateRequest) -> Result<Value> {
        let namespace = request.namespace.as_deref().unwrap_or("default");
        
        let cluster_spec = ClusterSpec {
            instances: request.instances,
            postgresql: PostgreSQLConfig {
                parameters: request.postgresql_parameters.unwrap_or_default(),
            },
            bootstrap: Some(BootstrapConfig {
                initdb: Some(InitDBConfig {
                    database: request.database_name,
                    owner: request.database_owner,
                    secret: SecretConfig {
                        name: request.secret_name,
                    },
                }),
            }),
            storage: Some(StorageConfig {
                size: request.storage_size,
                storage_class: request.storage_class,
            }),
            monitoring: request.monitoring_enabled.map(|enabled| MonitoringConfig {
                enable_pod_monitor: enabled,
                disable_default_queries: false,
            }),
        };
        
        let cluster = Cluster {
            metadata: ObjectMeta {
                name: Some(request.name.clone()),
                namespace: Some(namespace.to_string()),
                ..Default::default()
            },
            spec: cluster_spec,
        };
        
        let clusters: Api<Cluster> = Api::namespaced(client, namespace);
        let created = clusters.create(&Default::default(), &cluster).await?;
        
        Ok(json!({
            "message": "CNPG cluster created successfully",
            "name": created.metadata.name,
            "namespace": created.metadata.namespace,
            "resource_type": "cnpg-cluster"
        }))
    }

    async fn get(&self, client: Client, namespace: &str, name: &str) -> Result<Self::Resource> {
        let clusters: Api<Cluster> = Api::namespaced(client, namespace);
        
        match clusters.get(name).await {
            Ok(cluster) => Ok(cluster),
            Err(kube::Error::Api(err)) if err.code == 404 => {
                Err(AppError::NotFound(format!(
                    "CNPG cluster '{}' not found in namespace '{}'",
                    name, namespace
                )))
            }
            Err(e) => Err(AppError::Kube(e)),
        }
    }

    async fn list(&self, client: Client, namespace: &str) -> Result<Value> {
        let clusters: Api<Cluster> = Api::namespaced(client, namespace);
        let cluster_list = clusters.list(&Default::default()).await?;
        
        let clusters_info: Vec<Value> = cluster_list
            .items
            .iter()
            .map(|cluster| {
                json!({
                    "name": cluster.metadata.name,
                    "namespace": cluster.metadata.namespace,
                    "instances": cluster.spec.instances,
                    "creation_timestamp": cluster.metadata.creation_timestamp,
                    "resource_type": "cnpg-cluster"
                })
            })
            .collect();
        
        Ok(json!({
            "resources": clusters_info,
            "count": clusters_info.len(),
            "resource_type": "cnpg-clusters"
        }))
    }

    async fn update(
        &self,
        client: Client,
        namespace: &str,
        name: &str,
        request: Self::UpdateRequest,
    ) -> Result<Value> {
        let clusters: Api<Cluster> = Api::namespaced(client, namespace);
        
        let mut cluster = match clusters.get(name).await {
            Ok(cluster) => cluster,
            Err(kube::Error::Api(err)) if err.code == 404 => {
                return Err(AppError::NotFound(format!(
                    "CNPG cluster '{}' not found in namespace '{}'",
                    name, namespace
                )));
            }
            Err(e) => return Err(AppError::Kube(e)),
        };
        
        if let Some(instances) = request.instances {
            cluster.spec.instances = instances;
        }
        
        if let Some(parameters) = request.postgresql_parameters {
            cluster.spec.postgresql.parameters = parameters;
        }
        
        if let Some(monitoring_enabled) = request.monitoring_enabled {
            cluster.spec.monitoring = Some(MonitoringConfig {
                enable_pod_monitor: monitoring_enabled,
                disable_default_queries: false,
            });
        }
        
        let updated = clusters.replace(name, &Default::default(), &cluster).await?;
        
        Ok(json!({
            "message": "CNPG cluster updated successfully",
            "name": updated.metadata.name,
            "namespace": updated.metadata.namespace,
            "resource_type": "cnpg-cluster"
        }))
    }

    async fn delete(&self, client: Client, namespace: &str, name: &str) -> Result<Value> {
        let clusters: Api<Cluster> = Api::namespaced(client, namespace);
        
        match clusters.delete(name, &Default::default()).await {
            Ok(_) => Ok(json!({
                "message": format!("CNPG cluster '{}' deleted successfully", name),
                "resource_type": "cnpg-cluster"
            })),
            Err(kube::Error::Api(err)) if err.code == 404 => {
                Err(AppError::NotFound(format!(
                    "CNPG cluster '{}' not found in namespace '{}'",
                    name, namespace
                )))
            }
            Err(e) => Err(AppError::Kube(e)),
        }
    }
}