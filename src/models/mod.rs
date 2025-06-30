pub mod cnpg;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub namespace: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ResourceInfo {
    pub name: Option<String>,
    pub namespace: Option<String>,
    pub resource_type: String,
    pub creation_timestamp: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ResourceList {
    pub resources: Vec<ResourceInfo>,
    pub count: usize,
    pub resource_type: String,
}