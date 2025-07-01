# Architecture Overview

## Project Structure

```
k8s-resource-manager/
├── Cargo.toml                    # Project dependencies and metadata
├── README.md                     # Main documentation
├── ARCHITECTURE.md               # This file - architecture details
│
├── src/
│   ├── main.rs                   # Application entry point, routing, and graceful shutdown
│   ├── error.rs                  # Centralized error handling with custom error types
│   │
│   ├── handlers/                 # HTTP request handlers with comprehensive validation
│   │   ├── mod.rs               # Module exports
│   │   ├── health.rs            # Health check endpoints
│   │   ├── cnpg.rs              # CNPG cluster endpoints
│   │   ├── kubeflow.rs          # Kubeflow notebook endpoints
│   │   ├── kubevirt.rs          # (Future) VM management endpoints
│   │   ├── strimzi.rs           # (Future) Kafka cluster endpoints
│   │   └── cluster_api.rs       # (Future) K8s cluster endpoints
│   │
│   ├── resources/               # Business logic for resource management
│   │   ├── mod.rs              # ResourceManager trait definition
│   │   ├── cnpg.rs             # CNPG resource management implementation
│   │   ├── kubeflow.rs         # Kubeflow notebook management implementation
│   │   ├── kubevirt.rs         # (Future) KubeVirt implementation
│   │   ├── strimzi.rs          # (Future) Strimzi implementation
│   │   └── cluster_api.rs      # (Future) Cluster API implementation
│   │
│   ├── models/                 # Data models and request/response types
│   │   ├── mod.rs             # Common models and exports
│   │   ├── cnpg.rs            # CNPG-specific models
│   │   ├── kubeflow.rs        # Kubeflow notebook models with Kubernetes CRDs
│   │   ├── kubevirt.rs        # (Future) VM models
│   │   ├── strimzi.rs         # (Future) Kafka models
│   │   └── cluster_api.rs     # (Future) Cluster API models
│   │
│   └── utils/                 # Shared utilities
│       ├── mod.rs            # Utility exports
│       └── validation.rs     # Comprehensive input validation with unit tests
│
├── install.sh               # Automated CNPG and Kubeflow installation script
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
All errors flow through the `AppError` enum with:
- Automatic HTTP status code mapping
- Kubernetes error translation
- Structured error responses
- No panic guarantees through comprehensive Result handling

### 4. RESTful API Design
- Resource-specific endpoints: `/cnpg/clusters`, `/kubeflow/notebooks`
- Consistent CRUD operations across all resource types
- Legacy compatibility maintained during transitions
- Comprehensive input validation for all endpoints

### 5. Production-Ready Features
- Graceful shutdown handling (SIGTERM, Ctrl+C)
- Comprehensive logging with structured tracing
- Input validation with detailed error messages
- PVC lifecycle management for persistent storage
- Environment-based configuration

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

### ✅ Fully Implemented
- **CNPG PostgreSQL Clusters**: Full CRUD operations with validation
- **Kubeflow Notebooks**: Complete notebook lifecycle management
  - Jupyter notebook deployment with custom resources
  - PVC management for persistent workspaces
  - Resource allocation (CPU, memory, GPU)
  - Environment variable configuration
  - Volume mounting and storage management
- **Modular architecture**: Proven extensible design
- **Error handling**: Panic-free with comprehensive Result types
- **Input validation**: All endpoints validate requests with detailed errors
- **Health checks**: Service monitoring with JSON responses
- **Graceful shutdown**: Signal handling for production deployment
- **Installation automation**: Complete CNPG and Kubeflow deployment script
- **Legacy compatibility**: Backward-compatible API during transitions

### 🚧 Ready for Implementation
The proven architecture easily supports:
- **KubeVirt VMs**: Virtual machine lifecycle management
- **Strimzi Kafka**: Kafka cluster operations
- **Cluster API**: Kubernetes cluster provisioning
- **Custom Resources**: Any Kubernetes CRD following the established pattern

## API Evolution Strategy

### Current API Structure
```
/health                              # Health check

# Resource-specific endpoints
/cnpg/clusters                       # PostgreSQL clusters (full CRUD)
/kubeflow/notebooks                  # Jupyter notebooks (full CRUD)

# Legacy endpoints (deprecated)
/clusters                            # Legacy CNPG (backward compatibility)
```

### Future API Structure
```
/health                              # Health check

# Resource-specific endpoints
/cnpg/clusters                       # PostgreSQL clusters
/kubeflow/notebooks                  # Jupyter notebooks
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
- Comprehensive input validation with custom validation functions
- Kubernetes RBAC integration with proper service accounts
- Structured error responses (no internal details leaked)
- Pod security contexts with non-root execution
- Capability dropping for minimal privilege containers
- Resource name validation to prevent injection attacks
- Storage size and resource limit validation

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
- Graceful shutdown for clean resource cleanup
- Structured logging for efficient debugging
- Early validation to fail fast on invalid inputs
- Optimized error handling without panics

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
- Input validation functions (comprehensive test coverage)
- Custom error type conversions

### Integration Tests
- End-to-end API flows
- Kubernetes resource creation/modification
- Error condition handling
- CNPG and Kubeflow operator integration
- PVC lifecycle management
- RBAC permission validation

### Production Testing
- Release build validation
- Real cluster deployment testing
- Graceful shutdown behavior
- Error recovery scenarios
- Resource cleanup verification

### Load Tests
- Concurrent request handling
- Resource creation at scale
- Memory usage under load

## Deployment and Operations

### Installation and Setup
The project includes `install.sh` which automates the complete setup process:
- Validates Talos kubeconfig and cluster connectivity
- Installs CNPG operator with proper RBAC permissions
- Deploys Kubeflow notebook controller and CRDs
- Creates necessary storage classes and service accounts
- Configures pod security policies for secure operation
- Verifies all components are running correctly

### Production Deployment
- **Binary**: Single statically-linked binary for easy deployment
- **Configuration**: Environment variables for runtime configuration
- **Monitoring**: Structured JSON logging with configurable levels
- **Health**: `/health` endpoint for load balancer health checks
- **Shutdown**: Graceful shutdown handling for container orchestration

### Operational Features
- **Error Resilience**: No panics, comprehensive error handling
- **Validation**: Early input validation prevents invalid Kubernetes resources
- **Resource Cleanup**: Automatic PVC cleanup when deleting notebooks
- **Security**: Pod security contexts and capability dropping
- **Observability**: Structured logging with resource tracking

### Development Workflow
1. **Local Development**: `cargo run` for development server
2. **Testing**: `cargo test` for unit and integration tests
3. **Production Build**: `cargo build --release` for optimized binary
4. **Cluster Setup**: `./install.sh` for complete environment setup
5. **API Testing**: cURL commands for manual testing (see README.md)

This modular architecture ensures the service can grow to support multiple Kubernetes resource types while maintaining clean separation of concerns, consistent APIs, and production-ready operational characteristics.