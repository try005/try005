# Kubernetes Resource Manager

A modular Rust microservice for managing various Kubernetes resources via REST APIs.

## Overview

This service provides a unified REST API interface for managing different types of Kubernetes resources. Currently supports CloudNativePG (CNPG) PostgreSQL clusters, with a modular architecture designed for easy extension to support additional resource types.

## Supported Resources

### ✅ Currently Available
- **CNPG PostgreSQL Clusters** - Full CRUD operations for CloudNativePG database clusters

### 🚧 Planned Future Support
- **KubeVirt Virtual Machines** - VM lifecycle management
- **Strimzi Kafka Clusters** - Kafka cluster deployment and management  
- **Cluster API Kubernetes Clusters** - Managed Kubernetes cluster provisioning
- **Additional operators and CRDs** - Extensible framework for any Kubernetes resource

## Architecture

The service uses a modular plugin-based architecture:

```
src/
├── handlers/          # HTTP request handlers for each resource type
│   ├── cnpg.rs       # CNPG-specific endpoints
│   └── health.rs     # Health check endpoint
├── resources/         # Resource management logic
│   ├── cnpg.rs       # CNPG resource manager implementation
│   └── mod.rs        # ResourceManager trait definition
├── models/           # Data models and request/response types
│   ├── cnpg.rs       # CNPG-specific models
│   └── mod.rs        # Common models
├── error.rs          # Centralized error handling
├── utils/            # Shared utilities
└── main.rs           # Application entry point and routing
```

## API Endpoints

### Health Check
- `GET /health` - Service health status

### CNPG PostgreSQL Clusters
- `POST /cnpg/clusters` - Create a new PostgreSQL cluster
- `GET /cnpg/clusters?namespace=<ns>` - List clusters in namespace
- `GET /cnpg/clusters/<namespace>/<name>` - Get specific cluster
- `PUT /cnpg/clusters/<namespace>/<name>` - Update cluster configuration
- `DELETE /cnpg/clusters/<namespace>/<name>` - Delete cluster

### Legacy Compatibility
- `POST /clusters` - ⚠️ **Deprecated** - Use `/cnpg/clusters` instead
- `GET /clusters` - ⚠️ **Deprecated** - Use `/cnpg/clusters` instead
- And corresponding CRUD operations...

### Future Endpoints (Planned)
```
/kubevirt/vms          # Virtual machine management
/strimzi/kafka         # Kafka cluster management  
/cluster-api/clusters  # Kubernetes cluster management
```

## Example Usage

### Create a PostgreSQL Cluster
```bash
curl -X POST http://localhost:3000/cnpg/clusters \
-H "Content-Type: application/json" \
-d '{
  "name": "my-postgres",
  "instances": 3,
  "database_name": "myapp",
  "database_owner": "appuser",
  "secret_name": "postgres-secret",
  "storage_size": "10Gi",
  "postgresql_parameters": {
    "max_connections": "200",
    "shared_buffers": "512MB"
  },
  "monitoring_enabled": true
}'
```

### List All Clusters
```bash
curl http://localhost:3000/cnpg/clusters
```

### Get Specific Cluster
```bash
curl http://localhost:3000/cnpg/clusters/default/my-postgres
```

### Update Cluster
```bash
curl -X PUT http://localhost:3000/cnpg/clusters/default/my-postgres \
-H "Content-Type: application/json" \
-d '{
  "instances": 5,
  "postgresql_parameters": {
    "max_connections": "300"
  }
}'
```

### Delete Cluster
```bash
curl -X DELETE http://localhost:3000/cnpg/clusters/default/my-postgres
```

## Prerequisites

- Rust toolchain (1.70+)
- Kubernetes cluster with appropriate operators installed:
  - CloudNativePG operator for PostgreSQL clusters
  - (Future: KubeVirt, Strimzi, Cluster API, etc.)
- PostgreSQL secrets created in target namespaces
- kubectl access to the cluster

## Quick Start

### 1. Clone and Build
```bash
git clone <repository>
cd k8s-resource-manager
cargo build --release
```

### 2. Set Up Local Development Environment

**Create Kind cluster:**
```bash
kind create cluster --name resource-manager
```

**Install CNPG operator:**
```bash
kubectl apply --server-side -f https://raw.githubusercontent.com/cloudnative-pg/cloudnative-pg/release-1.24/releases/cnpg-1.24.1.yaml
```

**Create PostgreSQL secret:**
```bash
kubectl create secret generic postgres-secret \
  --from-literal=username=postgres \
  --from-literal=password=your-secure-password
```

### 3. Run the Service
```bash
cargo run
```

The service starts on `http://localhost:3000`

## Connecting to PostgreSQL Clusters with DBeaver

Once you've created a CNPG cluster, you can connect to it using DBeaver or any PostgreSQL client.

