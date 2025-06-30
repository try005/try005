# Architecture Overview

## Project Structure

```
k8s-resource-manager/
├── Cargo.toml                    # Project dependencies and metadata
├── README.md                     # Main documentation
├── ARCHITECTURE.md               # This file - architecture details
│
├── src/
│   ├── main.rs                   # Application entry point and routing
│   ├── error.rs                  # Centralized error handling
│   │
│   ├── handlers/                 # HTTP request handlers
│   │   ├── mod.rs               # Module exports
│   │   ├── health.rs            # Health check endpoints
│   │   ├── cnpg.rs              # CNPG cluster endpoints
│   │   ├── kubevirt.rs          # (Future) VM management endpoints
│   │   ├── strimzi.rs           # (Future) Kafka cluster endpoints
│   │   └── cluster_api.rs       # (Future) K8s cluster endpoints
│   │
│   ├── resources/               # Business logic for resource management
│   │   ├── mod.rs              # ResourceManager trait definition
│   │   ├── cnpg.rs             # CNPG resource management implementation
│   │   ├── kubevirt.rs         # (Future) KubeVirt implementation
│   │   ├── strimzi.rs          # (Future) Strimzi implementation
│   │   └── cluster_api.rs      # (Future) Cluster API implementation
│   │
│   ├── models/                 # Data models and request/response types
│   │   ├── mod.rs             # Common models and exports
│   │   ├── cnpg.rs            # CNPG-specific models
│   │   ├── kubevirt.rs        # (Future) VM models
│   │   ├── strimzi.rs         # (Future) Kafka models
│   │   └── cluster_api.rs     # (Future) Cluster API models
│   │
│   └── utils/                 # Shared utilities
│       ├── mod.rs            # Utility exports
│       ├── validation.rs     # (Future) Input validation
│       ├── yaml.rs           # (Future) YAML generation helpers
│       └── k8s.rs            # (Future) Kubernetes client helpers
│
└── target/                   # Compiled artifacts (git-ignored)
```

## Key Design Principles

### 1. Modular Architecture
Each resource type (CNPG, KubeVirt, etc.) is implemented as a separate module with:
- **Models**: Request/response types and Kubernetes resource definitions
- **Resource Manager**: Business logic implementing the `ResourceManager` trait
- **Handlers**: HTTP endpoint handlers that use the resource managers

### 2. Trait-Based Abstraction
The `ResourceManager` trait provides a consistent interface:
```rust
#[async_trait]
pub trait ResourceManager {
    type CreateRequest;
    type UpdateRequest;
    type Resource;

    async fn create(&self, client: Client, request: Self::CreateRequest) -> Result<Value>;
    async fn get(&self, client: Client, namespace: &str, name: &str) -> Result<Self::Resource>;
    async fn list(&self, client: Client, namespace: &str) -> Result<Value>;
    async fn update(&self, client: Client, namespace: &str, name: &str, request: Self::UpdateRequest) -> Result<Value>;
    async fn delete(&self, client: Client, namespace: &str, name: &str) -> Result<Value>;
}
```

### 3. Centralized Error Handling
All errors flow through the `AppError` enum with automatic HTTP status code mapping.

### 4. RESTful API Design
- Resource-specific endpoints: `/cnpg/clusters`, `/kubevirt/vms`
- Consistent CRUD operations across all resource types
- Legacy compatibility maintained during transitions

## Adding New Resource Types

### Step-by-Step Guide

1. **Define Models** (`src/models/<resource>.rs`)
   ```rust
   #[derive(Debug, Deserialize)]
   pub struct CreateVmRequest {
       pub name: String,
       pub namespace: Option<String>,
       // ... resource-specific fields
   }
   ```

2. **Implement ResourceManager** (`src/resources/<resource>.rs`)
   ```rust
   pub struct VmManager;
   
   #[async_trait]
   impl ResourceManager for VmManager {
       // Implement all trait methods
   }
   ```

3. **Create HTTP Handlers** (`src/handlers/<resource>.rs`)
   ```rust
   pub async fn create_vm(Json(payload): Json<CreateVmRequest>) -> Result<ResponseJson<Value>> {
       let manager = VmManager;
       // ... implementation
   }
   ```

4. **Register Routes** (`src/main.rs`)
   ```rust
   .route("/kubevirt/vms", post(kubevirt::create_vm))
   .route("/kubevirt/vms", get(kubevirt::list_vms))
   // ... other routes
   ```

5. **Update Documentation** (README.md, this file)

## Current Implementation Status

### ✅ Implemented
- **CNPG PostgreSQL Clusters**: Full CRUD operations
- **Modular architecture**: Ready for extension
- **Error handling**: Centralized and HTTP-aware
- **Health checks**: Service monitoring
- **Legacy compatibility**: Backward-compatible API

### 🚧 Ready for Implementation
The architecture is designed to easily support:
- **KubeVirt VMs**: Virtual machine lifecycle management
- **Strimzi Kafka**: Kafka cluster operations
- **Cluster API**: Kubernetes cluster provisioning
- **Custom Resources**: Any Kubernetes CRD

## API Evolution Strategy

### Current API Structure
```
/health                              # Health check
/cnpg/clusters                       # New CNPG endpoints
/clusters                            # Legacy CNPG (deprecated)
```

### Future API Structure
```
/health                              # Health check

# Resource-specific endpoints
/cnpg/clusters                       # PostgreSQL clusters
/kubevirt/vms                        # Virtual machines
/strimzi/kafka                       # Kafka clusters
/cluster-api/clusters                # Kubernetes clusters

# Legacy endpoints (deprecated)
/clusters                            # Legacy CNPG
```

### Migration Strategy
1. **Phase 1**: Implement new resource-specific endpoints
2. **Phase 2**: Mark legacy endpoints as deprecated
3. **Phase 3**: Remove legacy endpoints in next major version

## Security Considerations

### Current Security Features
- Input validation through Rust type system
- Kubernetes RBAC integration
- Structured error responses (no internal details leaked)

### Future Security Enhancements
- Authentication middleware
- Authorization per resource type
- Request rate limiting
- Audit logging
- Input sanitization utilities

## Performance Considerations

### Current Performance Features
- Async/await throughout (Tokio runtime)
- Efficient JSON serialization (serde)
- Connection pooling (kube-rs)

### Future Performance Enhancements
- Request queuing for resource-intensive operations
- Caching for frequently accessed resources
- Metrics collection (Prometheus)
- Graceful degradation under load

## Testing Strategy

### Unit Tests
- Resource manager implementations
- Model serialization/deserialization
- Error handling paths

### Integration Tests
- End-to-end API flows
- Kubernetes resource creation/modification
- Error condition handling

### Load Tests
- Concurrent request handling
- Resource creation at scale
- Memory usage under load

This modular architecture ensures the service can grow to support multiple Kubernetes resource types while maintaining clean separation of concerns and consistent APIs.