use kube::CustomResource;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::collections::HashMap;

#[derive(CustomResource, Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[kube(group = "kubeflow.org", version = "v1", kind = "Notebook")]
#[kube(namespaced)]
pub struct NotebookSpec {
    pub template: NotebookTemplate,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct NotebookTemplate {
    pub spec: NotebookPodSpec,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct NotebookPodSpec {
    pub containers: Vec<NotebookContainer>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volumes: Option<Vec<NotebookVolume>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "serviceAccountName")]
    pub service_account_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct NotebookContainer {
    pub name: String,
    pub image: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<NotebookResources>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<Vec<NotebookEnvVar>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "volumeMounts")]
    pub volume_mounts: Option<Vec<NotebookVolumeMount>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ports: Option<Vec<NotebookPort>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct NotebookResources {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requests: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limits: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct NotebookEnvVar {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct NotebookVolume {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "persistentVolumeClaim")]
    pub persistent_volume_claim: Option<NotebookPvcSource>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "emptyDir")]
    pub empty_dir: Option<NotebookEmptyDirSource>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct NotebookPvcSource {
    #[serde(rename = "claimName")]
    pub claim_name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct NotebookEmptyDirSource {}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct NotebookVolumeMount {
    pub name: String,
    #[serde(rename = "mountPath")]
    pub mount_path: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct NotebookPort {
    #[serde(rename = "containerPort")]
    pub container_port: i32,
    pub name: String,
    pub protocol: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateNotebookRequest {
    pub name: String,
    pub namespace: Option<String>,
    pub image: String,
    pub cpu_request: Option<String>,
    pub cpu_limit: Option<String>,
    pub memory_request: Option<String>,
    pub memory_limit: Option<String>,
    pub gpu_limit: Option<String>,
    pub workspace_volume_size: Option<String>,
    pub workspace_volume_mount: Option<String>,
    pub environment_variables: Option<HashMap<String, String>>,
    pub service_account: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateNotebookRequest {
    pub image: Option<String>,
    pub cpu_request: Option<String>,
    pub cpu_limit: Option<String>,
    pub memory_request: Option<String>,
    pub memory_limit: Option<String>,
    pub gpu_limit: Option<String>,
    pub environment_variables: Option<HashMap<String, String>>,
}