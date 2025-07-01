#!/bin/bash

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to check if we're using the correct Talos cluster
verify_talos_cluster() {
    print_status "Verifying Talos cluster connection..."
    
    # Check if talosctl is available
    if ! command_exists talosctl; then
        print_error "talosctl not found. Please install Talos CLI first."
        exit 1
    fi
    
    # Check current kubeconfig context
    current_context=$(kubectl config current-context 2>/dev/null || echo "")
    if [[ -z "$current_context" ]]; then
        print_error "No kubectl context found. Please configure kubeconfig first."
        exit 1
    fi
    
    # Verify this is a Talos cluster by checking talosctl config
    if ! talosctl config info >/dev/null 2>&1; then
        print_error "talosctl not configured or not connected to a Talos cluster."
        exit 1
    fi
    
    # Get Talos cluster info
    talos_context=$(talosctl config info | grep "Current context:" | awk '{print $3}')
    
    print_status "Current kubectl context: $current_context"
    print_status "Current Talos context: $talos_context"
    
    # Verify kubectl can connect to cluster
    if ! kubectl cluster-info >/dev/null 2>&1; then
        print_error "Cannot connect to Kubernetes cluster. Please check your kubeconfig."
        exit 1
    fi
    
    # Check if this looks like a Talos cluster (check for Talos-specific resources or labels)
    if kubectl get nodes -o jsonpath='{.items[0].metadata.labels}' 2>/dev/null | grep -q "node.kubernetes.io/instance-type"; then
        print_success "Connected to Talos Kubernetes cluster"
    else
        print_warning "Could not verify this is a Talos cluster, but proceeding anyway..."
    fi
}

# Function to install CloudNativePG operator
install_cnpg() {
    print_status "Installing CloudNativePG operator..."
    
    # Check if CNPG is already installed
    if kubectl get crd clusters.postgresql.cnpg.io >/dev/null 2>&1; then
        print_warning "CloudNativePG CRD already exists, skipping installation"
        return 0
    fi
    
    # Install CNPG operator
    print_status "Downloading and applying CNPG operator manifests..."
    kubectl apply --server-side -f https://raw.githubusercontent.com/cloudnative-pg/cloudnative-pg/release-1.24/releases/cnpg-1.24.1.yaml
    
    # Wait for CNPG operator to be ready
    print_status "Waiting for CNPG operator to be ready..."
    kubectl wait --for=condition=Available deployment/cnpg-controller-manager -n cnpg-system --timeout=300s
    
    print_success "CloudNativePG operator installed successfully"
}

# Function to create PostgreSQL secret
create_postgres_secret() {
    print_status "Creating PostgreSQL secret..."
    
    # Check if secret already exists
    if kubectl get secret postgres-secret >/dev/null 2>&1; then
        print_warning "PostgreSQL secret already exists, skipping creation"
        return 0
    fi
    
    # Generate a secure password
    POSTGRES_PASSWORD=$(openssl rand -base64 32 | tr -d "=+/" | cut -c1-25)
    
    # Create the secret
    kubectl create secret generic postgres-secret \
        --from-literal=username=postgres \
        --from-literal=password="$POSTGRES_PASSWORD"
    
    print_success "PostgreSQL secret created with password: $POSTGRES_PASSWORD"
    print_warning "Please save this password securely!"
}

# Function to create storage class
create_storage_class() {
    print_status "Creating storage class for notebook workspaces..."
    
    # Check if storage class already exists
    if kubectl get storageclass local-storage >/dev/null 2>&1; then
        print_warning "Storage class 'local-storage' already exists, skipping creation"
        return 0
    fi
    
    # Create storage class
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
    
    print_success "Storage class created successfully"
}

