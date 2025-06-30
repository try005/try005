use kube::CustomResource;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::collections::HashMap;

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