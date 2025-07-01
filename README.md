# Kubernetes Resource Manager

A modular Rust microservice for managing various Kubernetes resources via REST APIs. Currently supports CloudNativePG (CNPG) PostgreSQL clusters and Kubeflow Jupyter Notebooks, with a modular architecture designed for easy extension to support additional resource types.

## Overview

This service provides a unified REST API interface for managing different types of Kubernetes resources. The modular plugin-based architecture makes it easy to add support for new resource types.

### ✅ Currently Available
- **CNPG PostgreSQL Clusters** - Full CRUD operations for CloudNativePG database clusters
- **Kubeflow Jupyter Notebooks** - Create and manage Jupyter notebook servers with custom resources

### 🚧 Planned Future Support
- **KubeVirt Virtual Machines** - VM lifecycle management
- **Strimzi Kafka Clusters** - Kafka cluster deployment and management  
- **Cluster API Kubernetes Clusters** - Managed Kubernetes cluster provisioning
- **Additional operators and CRDs** - Extensible framework for any Kubernetes resource

---

# 1. Setup from Scratch

## Prerequisites

- **Rust toolchain** (1.70+)
- **Kubernetes cluster** (Talos recommended)
- **kubectl** access to the cluster
- **talosctl** (for Talos clusters)
- **openssl** for password generation

## Quick Installation

### Step 1: Clone and Build
```bash
git clone <repository>
cd k8s-resource-manager
cargo build --release
```

### Step 2: Run Automated Installation
```bash
# Make script executable
chmod +x install.sh

# Run installation (will install CNPG + Kubeflow)
./install.sh
```

The installation script will:
- ✅ Verify Talos cluster connection
- ✅ Install CloudNativePG operator
- ✅ Install Kubeflow notebook controller  
- ✅ Create storage classes for persistent volumes
- ✅ Configure proper RBAC permissions
- ✅ Create PostgreSQL secrets
- ✅ Run verification tests

### Step 3: Start the Service
```bash
cargo run
```

The service starts on `http://localhost:3000`

### Step 4: Verify Installation
```bash
# Check health
curl http://localhost:3000/health

# Check available endpoints
curl http://localhost:3000/cnpg/clusters
curl http://localhost:3000/kubeflow/notebooks
```

## Manual Installation (Alternative)

If you prefer manual setup or need to customize the installation:

### Install CNPG Operator
```bash
kubectl apply --server-side -f https://raw.githubusercontent.com/cloudnative-pg/cloudnative-pg/release-1.24/releases/cnpg-1.24.1.yaml
kubectl wait --for=condition=Available deployment/cnpg-controller-manager -n cnpg-system --timeout=300s
```

### Install Kubeflow Components
```bash
# Create namespace
kubectl create namespace kubeflow

# Install notebook CRDs
kubectl apply -f https://raw.githubusercontent.com/kubeflow/kubeflow/v1.8.0/components/notebook-controller/config/crd/bases/kubeflow.org_notebooks.yaml

# Create service account and RBAC (see install.sh for complete configuration)
```

### Create Storage Class
```bash
cat <<EOF | kubectl apply -f -
apiVersion: storage.k8s.io/v1
kind: StorageClass
metadata:
  name: local-storage
  annotations:
    storageclass.kubernetes.io/is-default-class: "true"
provisioner: kubernetes.io/no-provisioner
volumeBindingMode: Immediate
allowVolumeExpansion: true
EOF
```

---

# 2. How to Use Kubeflow Notebooks

## API Endpoints

- `POST /kubeflow/notebooks` - Create a new Jupyter notebook server
- `GET /kubeflow/notebooks?namespace=<ns>` - List notebooks in namespace
- `GET /kubeflow/notebooks/<namespace>/<name>` - Get specific notebook
- `PUT /kubeflow/notebooks/<namespace>/<name>` - Update notebook configuration
- `DELETE /kubeflow/notebooks/<namespace>/<name>` - Delete notebook

## Creating Notebooks

### Basic Notebook (No Persistent Storage)
```bash
curl -X POST http://localhost:3000/kubeflow/notebooks \
-H "Content-Type: application/json" \
-d '{
  "name": "basic-notebook",
  "image": "jupyter/scipy-notebook:latest",
  "cpu_request": "500m",
  "memory_request": "1Gi"
}'
```