# Function to install Kubeflow components
install_kubeflow() {
    print_status "Installing Kubeflow components for notebooks..."
    
    # Check if Kubeflow notebook CRD exists
    if kubectl get crd notebooks.kubeflow.org >/dev/null 2>&1; then
        print_warning "Kubeflow notebook CRD already exists, checking controller..."
    else
        # Create kubeflow namespace if it doesn't exist
        kubectl create namespace kubeflow --dry-run=client -o yaml | kubectl apply -f -
        
        # Install Kubeflow notebook CRDs
        print_status "Installing Kubeflow notebook CRDs..."
        kubectl apply -f https://raw.githubusercontent.com/kubeflow/kubeflow/v1.8.0/components/notebook-controller/config/crd/bases/kubeflow.org_notebooks.yaml
    fi
    
    # Check if controller is already running
    if kubectl get deployment notebook-controller -n kubeflow >/dev/null 2>&1; then
        print_warning "Kubeflow notebook controller already exists, checking RBAC..."
    else
        print_status "Installing Kubeflow notebook controller..."
    fi
    
    # Create service account if it doesn't exist
    if ! kubectl get serviceaccount notebook-controller-service-account -n kubeflow >/dev/null 2>&1; then
        kubectl create serviceaccount notebook-controller-service-account -n kubeflow
    fi
    
    # Apply comprehensive RBAC for notebook controller
    print_status "Configuring RBAC permissions..."
    cat <<EOF | kubectl apply -f -
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: notebook-controller-role
rules:
- apiGroups: [""]
  resources: ["pods", "services", "endpoints", "persistentvolumeclaims", "events", "configmaps", "secrets"]
  verbs: ["*"]
- apiGroups: ["apps"]
  resources: ["deployments", "daemonsets", "replicasets", "statefulsets"]
  verbs: ["*"]
- apiGroups: ["kubeflow.org"]
  resources: ["notebooks", "notebooks/status"]
  verbs: ["*"]
- apiGroups: ["networking.k8s.io"]
  resources: ["ingresses", "networkpolicies"]
  verbs: ["*"]
- apiGroups: [""]
  resources: ["nodes"]
  verbs: ["get", "list", "watch"]
---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRoleBinding
metadata:
  name: notebook-controller-rolebinding
roleRef:
  apiGroup: rbac.authorization.k8s.io
  kind: ClusterRole
  name: notebook-controller-role
subjects:
- kind: ServiceAccount
  name: notebook-controller-service-account
  namespace: kubeflow
EOF
    
    # Deploy notebook controller with proper security context
    print_status "Deploying Kubeflow notebook controller..."
    cat <<EOF | kubectl apply -f -
apiVersion: apps/v1
kind: Deployment
metadata:
  name: notebook-controller
  namespace: kubeflow
  labels:
    app: notebook-controller
spec:
  replicas: 1
  selector:
    matchLabels:
      app: notebook-controller
  template:
    metadata:
      labels:
        app: notebook-controller
    spec:
      serviceAccountName: notebook-controller-service-account
      securityContext:
        runAsNonRoot: true
        runAsUser: 65532
        fsGroup: 65532
      containers:
      - name: manager
        image: kubeflownotebookswg/notebook-controller:v1.8.0
        command:
        - /manager
        env:
        - name: USE_ISTIO
          value: "false"
        - name: ISTIO_GATEWAY
          value: "kubeflow/kubeflow-gateway"
        resources:
          limits:
            cpu: 100m
            memory: 128Mi
          requests:
            cpu: 100m
            memory: 128Mi
        securityContext:
          allowPrivilegeEscalation: false
          capabilities:
            drop:
            - ALL
          runAsNonRoot: true
          seccompProfile:
            type: RuntimeDefault
EOF
    
    # Wait for notebook controller to be ready
    print_status "Waiting for Kubeflow notebook controller to be ready..."
    kubectl wait --for=condition=Available deployment/notebook-controller -n kubeflow --timeout=300s
    
    print_success "Kubeflow components installed successfully"
}

# Function to verify installations
verify_installations() {
    print_status "Verifying installations..."
    
    # Check CNPG
    if kubectl get crd clusters.postgresql.cnpg.io >/dev/null 2>&1; then
        print_success "✓ CloudNativePG CRD is available"
    else
        print_error "✗ CloudNativePG CRD not found"
        return 1
    fi
    
    # Check CNPG operator
    if kubectl get deployment cnpg-controller-manager -n cnpg-system >/dev/null 2>&1; then
        cnpg_ready=$(kubectl get deployment cnpg-controller-manager -n cnpg-system -o jsonpath='{.status.readyReplicas}')
        if [[ "$cnpg_ready" == "1" ]]; then
            print_success "✓ CNPG controller is running"
        else
            print_warning "⚠ CNPG controller exists but not ready"
        fi
    else
        print_error "✗ CNPG controller not found"
        return 1
    fi
    
    # Check Kubeflow
    if kubectl get crd notebooks.kubeflow.org >/dev/null 2>&1; then
        print_success "✓ Kubeflow notebook CRD is available"
    else
        print_error "✗ Kubeflow notebook CRD not found"
        return 1
    fi
    
    # Check Kubeflow controller
    if kubectl get deployment notebook-controller -n kubeflow >/dev/null 2>&1; then
        kf_ready=$(kubectl get deployment notebook-controller -n kubeflow -o jsonpath='{.status.readyReplicas}')
        if [[ "$kf_ready" == "1" ]]; then
            print_success "✓ Kubeflow notebook controller is running"
        else
            print_warning "⚠ Kubeflow notebook controller exists but not ready"
        fi
    else
        print_error "✗ Kubeflow notebook controller not found"
        return 1
    fi
    
    # Check PostgreSQL secret
    if kubectl get secret postgres-secret >/dev/null 2>&1; then
        print_success "✓ PostgreSQL secret exists"
    else
        print_warning "✗ PostgreSQL secret not found"
    fi
    
    # Check storage class
    if kubectl get storageclass local-storage >/dev/null 2>&1; then
        print_success "✓ Storage class 'local-storage' is available"
    else
        print_warning "✗ Storage class 'local-storage' not found"
    fi
}

