# Proxy Re-encryption Enclave Application Makefile

.PHONY: help build test run clean docker docker-dev docker-test docs fmt clippy audit

# Default target
help: ## Show this help message
	@echo "Proxy Re-encryption Enclave Application"
	@echo ""
	@echo "Available commands:"
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  %-15s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

# Development commands
build: ## Build the application in debug mode
	cargo build

build-release: ## Build the application in release mode
	cargo build --release

build-musl: ## Build for musl target (AWS Nitro Enclave compatible)
	cargo build --target=x86_64-unknown-linux-musl --release

test: ## Run all tests
	cargo test

test-integration: ## Run integration tests only
	cargo test --test integration_client

test-coverage: ## Run tests with coverage report
	cargo install cargo-tarpaulin
	cargo tarpaulin --out Lcov

bench: ## Run benchmarks
	cargo bench

run-client: ## Run the client (parent instance)
	cargo run -- client --port 8000 --cid 3

run-server: ## Run the server (enclave instance)
	cargo run -- server --port 5005

run-dev: ## Run in development mode with debug logging
	RUST_LOG=debug cargo run -- client --port 8000 --cid 3

# Code quality commands
fmt: ## Format code
	cargo fmt

fmt-check: ## Check code formatting
	cargo fmt --all -- --check

clippy: ## Run clippy linter
	cargo clippy --all-targets --all-features

clippy-strict: ## Run clippy with strict settings
	cargo clippy --all-targets --all-features -- -W clippy::pedantic -D warnings

audit: ## Run security audit
	cargo install cargo-audit
	cargo audit

udeps: ## Check for unused dependencies
	cargo install cargo-udeps --locked
	cargo +nightly udeps --all-targets

# Documentation commands
docs: ## Generate and open documentation
	cargo doc --open

docs-private: ## Generate documentation including private items
	cargo doc --document-private-items --open

# Docker commands
docker-build: ## Build Docker image
	docker build -t proxy-reencryption-enclave .

docker-run: ## Run Docker container
	docker run -p 8000:8000 --name proxy-reencryption proxy-reencryption-enclave

docker-dev: ## Build and run development Docker image
	docker build --target development -t proxy-reencryption-dev .
	docker run -p 8000:8000 -v $(PWD):/app proxy-reencryption-dev

docker-test: ## Run tests in Docker
	docker build --target test -t proxy-reencryption-test .

docker-clean: ## Clean Docker resources
	docker stop proxy-reencryption proxy-reencryption-dev 2>/dev/null || true
	docker rm proxy-reencryption proxy-reencryption-dev 2>/dev/null || true
	docker rmi proxy-reencryption-enclave proxy-reencryption-dev proxy-reencryption-test 2>/dev/null || true

# Docker Compose commands
compose-up: ## Start services with docker-compose
	docker-compose up -d

compose-dev: ## Start development services
	docker-compose --profile dev up

compose-test: ## Run tests with docker-compose
	docker-compose --profile test up

compose-down: ## Stop and remove services
	docker-compose down

compose-logs: ## View service logs
	docker-compose logs -f

# AWS Nitro Enclave commands
enclave-build: ## Build enclave image
	nitro-cli build-enclave \
		--docker-dir ./docker-enclave \
		--docker-uri proxy-reencryption-enclave \
		--output-file enclave.eif

enclave-run: ## Run enclave
	nitro-cli run-enclave \
		--eif-path enclave.eif \
		--cpu-count 2 \
		--enclave-cid 16 \
		--memory 2048 \
		--debug-mode

enclave-stop: ## Stop enclave
	nitro-cli terminate-enclave --enclave-id $(nitro-cli describe-enclaves | jq -r '.[0].EnclaveID')

enclave-logs: ## View enclave logs
	nitro-cli console --enclave-id $(nitro-cli describe-enclaves | jq -r '.[0].EnclaveID')

# Utility commands
clean: ## Clean build artifacts
	cargo clean
	docker-clean

install-deps: ## Install development dependencies
	cargo install cargo-tarpaulin
	cargo install cargo-audit
	cargo install cargo-watch
	cargo install cargo-udeps --locked

update-deps: ## Update dependencies
	cargo update

check-all: fmt-check clippy audit test ## Run all quality checks

# CI/CD simulation
ci: ## Simulate CI pipeline locally
	@echo "Running CI pipeline..."
	@cargo fmt --all -- --check
	@cargo clippy --all-targets --all-features -- -D warnings
	@cargo audit
	@cargo test
	@cargo build --release
	@echo "CI pipeline completed successfully!"

# Help for specific targets
help-targets:
	@echo "Common development workflow:"
	@echo "  1. make build          # Build the application"
	@echo "  2. make test           # Run tests"
	@echo "  3. make run-client     # Start client"
	@echo "  4. make run-server     # Start server (in another terminal)"
	@echo ""
	@echo "Code quality:"
	@echo "  make fmt               # Format code"
	@echo "  make clippy           # Run linter"
	@echo "  make audit            # Security audit"
	@echo ""
	@echo "Docker:"
	@echo "  make docker-build     # Build image"
	@echo "  make docker-run       # Run container"
	@echo "  make compose-up       # Start with docker-compose"
	@echo ""
	@echo "AWS Nitro Enclave:"
	@echo "  make enclave-build    # Build enclave image"
	@echo "  make enclave-run      # Run enclave"
	@echo ""
	@echo "Quality checks:"
	@echo "  make check-all        # Run all checks"
	@echo "  make ci              # Simulate CI pipeline"