### Advanced Notebook (With Persistent Storage)
```bash
curl -X POST http://localhost:3000/kubeflow/notebooks \
-H "Content-Type: application/json" \
-d '{
  "name": "data-science-notebook", 
  "image": "jupyter/tensorflow-notebook:latest",
  "cpu_request": "1",
  "cpu_limit": "2", 
  "memory_request": "2Gi",
  "memory_limit": "4Gi",
  "workspace_volume_size": "10Gi",
  "workspace_volume_mount": "/home/jovyan/work",
  "environment_variables": {
    "JUPYTER_ENABLE_LAB": "yes",
    "GRANT_SUDO": "yes"
  }
}'
```

### GPU-Enabled Notebook
```bash
curl -X POST http://localhost:3000/kubeflow/notebooks \
-H "Content-Type: application/json" \
-d '{
  "name": "gpu-notebook",
  "image": "jupyter/tensorflow-notebook:latest",
  "cpu_request": "1", 
  "memory_request": "4Gi",
  "gpu_limit": "1",
  "workspace_volume_size": "20Gi"
}'
```

## Managing Notebooks

### List All Notebooks
```bash
# Default namespace
curl http://localhost:3000/kubeflow/notebooks

# Specific namespace  
curl http://localhost:3000/kubeflow/notebooks?namespace=kubeflow

# Using kubectl
kubectl get notebooks --all-namespaces
kubectl get notebooks -n default
```

### Get Notebook Details
```bash
# Via API
curl http://localhost:3000/kubeflow/notebooks/default/basic-notebook

# Using kubectl
kubectl get notebook basic-notebook -o yaml
kubectl describe notebook basic-notebook
```

### Update Notebook Configuration
```bash
curl -X PUT http://localhost:3000/kubeflow/notebooks/default/basic-notebook \
-H "Content-Type: application/json" \
-d '{
  "cpu_limit": "2",
  "memory_limit": "4Gi",
  "environment_variables": {
    "NEW_VAR": "value"
  }
}'
```

### Delete Notebook
```bash
# Via API
curl -X DELETE http://localhost:3000/kubeflow/notebooks/default/basic-notebook

# Using kubectl
kubectl delete notebook basic-notebook
```

## Accessing Jupyter Notebooks

### Step 1: Wait for Notebook to be Ready
```bash
# Check pod status
kubectl get pods -l notebook-name=basic-notebook

# Wait for ready state
kubectl wait --for=condition=Ready pod/basic-notebook-0 --timeout=300s
```

### Step 2: Port Forward to Access
```bash
# Option A: Port forward to pod
kubectl port-forward pod/basic-notebook-0 8888:8888

# Option B: Port forward to service (if exists)
kubectl port-forward service/basic-notebook 8888:8888
```

### Step 3: Open in Browser
1. Open `http://localhost:8888` in your browser
2. If prompted for a token, get it from logs:
   ```bash
   kubectl logs basic-notebook-0 | grep -E "(token=|password=)"
   ```
3. Look for output like: `http://127.0.0.1:8888/?token=abc123...`

### Common Jupyter Images
- `jupyter/minimal-notebook:latest` - Basic Python environment
- `jupyter/scipy-notebook:latest` - Scientific Python stack
- `jupyter/tensorflow-notebook:latest` - TensorFlow + Keras
- `jupyter/pytorch-notebook:latest` - PyTorch
- `jupyter/datascience-notebook:latest` - R + Python + Julia

## Troubleshooting Notebooks

### Check Notebook Status
```bash
# Notebook resource status
kubectl describe notebook <notebook-name>

# Pod status and events
kubectl describe pod <notebook-name>-0

# Controller logs
kubectl logs -n kubeflow deployment/notebook-controller
```

### Common Issues

**Pod stuck in Pending:**
```bash
# Check events
kubectl get events --sort-by='.lastTimestamp' | grep <notebook-name>

# Check resource constraints
kubectl describe pod <notebook-name>-0 | grep -A 10 "Events:"
```

**Storage issues:**
```bash
# Check PVC status
kubectl get pvc | grep <notebook-name>

# Check storage class
kubectl get storageclass
```

**Access issues:**
```bash
# Verify port forwarding
kubectl get pods -l notebook-name=<notebook-name> -o wide

# Check service endpoints
kubectl get endpoints <notebook-name>

# Test connectivity
curl -I http://localhost:8888
```

---

# 3. How to Use CNPG PostgreSQL

## API Endpoints

- `POST /cnpg/clusters` - Create a new PostgreSQL cluster
- `GET /cnpg/clusters?namespace=<ns>` - List clusters in namespace
- `GET /cnpg/clusters/<namespace>/<name>` - Get specific cluster
- `PUT /cnpg/clusters/<namespace>/<name>` - Update cluster configuration
- `DELETE /cnpg/clusters/<namespace>/<name>` - Delete cluster

## Creating PostgreSQL Clusters