# Function to run verification tests
run_verification_tests() {
    print_status "Running verification tests..."
    
    # Test 1: Create a test notebook without volume
    print_status "Test 1: Creating test notebook..."
    cat <<EOF | kubectl apply -f -
apiVersion: kubeflow.org/v1
kind: Notebook
metadata:
  name: install-test-notebook
  namespace: default
spec:
  template:
    spec:
      containers:
      - name: notebook
        image: jupyter/minimal-notebook:latest
        resources:
          requests:
            cpu: 100m
            memory: 128Mi
        ports:
        - containerPort: 8888
          name: notebook-port
          protocol: TCP
EOF
    
    # Wait a bit and check if notebook is being processed
    sleep 10
    if kubectl get notebook install-test-notebook >/dev/null 2>&1; then
        print_success "✓ Test notebook created successfully"
        
        # Check if StatefulSet was created
        if kubectl get statefulset install-test-notebook >/dev/null 2>&1; then
            print_success "✓ Notebook controller is processing notebooks"
        else
            print_warning "⚠ StatefulSet not created yet (controller may still be starting)"
        fi
    else
        print_error "✗ Failed to create test notebook"
    fi
    
    # Clean up test resources
    print_status "Cleaning up test resources..."
    kubectl delete notebook install-test-notebook --ignore-not-found=true
    kubectl delete statefulset install-test-notebook --ignore-not-found=true
    kubectl delete pod install-test-notebook-0 --ignore-not-found=true
}

# Main installation function
main() {
    echo "=================================================="
    echo "  K8s Resource Manager - Installation Script"
    echo "=================================================="
    echo ""
    
    # Check prerequisites
    print_status "Checking prerequisites..."
    
    if ! command_exists kubectl; then
        print_error "kubectl not found. Please install kubectl first."
        exit 1
    fi
    
    if ! command_exists openssl; then
        print_error "openssl not found. Please install openssl first."
        exit 1
    fi
    
    # Verify Talos cluster
    verify_talos_cluster
    
    echo ""
    print_status "Starting installation process..."
    echo ""
    
    # Install components
    install_cnpg
    echo ""
    
    create_postgres_secret
    echo ""
    
    create_storage_class
    echo ""
    
    install_kubeflow
    echo ""
    
    # Verify everything is working
    verify_installations
    echo ""
    
    # Run verification tests
    run_verification_tests
    
    echo ""
    echo "=================================================="
    print_success "Installation completed successfully!"
    echo "=================================================="
    echo ""
    print_status "Next steps:"
    echo "1. Start the k8s-resource-manager service with: cargo run"
    echo "2. Test CNPG clusters: curl http://localhost:3000/cnpg/clusters"
    echo "3. Test Kubeflow notebooks: curl http://localhost:3000/kubeflow/notebooks"
    echo ""
    print_status "Create your first notebook:"
    echo 'curl -X POST http://localhost:3000/kubeflow/notebooks \'
    echo '-H "Content-Type: application/json" \'
    echo '-d "{"name": "my-notebook", "image": "jupyter/scipy-notebook:latest", "cpu_request": "500m", "memory_request": "1Gi"}"'
    echo ""
    print_status "Access notebook:"
    echo "kubectl wait --for=condition=Ready pod/my-notebook-0 --timeout=300s"
    echo "kubectl port-forward pod/my-notebook-0 8888:8888"
    echo "# Then open http://localhost:8888 in your browser"
    echo ""
    print_status "API endpoints available at: http://localhost:3000"
    echo "  - Health check: GET /health"
    echo "  - CNPG clusters: /cnpg/clusters"
    echo "  - Kubeflow notebooks: /kubeflow/notebooks"
    echo ""
}

# Run main function
main "$@"