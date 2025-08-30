# Proxy Re-encryption Enclave Application

[![CI](https://github.com/your-username/proxy-reencryption-enclave-app/actions/workflows/ci.yml/badge.svg)](https://github.com/your-username/proxy-reencryption-enclave-app/actions/workflows/ci.yml)
[![Code Quality](https://github.com/your-username/proxy-reencryption-enclave-app/actions/workflows/code-quality.yml/badge.svg)](https://github.com/your-username/proxy-reencryption-enclave-app/actions/workflows/code-quality.yml)
[![Security Audit](https://github.com/your-username/proxy-reencryption-enclave-app/actions/workflows/scheduled.yml/badge.svg)](https://github.com/your-username/proxy-reencryption-enclave-app/actions/workflows/scheduled.yml)

A secure proxy re-encryption service built in Rust that runs inside AWS Nitro Enclaves. This application provides cryptographic proxy re-encryption capabilities with a REST API interface, ensuring data privacy and security through hardware-backed trusted execution environments.

## 🚀 Features

- **🔐 Cryptographic Proxy Re-encryption**: Full proxy re-encryption implementation using recrypt library
- **🏰 AWS Nitro Enclave Support**: Runs in trusted execution environment
- **🌐 REST API**: HTTP endpoints for key generation and content transformation
- **🔄 Real-time Communication**: Vsock-based communication between parent and enclave instances
- **📊 Structured Logging**: Comprehensive logging with configurable levels
- **🧪 Comprehensive Testing**: 71+ unit tests with integration test coverage
- **🐳 Docker Support**: Containerized deployment with multi-stage builds
- **⚡ High Performance**: Asynchronous processing with Tokio runtime
- **🔒 Security Focused**: Security audit integration and dependency scanning

## 🏗️ Architecture

The application consists of two main components:

### Parent Instance (Client)

- Runs outside the enclave
- Provides REST API endpoints
- Handles HTTP requests and responses
- Communicates with enclave via Vsock

### Enclave Instance (Server)

- Runs inside AWS Nitro Enclave
- Performs cryptographic operations
- Handles proxy re-encryption transformations
- Communicates securely with parent instance

## 📋 Prerequisites

- **Rust**: 1.70.0 or later
- **Docker**: For containerized deployment
- **AWS Nitro CLI**: For enclave deployment
- **Linux**: For enclave execution (AWS Nitro Enclaves)

### System Requirements

This app uses AWS Nitro Enclaves to create an environment where a client can securely request the creation of cryptographic keys inside an enclave.

It enables content reencryption using Rust's recrypt library.

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustlang.org | sh

# Install AWS Nitro CLI (on Amazon Linux 2)
sudo yum install aws-nitro-cli

# Install Docker
curl -fsSL https://get.docker.com | sh
```

## 🛠️ Installation

### Local Development

1. **Clone the repository**

```bash
git clone https://github.com/your-username/proxy-reencryption-enclave-app.git
cd proxy-reencryption-enclave-app
```

2. **Install dependencies**

```bash
cargo build
```

3. **Run tests**

```bash
cargo test
```

### Docker Deployment

#### Quick Start with Docker Compose

```bash
# Start the production service
make compose-up

# Or manually:
docker-compose up -d proxy-reencryption

# View logs
make compose-logs
# Or: docker-compose logs -f
```

#### Development with Docker

```bash
# Start development service with hot reload
make compose-dev

# Or manually:
docker-compose --profile dev up proxy-reencryption-dev
```

#### Testing with Docker

```bash
# Run tests in container
make compose-test

# Or manually:
docker-compose --profile test up proxy-reencryption-test
```

#### Manual Docker Commands

```bash
# Build production image
make docker-build

# Build development image
make docker-dev

# Run production container
make docker-run

# Clean up Docker resources
make docker-clean
```

### Docker Architecture

The application uses multi-stage Docker builds with the following stages:

- **`runtime`**: Production-ready minimal image
- **`development`**: Development image with hot reload
- **`test`**: Testing image with test dependencies
- **`enclave-builder`**: Specialized for AWS Nitro Enclave

#### Production Dockerfile Features

- **Multi-stage build** for optimized image size
- **Security hardening** with non-root user
- **Health checks** for container orchestration
- **Minimal attack surface** with slim base image
- **Proper signal handling** for graceful shutdown

## 🚀 Usage

### Development Mode

#### Start the Client (Parent Instance)

```bash
cargo run -- client --port 8000 --cid 3
```

#### Start the Server (Enclave Instance)

```bash
cargo run -- server --port 5005
```

### Production Deployment

#### 1. Build for AWS Nitro Enclave

```bash
# Build for musl target
cargo build --target=x86_64-unknown-linux-musl --release

# Create enclave image
nitro-cli build-enclave \
  --docker-dir ./ \
  --docker-uri proxy-reencryption-enclave \
  --output-file enclave.eif
```

#### 2. Deploy to AWS Nitro Enclave

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

#### 3. Connect to the Enclave

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

## 📡 API Documentation

The application provides a REST API for cryptographic operations:

### Endpoints

#### `GET /`

**Health Check**

```bash
curl http://localhost:8000/
# Response: "Hola!!!"
```

#### `GET /get-keys`

**Generate Key Pair**

```bash
curl http://localhost:8000/get-keys
# Response: {"private_key":[...],"public_key_x":[...],"public_key_y":[...]}
```

#### `POST /upload`

**Upload Content for Encryption** (Work in Progress)

```bash
curl -X POST http://localhost:8000/upload \
  -H "Content-Type: application/json" \
  -d '{"data": "your content here"}'
```

#### `POST /fetch`

**Transform Encrypted Content**

```bash
curl -X POST http://localhost:8000/fetch \
  -H "Content-Type: application/json" \
  -d '{
    "initial_private_key": [...],
    "initial_public_key_x": [...],
    "initial_public_key_y": [...],
    "delegatee_public_key_x": [...],
    "delegatee_public_key_y": [...],
    "resource": [...]
  }'
```

### API Response Format

```json
{
  "transformed_object": "encrypted_data_string",
  "public_key": {
    "public_key_x": "base64_encoded_x",
    "public_key_y": "base64_encoded_y"
  },
  "encrypted_temp_key": "encrypted_key_data",
  "random_transform_public_key": {
    "public_key_x": "transformed_x",
    "public_key_y": "transformed_y"
  }
}
```

## ⚙️ Configuration

### Environment Variables

```bash
# Logging configuration
RUST_LOG=info,proxy_reencyption_enclave_app=debug

# Server configuration
ROCKET_ADDRESS=0.0.0.0
ROCKET_PORT=8000
ROCKET_WORKERS=12

# Enclave configuration
ENCLAVE_CID=16
ENCLAVE_MEMORY=2048
ENCLAVE_CPU_COUNT=2
```

### Command Line Options

#### Client Mode

```bash
proxy_reencyption_enclave_app client --help
Usage: proxy_reencyption_enclave_app client [OPTIONS] --port <port> --cid <cid>

Options:
      --port <port>  Port to connect to the enclave
      --cid <cid>    Enclave connection ID
  -h, --help         Print help information
```

#### Server Mode

```bash
proxy_reencyption_enclave_app server --help
Usage: proxy_reencyption_enclave_app server [OPTIONS] --port <port>

Options:
      --port <port>  Port to listen on
  -h, --help         Print help information
```

## 🧪 Testing

### Using Make Commands (Recommended)

```bash
# Run all tests
make test

# Run integration tests only
make test-integration

# Run tests with coverage
make test-coverage

# Run benchmarks
make bench

# Run all quality checks + tests
make check-all

# Simulate CI pipeline locally
make ci
```

### Manual Testing Commands

#### Run All Tests

```bash
cargo test
```

#### Run with Coverage

```bash
# Install cargo-tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Lcov
```

#### Integration Tests

```bash
cargo test --test integration_client
```

#### Performance Benchmarks

```bash
cargo bench
```

### Test Coverage

The application includes comprehensive test coverage with:

- **71 unit tests** covering all major functionality
- **1 integration test** for end-to-end validation
- **Test coverage** for models, utilities, and protocols
- **Documentation tests** validating code examples

## 🔧 Development

### Code Quality Tools

#### Using Make Commands (Recommended)

```bash
# Format and check code
make fmt              # Format code
make fmt-check        # Check formatting

# Lint and audit
make clippy          # Run clippy linter
make clippy-strict   # Run with strict settings
make audit           # Security audit

# Check dependencies
make udeps           # Check unused dependencies

# Documentation
make docs            # Generate docs
make docs-private    # Include private items
```

#### Manual Commands

```bash
# Format code
cargo fmt
cargo fmt --all -- --check

# Lint code
cargo clippy --all-targets --all-features
cargo clippy --all-targets --all-features -- -W clippy::pedantic

# Security audit
cargo install cargo-audit
cargo audit

# Check documentation
cargo doc --open
cargo doc --document-private-items --open
```

### Project Structure

```
src/
├── main.rs              # Application entry point
├── lib.rs               # Core library functionality
├── command_parser.rs    # CLI argument parsing
├── protocol_helpers.rs  # Network protocol utilities
├── utils.rs            # Utility functions and traits
├── models.rs           # Data structures and serialization
└── proto/              # Protocol buffer definitions

.github/
├── workflows/          # GitHub Actions CI/CD pipelines
├── dependabot.yml      # Automated dependency updates

docker-enclave/         # AWS Nitro Enclave Docker configuration
Dockerfile             # Multi-stage Docker build
docker-compose.yml     # Container orchestration
Makefile              # Development and deployment automation
```

### Development Workflow

#### Quick Start

```bash
# Clone and setup
git clone <repository-url>
cd proxy-reencryption-enclave-app
make install-deps

# Development cycle
make build          # Build application
make test           # Run tests
make run-client     # Start client
make run-server     # Start server (separate terminal)

# Code quality
make check-all      # Run all quality checks
make fmt           # Format code
make clippy        # Lint code
```

#### Available Make Commands

```bash
# Core development
make build          # Build in debug mode
make build-release  # Build in release mode
make test           # Run all tests
make run-client     # Start client instance
make run-server     # Start server instance
make clean          # Clean build artifacts

# Code quality
make fmt            # Format code
make clippy         # Run linter
make audit          # Security audit
make docs           # Generate documentation

# Docker operations
make docker-build   # Build Docker image
make docker-run     # Run Docker container
make compose-up     # Start with docker-compose

# AWS Nitro Enclave
make enclave-build  # Build enclave image
make enclave-run    # Run enclave

# CI simulation
make ci            # Simulate full CI pipeline
```

Use `make help` to see all available commands with descriptions.

### Adding New Features

1. **Create a feature branch**

```bash
git checkout -b feature/new-feature
```

2. **Add tests first** (TDD approach)

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_new_feature() {
        // Test implementation
    }
}
```

3. **Implement the feature**
4. **Update documentation**
5. **Run full test suite**

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

## 🤝 Contributing

1. **Fork the repository**
2. **Create a feature branch**

```bash
git checkout -b feature/amazing-feature
```

3. **Make your changes**
4. **Add tests** for new functionality
5. **Ensure all tests pass**

```bash
cargo test
cargo clippy
cargo fmt --check
```

6. **Update documentation**
7. **Commit your changes**

```bash
git commit -m "feat: add amazing feature"
```

8. **Push to the branch**

```bash
git push origin feature/amazing-feature
```

9. **Open a Pull Request**

### Commit Message Format

This project follows [Conventional Commits](https://conventionalcommits.org/):

```bash
feat: add new encryption endpoint
fix: resolve memory leak in key generation
docs: update API documentation
test: add unit tests for crypto operations
refactor: simplify error handling logic
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [AWS Nitro Enclaves](https://aws.amazon.com/ec2/nitro/) for the trusted execution environment
- [Recrypt](https://github.com/twardoch/recrypt) library for proxy re-encryption implementation
- [Rocket](https://rocket.rs/) web framework for the HTTP API
- [Tokio](https://tokio.rs/) for asynchronous runtime

## 📞 Support

For questions, issues, or contributions:

- 📧 **Email**: <dev@proxy-reencyption.io>
- 🐛 **Issues**: [GitHub Issues](https://github.com/your-username/proxy-reencryption-enclave-app/issues)
- 📖 **Documentation**: [GitHub Wiki](https://github.com/your-username/proxy-reencryption-enclave-app/wiki)
- 💬 **Discussions**: [GitHub Discussions](https://github.com/your-username/proxy-reencryption-enclave-app/discussions)

---

**⚠️ Security Notice**: This application handles cryptographic operations. Ensure proper key management and secure deployment practices in production environments.

