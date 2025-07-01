pub mod cnpg;
pub mod kubeflow;

use crate::error::Result;
use kube::Client;
use serde_json::Value;

#[async_trait::async_trait]
pub trait ResourceManager {
    type CreateRequest;
    type UpdateRequest;
    type Resource;

    async fn create(&self, client: Client, request: Self::CreateRequest) -> Result<Value>;
    async fn get(&self, client: Client, namespace: &str, name: &str) -> Result<Self::Resource>;
    async fn list(&self, client: Client, namespace: &str) -> Result<Value>;
    async fn update(
        &self,
        client: Client,
        namespace: &str,
        name: &str,
        request: Self::UpdateRequest,
    ) -> Result<Value>;
    async fn delete(&self, client: Client, namespace: &str, name: &str) -> Result<Value>;
}