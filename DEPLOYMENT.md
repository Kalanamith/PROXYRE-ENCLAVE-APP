# 🚢 Deployment Guide

This guide covers various deployment strategies for the Proxy Re-encryption Enclave Application.

## Production Deployment

### 1. Build for AWS Nitro Enclave

```bash
# Build for musl target
cargo build --target=x86_64-unknown-linux-musl --release

# Create enclave image
nitro-cli build-enclave \
  --docker-dir ./ \
  --docker-uri proxy-reencryption-enclave \
  --output-file enclave.eif
```

### 2. Deploy to AWS Nitro Enclave

```bash
# Run the enclave
nitro-cli run-enclave \
  --eif-path enclave.eif \
  --cpu-count 2 \
  --enclave-cid 16 \
  --memory 2048 \
  --debug-mode

# Expected output:
# {
#   "Measurements": {
#     "HashAlgorithm": "Sha384 { ... }",
#     "PCR0": "e07aa8d3344dac11daa480dc3fb67d5c4296c384c7583d8d0a56b5656123fcfdaf668c85888229d6df19b4a7f4892bac",
#     "PCR1": "bcdf05fefccaa8e55bf2c8d6dee9e79bbff31e34bf28a99aa19e6b29c37ee80b214a414b7607236edf26fcb78654e63f",
#     "PCR2": "f7b8216534d0e6bdd1c2a338e71073da8e13b7dccec1dbe749cbc95edd6ea29903a2b463a60f95ad781e231b1b09acd3"
#   }
# }
```

### 3. Connect to the Enclave

```bash
# Run the parent instance
./proxy_reencyption_enclave_app client --cid 16 --port 5005

# Expected output:
# 🔧 Configured for production.
#     => address: 0.0.0.0
#     => port: 8000
#     => workers: 12
#     => secret key: generated
# 🚀 Rocket has launched from http://0.0.0.0:8000
```

## 🚢 Deployment Strategies

### AWS Nitro Enclave Deployment

1. **Prerequisites**

```bash
# Install AWS Nitro CLI
sudo yum install aws-nitro-cli

# Configure AWS credentials
aws configure
```

2. **Build and Deploy**

```bash
# Build enclave image
nitro-cli build-enclave \
  --docker-dir ./ \
  --docker-uri proxy-reencryption \
  --output-file enclave.eif

# Deploy enclave
nitro-cli run-enclave \
  --eif-path enclave.eif \
  --cpu-count 2 \
  --enclave-cid 16 \
  --memory 2048
```

### Docker Swarm/Kubernetes

```yaml
# kubernetes-deployment.yml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: proxy-reencryption
spec:
  replicas: 3
  selector:
    matchLabels:
      app: proxy-reencryption
  template:
    metadata:
      labels:
        app: proxy-reencryption
    spec:
      containers:
      - name: proxy-reencryption
        image: your-registry/proxy-reencryption:latest
        ports:
        - containerPort: 8000
        env:
        - name: RUST_LOG
          value: "info"
        resources:
          limits:
            memory: "512Mi"
            cpu: "500m"
          requests:
            memory: "256Mi"
            cpu: "250m"
```

### Cloud Deployment

#### AWS ECS/Fargate

```hcl
# Terraform example
resource "aws_ecs_task_definition" "proxy_reencryption" {
  family                   = "proxy-reencryption"
  network_mode             = "awsvpc"
  requires_compatibilities = ["FARGATE"]
  cpu                      = 512
  memory                   = 1024

  container_definitions = jsonencode([{
    name  = "proxy-reencryption"
    image = "${aws_ecr_repository.proxy_reencryption.repository_url}:latest"
    portMappings = [{
      containerPort = 8000
      hostPort      = 8000
    }]
  }])
}
```

## 🔒 Security Considerations

- **Hardware Security**: Runs in AWS Nitro Enclave with cryptographic attestation
- **Memory Protection**: All sensitive data processed in enclave memory
- **Network Security**: Vsock communication between parent and enclave
- **Cryptographic Security**: Uses industry-standard cryptographic primitives
- **Audit Trail**: Comprehensive logging for security monitoring

## 📊 Monitoring

### Health Checks

```bash
# Health check endpoint
curl http://localhost:8000/

# Expected response: "Hola!!!"
```

### Logs

```bash
# Set log level
export RUST_LOG=debug

# View application logs
docker logs proxy-reencryption
```

### Metrics

The application provides basic health check endpoints. For production monitoring, consider integrating with:

- AWS CloudWatch
- Prometheus/Grafana
- ELK Stack