### Basic PostgreSQL Cluster
```bash
curl -X POST http://localhost:3000/cnpg/clusters \
-H "Content-Type: application/json" \
-d '{
  "name": "my-postgres",
  "instances": 1,
  "database_name": "myapp",
  "database_owner": "appuser", 
  "secret_name": "postgres-secret",
  "storage_size": "10Gi"
}'
```

### High Availability Cluster (3 replicas)
```bash
curl -X POST http://localhost:3000/cnpg/clusters \
-H "Content-Type: application/json" \
-d '{
  "name": "ha-postgres",
  "instances": 3,
  "database_name": "production",
  "database_owner": "produser",
  "secret_name": "postgres-secret", 
  "storage_size": "50Gi",
  "postgresql_parameters": {
    "max_connections": "200",
    "shared_buffers": "512MB",
    "effective_cache_size": "1GB"
  },
  "monitoring_enabled": true
}'
```

### Development Cluster (Small resources)
```bash
curl -X POST http://localhost:3000/cnpg/clusters \
-H "Content-Type: application/json" \
-d '{
  "name": "dev-postgres", 
  "instances": 1,
  "database_name": "devdb",
  "database_owner": "dev",
  "secret_name": "postgres-secret",
  "storage_size": "5Gi",
  "postgresql_parameters": {
    "max_connections": "50",
    "shared_buffers": "128MB"
  }
}'
```

## Managing PostgreSQL Clusters

### List All Clusters
```bash
# Default namespace
curl http://localhost:3000/cnpg/clusters

# Specific namespace
curl http://localhost:3000/cnpg/clusters?namespace=production

# Using kubectl
kubectl get clusters --all-namespaces
kubectl get cluster -o wide
```

### Get Cluster Details
```bash
# Via API
curl http://localhost:3000/cnpg/clusters/default/my-postgres

# Using kubectl
kubectl get cluster my-postgres -o yaml
kubectl describe cluster my-postgres
```

### Update Cluster Configuration
```bash
curl -X PUT http://localhost:3000/cnpg/clusters/default/my-postgres \
-H "Content-Type: application/json" \
-d '{
  "instances": 3,
  "postgresql_parameters": {
    "max_connections": "300",
    "work_mem": "4MB"
  }
}'
```

### Delete Cluster
```bash
# Via API
curl -X DELETE http://localhost:3000/cnpg/clusters/default/my-postgres

# Using kubectl  
kubectl delete cluster my-postgres
```

## Accessing PostgreSQL Databases

### Step 1: Get Database Credentials

The PostgreSQL password was generated during installation. To retrieve it:

```bash
# Get the password from the secret
kubectl get secret postgres-secret -o jsonpath='{.data.password}' | base64 -d
echo

# Get the username (usually 'postgres')
kubectl get secret postgres-secret -o jsonpath='{.data.username}' | base64 -d
echo

# Get both credentials at once
echo "Username: $(kubectl get secret postgres-secret -o jsonpath='{.data.username}' | base64 -d)"
echo "Password: $(kubectl get secret postgres-secret -o jsonpath='{.data.password}' | base64 -d)"
```

### Step 2: Find Database Services

```bash
# List services for your cluster
kubectl get services | grep my-postgres

# You'll see services like:
# my-postgres-rw   - Read-write service (primary)
# my-postgres-r    - Read-only service (replicas) 
# my-postgres-ro   - Read-only service (alternative name)
```

### Step 3: Port Forward to Access Database

```bash
# For read-write access (primary database)
kubectl port-forward service/my-postgres-rw 5432:5432

# For read-only access (replicas) 
kubectl port-forward service/my-postgres-r 5433:5432

# Run in background
kubectl port-forward service/my-postgres-rw 5432:5432 &
```

### Step 4: Connect with Database Clients

#### Using psql Command Line
```bash
# Connect to primary (read-write)
psql -h localhost -p 5432 -U postgres -d myapp

# Connect to replica (read-only)
psql -h localhost -p 5433 -U postgres -d myapp
```

#### Using DBeaver GUI Client
1. **Create New Connection** → PostgreSQL
2. **Connection Settings:**
   - **Host:** `localhost`
   - **Port:** `5432` (primary) or `5433` (replica)
   - **Database:** `myapp` (or your database name)
   - **Username:** `postgres` (or from secret)
   - **Password:** (from secret above)
3. **Test Connection** and **Save**

#### Using Python (psycopg2)
```python
import psycopg2

# Connect to primary database
conn = psycopg2.connect(
    host="localhost",
    port=5432,
    database="myapp", 
    user="postgres",
    password="<password-from-secret>"
)

cursor = conn.cursor()
cursor.execute("SELECT version();")
print(cursor.fetchone())
```

