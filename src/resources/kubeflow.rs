use crate::error::{AppError, Result};
use crate::models::kubeflow::{
    CreateNotebookRequest, Notebook, NotebookContainer, NotebookEnvVar, NotebookPodSpec,
    NotebookPort, NotebookPvcSource, NotebookResources, NotebookSpec, NotebookTemplate,
    NotebookVolume, NotebookVolumeMount, UpdateNotebookRequest,
};
use crate::resources::ResourceManager;
use async_trait::async_trait;
use k8s_openapi::api::core::v1::PersistentVolumeClaim;
use kube::{
    api::{Api, ListParams, Patch, PatchParams},
    Client,
};
use serde_json::{json, Value};
use std::collections::HashMap;

pub struct KubeflowManager;

#[async_trait]
impl ResourceManager for KubeflowManager {
    type CreateRequest = CreateNotebookRequest;
    type UpdateRequest = UpdateNotebookRequest;
    type Resource = Notebook;

    async fn create(&self, client: Client, request: Self::CreateRequest) -> Result<Value> {
        let namespace = request.namespace.as_deref().unwrap_or("default");
        let api: Api<Notebook> = Api::namespaced(client.clone(), namespace);

        // Create PVC if workspace volume is requested
        if let Some(volume_size) = &request.workspace_volume_size {
            self.create_workspace_pvc(&client, namespace, &request.name, volume_size)
                .await?;
        }

        // Build notebook spec
        let notebook_spec = self.build_notebook_spec(&request)?;

        let notebook = Notebook::new(&request.name, notebook_spec);

        match api.create(&Default::default(), &notebook).await {
            Ok(created) => Ok(serde_json::to_value(created)?),
            Err(e) => Err(AppError::Kube(e)),
        }
    }

    async fn get(&self, client: Client, namespace: &str, name: &str) -> Result<Self::Resource> {
        let api: Api<Notebook> = Api::namespaced(client, namespace);

        match api.get(name).await {
            Ok(notebook) => Ok(notebook),
            Err(e) => Err(AppError::Kube(e)),
        }
    }

    async fn list(&self, client: Client, namespace: &str) -> Result<Value> {
        let api: Api<Notebook> = Api::namespaced(client, namespace);

        match api.list(&ListParams::default()).await {
            Ok(notebooks) => Ok(serde_json::to_value(notebooks)?),
            Err(e) => Err(AppError::Kube(e)),
        }
    }

    async fn update(
        &self,
        client: Client,
        namespace: &str,
        name: &str,
        request: Self::UpdateRequest,
    ) -> Result<Value> {
        let api: Api<Notebook> = Api::namespaced(client.clone(), namespace);

        // Get existing notebook
        let existing = self.get(client, namespace, name).await?;
        
        // Build updated spec
        let updated_spec = self.build_update_spec(&existing.spec, &request)?;
        
        let patch = json!({
            "spec": updated_spec
        });

        match api
            .patch(name, &PatchParams::default(), &Patch::Merge(patch))
            .await
        {
            Ok(updated) => Ok(serde_json::to_value(updated)?),
            Err(e) => Err(AppError::Kube(e)),
        }
    }

    async fn delete(&self, client: Client, namespace: &str, name: &str) -> Result<Value> {
        let api: Api<Notebook> = Api::namespaced(client.clone(), namespace);

        match api.delete(name, &Default::default()).await {
            Ok(_result) => {
                // Also delete the workspace PVC if it exists
                let _ = self.delete_workspace_pvc(&client, namespace, name).await;
                Ok(serde_json::json!({
                    "status": "deleted",
                    "name": name,
                    "namespace": namespace
                }))
            }
            Err(e) => Err(AppError::Kube(e)),
        }
    }
}

impl KubeflowManager {
    fn build_notebook_spec(&self, request: &CreateNotebookRequest) -> Result<NotebookSpec> {
        let mut limits = HashMap::new();
        let mut requests = HashMap::new();

        // Set CPU resources
        if let Some(cpu_request) = &request.cpu_request {
            requests.insert("cpu".to_string(), cpu_request.clone());
        }
        if let Some(cpu_limit) = &request.cpu_limit {
            limits.insert("cpu".to_string(), cpu_limit.clone());
        }

        // Set memory resources
        if let Some(memory_request) = &request.memory_request {
            requests.insert("memory".to_string(), memory_request.clone());
        }
        if let Some(memory_limit) = &request.memory_limit {
            limits.insert("memory".to_string(), memory_limit.clone());
        }

        // Set GPU resources
        if let Some(gpu_limit) = &request.gpu_limit {
            limits.insert("nvidia.com/gpu".to_string(), gpu_limit.clone());
        }

        let notebook_resources = if !requests.is_empty() || !limits.is_empty() {
            Some(NotebookResources {
                requests: if requests.is_empty() { None } else { Some(requests) },
                limits: if limits.is_empty() { None } else { Some(limits) },
            })
        } else {
            None
        };

        // Build environment variables
        let env_vars = request.environment_variables.as_ref().map(|env_map| {
            env_map
                .iter()
                .map(|(k, v)| NotebookEnvVar {
                    name: k.clone(),
                    value: v.clone(),
                })
                .collect()
        });

        // Build volume mounts and volumes
        let (volume_mounts, volumes) = if request.workspace_volume_size.is_some() {
            let mount_path = request
                .workspace_volume_mount
                .as_deref()
                .unwrap_or("/home/jovyan/work");

            let volume_mounts = vec![NotebookVolumeMount {
                name: format!("{}-workspace", request.name),
                mount_path: mount_path.to_string(),
            }];

            let volumes = vec![NotebookVolume {
                name: format!("{}-workspace", request.name),
                persistent_volume_claim: Some(NotebookPvcSource {
                    claim_name: format!("{}-workspace-pvc", request.name),
                }),
                empty_dir: None,
            }];

            (Some(volume_mounts), Some(volumes))
        } else {
            (None, None)
        };

        // Build ports
        let ports = vec![NotebookPort {
            container_port: 8888,
            name: "notebook-port".to_string(),
            protocol: "TCP".to_string(),
        }];

        let container = NotebookContainer {
            name: "notebook".to_string(),
            image: request.image.clone(),
            resources: notebook_resources,
            env: env_vars,
            volume_mounts,
            ports: Some(ports),
        };

        let pod_spec = NotebookPodSpec {
            containers: vec![container],
            volumes,
            service_account_name: request.service_account.clone(),
        };

        let template = NotebookTemplate { spec: pod_spec };

        Ok(NotebookSpec { template })
    }

