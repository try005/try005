# CNPG Microservice

A Rust microservice for managing CloudNativePG (CNPG) clusters via REST API.

## Features

- **Full CRUD operations** for CNPG PostgreSQL clusters
- **REST API** with JSON payloads
- **Kubernetes integration** using the official Kubernetes Rust client
- **Automatic YAML generation** from API requests
- **Error handling** with proper HTTP status codes

## API Endpoints

- `GET /health` - Health check
- `POST /clusters` - Create a new CNPG cluster
- `GET /clusters?namespace=<ns>` - List clusters in namespace (default: "default")
- `GET /clusters/<namespace>/<name>` - Get specific cluster
- `PUT /clusters/<namespace>/<name>` - Update cluster configuration
- `DELETE /clusters/<namespace>/<name>` - Delete cluster

## Example Usage

### Create a cluster
```bash
curl -X POST http://localhost:3000/clusters \
-H "Content-Type: application/json" \
-d '{
  "name": "my-cluster",
  "instances": 3,
  "database_name": "mydb",
  "database_owner": "myuser",
  "secret_name": "postgres-secret",
  "storage_size": "10Gi",
  "postgresql_parameters": {
    "max_connections": "200",
    "shared_buffers": "512MB"
  }
}'
```

### List clusters
```bash
curl http://localhost:3000/clusters
```

### Update cluster
```bash
curl -X PUT http://localhost:3000/clusters/default/my-cluster \
-H "Content-Type: application/json" \
-d '{
  "instances": 5,
  "postgresql_parameters": {
    "max_connections": "300"
  }
}'
```

### Delete cluster
```bash
curl -X DELETE http://localhost:3000/clusters/default/my-cluster
```

## Prerequisites

- Kubernetes cluster with CNPG operator installed
- PostgreSQL secrets created in the target namespace
- Rust toolchain installed

## Setup

1. **Install dependencies:**
   ```bash
   cargo build
   ```

2. **Create PostgreSQL secret:**
   ```bash
   kubectl create secret generic postgres-secret \
     --from-literal=username=postgres \
     --from-literal=password=your-password
   ```

3. **Run the service:**
   ```bash
   cargo run
   ```

The service will start on `http://localhost:3000`.

## Local Development with Kind

The project includes setup for local development with Kind:

1. **Install Kind and create cluster:**
   ```bash
   kind create cluster --name cnpg-cluster
   ```

2. **Install CNPG operator:**
   ```bash
   kubectl apply --server-side -f https://raw.githubusercontent.com/cloudnative-pg/cloudnative-pg/release-1.24/releases/cnpg-1.24.1.yaml
   ```

3. **Create test secret and run service as shown above**

## Connecting to PostgreSQL Clusters with DBeaver

Once you've created a CNPG cluster, you can connect to it using DBeaver or any PostgreSQL client.

### 1. Get Connection Details

First, find the services created by CNPG:
```bash
kubectl get services | grep my-cluster
```

You'll see services like:
- `my-cluster-rw` - Read-write service (connects to primary)
- `my-cluster-r` - Read-only service (connects to replicas)
- `my-cluster-ro` - Read-only service (alternative name)

### 2. Port Forward to Access Locally

Forward the PostgreSQL port to your local machine:
```bash
# For read-write access (primary)
kubectl port-forward service/my-cluster-rw 5432:5432

# Or for read-only access (replicas)
kubectl port-forward service/my-cluster-r 5433:5432
```

### 3. Get Database Credentials

The credentials are stored in Kubernetes secrets. Get the password:
```bash
# Get the application secret name (usually cluster-name-app)
kubectl get secrets | grep my-cluster

# Extract username and password
kubectl get secret my-cluster-app -o jsonpath='{.data.username}' | base64 -d
kubectl get secret my-cluster-app -o jsonpath='{.data.password}' | base64 -d
```

### 4. Configure DBeaver Connection

Open DBeaver and create a new PostgreSQL connection with these settings:

**Connection Settings:**
- **Host:** `localhost`
- **Port:** `5432` (or `5433` if using read-only port-forward)
- **Database:** `mydb` (the database name you specified when creating the cluster)
- **Username:** (from step 3, usually same as database_owner)
- **Password:** (from step 3)

**Advanced Settings:**
- You may want to set connection timeout to handle network latency
- For production, consider using SSL settings if configured

### 5. Test Connection

Click "Test Connection" in DBeaver to verify everything works. You should now be able to:
- Browse database schemas
- Execute SQL queries
- Manage database objects
- Monitor connection status

### Example Connection Workflow

```bash
# 1. Create a cluster
curl -X POST http://localhost:3000/clusters \
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

# 4. Get credentials
kubectl get secret demo-cluster-app -o jsonpath='{.data.password}' | base64 -d

# 5. Connect with DBeaver using:
# Host: localhost, Port: 5432, Database: myapp, Username: appuser, Password: <from step 4>
```

### Production Considerations

For production deployments:
- Use proper LoadBalancer or Ingress instead of port-forwarding
- Configure SSL/TLS certificates
- Set up connection pooling (PgBouncer)
- Use read-only connections for reporting workloads
- Monitor connection limits and performance

## Architecture

The microservice:
1. Receives REST API requests with cluster parameters
2. Converts them to CNPG Cluster custom resources
3. Applies the resources to Kubernetes using the official Rust client
4. Returns structured JSON responses

Built with:
- **Axum** - Web framework
- **Kube-rs** - Kubernetes client library
- **Tokio** - Async runtime
- **Serde** - JSON serialization