#### Using Node.js (pg)
```javascript
const { Client } = require('pg')

const client = new Client({
  host: 'localhost',
  port: 5432,
  database: 'myapp',
  user: 'postgres', 
  password: '<password-from-secret>'
})

client.connect()
client.query('SELECT NOW()', (err, res) => {
  console.log(res.rows[0])
  client.end()
})
```

## Complete Workflow Example

### Create → Connect → Use
```bash
# 1. Create cluster
curl -X POST http://localhost:3000/cnpg/clusters \
-H "Content-Type: application/json" \
-d '{
  "name": "example-db",
  "instances": 2,
  "database_name": "webapp",
  "database_owner": "webuser", 
  "secret_name": "postgres-secret",
  "storage_size": "10Gi"
}'

# 2. Wait for cluster to be ready
kubectl wait --for=condition=Ready cluster/example-db --timeout=300s

# 3. Get credentials
DB_PASSWORD=$(kubectl get secret postgres-secret -o jsonpath='{.data.password}' | base64 -d)
echo "Database password: $DB_PASSWORD"

# 4. Port forward
kubectl port-forward service/example-db-rw 5432:5432 &

# 5. Connect and create table
psql -h localhost -p 5432 -U postgres -d webapp -c "
CREATE TABLE users (
  id SERIAL PRIMARY KEY,
  name VARCHAR(100),
  email VARCHAR(100) UNIQUE
);
INSERT INTO users (name, email) VALUES ('John Doe', 'john@example.com');
SELECT * FROM users;
"
```

## Monitoring and Maintenance

### Check Cluster Health
```bash
# Cluster status
kubectl get cluster example-db -o yaml | grep -A 10 status

# Pod status  
kubectl get pods -l postgresql=example-db

# Service status
kubectl get services -l postgresql=example-db
```

### View Logs
```bash
# Primary pod logs
kubectl logs example-db-1

# All cluster logs
kubectl logs -l postgresql=example-db

# Follow logs
kubectl logs -f example-db-1
```

### Backup and Recovery
```bash
# Create backup
kubectl apply -f - <<EOF
apiVersion: postgresql.cnpg.io/v1
kind: Backup
metadata:
  name: example-db-backup
spec:
  cluster:
    name: example-db
EOF

# List backups
kubectl get backups

# Check backup status
kubectl describe backup example-db-backup
```

## Troubleshooting PostgreSQL

### Common Issues

**Cluster not starting:**
```bash
# Check cluster events
kubectl describe cluster example-db

# Check pod events  
kubectl describe pod example-db-1

# Check operator logs
kubectl logs -n cnpg-system deployment/cnpg-controller-manager
```

**Connection refused:**
```bash
# Verify service exists
kubectl get service example-db-rw

# Check port forwarding
netstat -tlnp | grep 5432

# Test connectivity
pg_isready -h localhost -p 5432
```

**Authentication failed:**
```bash
# Verify secret exists
kubectl get secret postgres-secret

# Check secret contents
kubectl get secret postgres-secret -o yaml

# Recreate secret if needed
kubectl delete secret postgres-secret
# Then re-run install.sh or create manually
```

---

## API Reference

### Health Check
- `GET /health` - Service health status

### Legacy Endpoints (Deprecated)
- `POST /clusters` - ⚠️ **Deprecated** - Use `/cnpg/clusters` instead
- `GET /clusters` - ⚠️ **Deprecated** - Use `/cnpg/clusters` instead

## Architecture

The service uses a modular plugin-based architecture:

```
src/
├── handlers/          # HTTP request handlers for each resource type
│   ├── cnpg.rs       # CNPG-specific endpoints
│   ├── kubeflow.rs   # Kubeflow notebook endpoints
│   └── health.rs     # Health check endpoint
├── resources/         # Resource management logic
│   ├── cnpg.rs       # CNPG resource manager implementation
│   ├── kubeflow.rs   # Kubeflow notebook manager implementation
│   └── mod.rs        # ResourceManager trait definition
├── models/           # Data models and request/response types
│   ├── cnpg.rs       # CNPG-specific models
│   ├── kubeflow.rs   # Kubeflow notebook models
│   └── mod.rs        # Common models
├── error.rs          # Centralized error handling
├── utils/            # Shared utilities
└── main.rs           # Application entry point and routing
```

## Contributing

1. Follow the modular architecture patterns
2. Add appropriate error handling  
3. Include tests for new functionality
4. Update documentation for new resource types
5. Ensure backward compatibility for existing APIs

## License

[Specify your license here]