### 1. Get Connection Details

Find the services created by CNPG:
```bash
kubectl get services | grep <cluster-name>
```

You'll see services like:
- `<cluster-name>-rw` - Read-write service (connects to primary)
- `<cluster-name>-r` - Read-only service (connects to replicas)

### 2. Port Forward to Access Locally

Forward the PostgreSQL port to your local machine:
```bash
# For read-write access (primary)
kubectl port-forward service/<cluster-name>-rw 5432:5432

# Or for read-only access (replicas)
kubectl port-forward service/<cluster-name>-r 5433:5432
```

### 3. Get Database Credentials

The credentials are stored in the secret you specified when creating the cluster:
```bash
# Extract username and password
kubectl get secret <secret-name> -o jsonpath='{.data.username}' | base64 -d
kubectl get secret <secret-name> -o jsonpath='{.data.password}' | base64 -d
```

### 4. Configure DBeaver Connection

Open DBeaver and create a new PostgreSQL connection:

**Connection Settings:**
- **Host:** `localhost`
- **Port:** `5432` (or `5433` if using read-only)
- **Database:** (the database name you specified when creating the cluster)
- **Username:** (from step 3)
- **Password:** (from step 3)

### 5. Test and Connect

Click "Test Connection" in DBeaver to verify, then save and connect.

### Complete Example Workflow

```bash
# 1. Create a cluster
curl -X POST http://localhost:3000/cnpg/clusters \
-H "Content-Type: application/json" \
-d '{
  "name": "demo-cluster",
  "instances": 2,
  "database_name": "myapp",
  "database_owner": "appuser",
  "secret_name": "postgres-secret",
  "storage_size": "5Gi"
}'

# 2. Wait for cluster to be ready
kubectl wait --for=condition=Ready cluster/demo-cluster --timeout=300s

# 3. Port forward
kubectl port-forward service/demo-cluster-rw 5432:5432 &

# 4. Get credentials from your secret
kubectl get secret postgres-secret -o jsonpath='{.data.password}' | base64 -d

# 5. Connect with DBeaver using:
# Host: localhost, Port: 5432, Database: myapp, Username: appuser, Password: <from step 4>
```

## Adding New Resource Types

The modular architecture makes it easy to add support for new Kubernetes resources:

### 1. Create Models
Add your resource models in `src/models/<resource_type>.rs`:
```rust
#[derive(Debug, Deserialize)]
pub struct CreateVmRequest {
    pub name: String,
    pub namespace: Option<String>,
    pub cpu_cores: u32,
    pub memory: String,
    // ... other fields
}
```

### 2. Implement ResourceManager
Create `src/resources/<resource_type>.rs`:
```rust
pub struct VmManager;

#[async_trait]
impl ResourceManager for VmManager {
    type CreateRequest = CreateVmRequest;
    type UpdateRequest = UpdateVmRequest;
    type Resource = VirtualMachine;

    async fn create(&self, client: Client, request: Self::CreateRequest) -> Result<Value> {
        // Implementation
    }
    // ... other methods
}
```

### 3. Add HTTP Handlers
Create `src/handlers/<resource_type>.rs` with endpoint handlers.

### 4. Register Routes
Add routes in `src/main.rs`:
```rust
.route("/kubevirt/vms", post(kubevirt::create_vm))
.route("/kubevirt/vms", get(kubevirt::list_vms))
// ... other routes
```

## Configuration

The service can be configured via environment variables:

- `RUST_LOG` - Logging level (default: `info`)
- `BIND_ADDRESS` - Server bind address (default: `0.0.0.0:3000`)
- `KUBECONFIG` - Path to kubeconfig file (uses default cluster config if not set)

## Production Considerations

### Security
- Use proper RBAC for service account permissions
- Enable TLS/SSL for production deployments
- Implement authentication and authorization
- Validate all inputs and sanitize responses

### Monitoring
- Health checks at `/health`
- Structured logging with tracing
- Consider adding metrics endpoints (Prometheus)
- Set up alerting for cluster operations

### High Availability
- Run multiple replicas behind a load balancer
- Implement graceful shutdown handling
- Use persistent storage for any local state
- Consider implementing request queuing for rate limiting

### Database Connections
- Use read-only connections for reporting workloads
- Configure connection pooling (PgBouncer) for production clusters
- Monitor connection limits and performance
- Set up SSL/TLS certificates for database connections

## Development

### Running Tests
```bash
cargo test
```

### Code Formatting
```bash
cargo fmt
```

### Linting
```bash
cargo clippy
```

### Building for Production
```bash
cargo build --release
```

## Contributing

1. Follow the modular architecture patterns
2. Add appropriate error handling
3. Include tests for new functionality
4. Update documentation for new resource types
5. Ensure backward compatibility for existing APIs

## License

[Specify your license here]