    fn build_update_spec(
        &self,
        existing_spec: &NotebookSpec,
        request: &UpdateNotebookRequest,
    ) -> Result<NotebookSpec> {
        let mut updated_spec = existing_spec.clone();

        if let Some(container) = updated_spec.template.spec.containers.get_mut(0) {
            // Update image if provided
            if let Some(image) = &request.image {
                container.image = image.clone();
            }

            // Update resources if provided
            if request.cpu_request.is_some()
                || request.cpu_limit.is_some()
                || request.memory_request.is_some()
                || request.memory_limit.is_some()
                || request.gpu_limit.is_some()
            {
                let mut requests = container
                    .resources
                    .as_ref()
                    .and_then(|r| r.requests.clone())
                    .unwrap_or_default();
                let mut limits = container
                    .resources
                    .as_ref()
                    .and_then(|r| r.limits.clone())
                    .unwrap_or_default();

                if let Some(cpu_request) = &request.cpu_request {
                    requests.insert("cpu".to_string(), cpu_request.clone());
                }
                if let Some(cpu_limit) = &request.cpu_limit {
                    limits.insert("cpu".to_string(), cpu_limit.clone());
                }
                if let Some(memory_request) = &request.memory_request {
                    requests.insert("memory".to_string(), memory_request.clone());
                }
                if let Some(memory_limit) = &request.memory_limit {
                    limits.insert("memory".to_string(), memory_limit.clone());
                }
                if let Some(gpu_limit) = &request.gpu_limit {
                    limits.insert("nvidia.com/gpu".to_string(), gpu_limit.clone());
                }

                container.resources = Some(NotebookResources {
                    requests: if requests.is_empty() { None } else { Some(requests) },
                    limits: if limits.is_empty() { None } else { Some(limits) },
                });
            }

            // Update environment variables if provided
            if let Some(env_vars) = &request.environment_variables {
                container.env = Some(
                    env_vars
                        .iter()
                        .map(|(k, v)| NotebookEnvVar {
                            name: k.clone(),
                            value: v.clone(),
                        })
                        .collect(),
                );
            }
        }

        Ok(updated_spec)
    }

    async fn create_workspace_pvc(
        &self,
        client: &Client,
        namespace: &str,
        notebook_name: &str,
        size: &str,
    ) -> Result<()> {
        let pvc_api: Api<PersistentVolumeClaim> = Api::namespaced(client.clone(), namespace);
        let pvc_name = format!("{}-workspace-pvc", notebook_name);

        let pvc = serde_json::from_value::<PersistentVolumeClaim>(json!({
            "apiVersion": "v1",
            "kind": "PersistentVolumeClaim",
            "metadata": {
                "name": pvc_name,
                "namespace": namespace
            },
            "spec": {
                "accessModes": ["ReadWriteOnce"],
                "resources": {
                    "requests": {
                        "storage": size
                    }
                }
            }
        })).map_err(|e| AppError::Internal(format!("Failed to create PVC spec: {}", e)))?;

        match pvc_api.create(&Default::default(), &pvc).await {
            Ok(_) => Ok(()),
            Err(e) => {
                tracing::warn!("Failed to create workspace PVC: {}", e);
                Ok(()) // Don't fail notebook creation if PVC creation fails
            }
        }
    }

    async fn delete_workspace_pvc(
        &self,
        client: &Client,
        namespace: &str,
        notebook_name: &str,
    ) -> Result<()> {
        let pvc_api: Api<PersistentVolumeClaim> = Api::namespaced(client.clone(), namespace);
        let pvc_name = format!("{}-workspace-pvc", notebook_name);

        match pvc_api.delete(&pvc_name, &Default::default()).await {
            Ok(_) => Ok(()),
            Err(_) => Ok(()), // PVC might not exist, that's okay
        }
    }
}