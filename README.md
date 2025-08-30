# proxy-reencyption enclave app

## Run server
```bash
cargo run -- server --port 5005
```

## Run client

```bash
cargo run -- client --cid 3 --port 5005
```

# Deployment

```bash
cargo build --target=x86_64-unknown-linux-musl --release
```

```bash
$ nitro-cli build-enclave --docker-dir ./ --docker-uri dark-server --output-file dark.eif
Start building the Enclave Image...
Enclave Image successfully created.
{
  "Measurements": {
    "HashAlgorithm": "Sha384 { ... }",
    "PCR0": "e07aa8d3344dac11daa480dc3fb67d5c4296c384c7583d8d0a56b5656123fcfdaf668c85888229d6df19b4a7f4892bac",
    "PCR1": "bcdf05fefccaa8e55bf2c8d6dee9e79bbff31e34bf28a99aa19e6b29c37ee80b214a414b7607236edf26fcb78654e63f",
    "PCR2": "f7b8216534d0e6bdd1c2a338e71073da8e13b7dccec1dbe749cbc95edd6ea29903a2b463a60f95ad781e231b1b09acd3"
  }
}

```

```bash
nitro-cli run-enclave --eif-path dark.eif --cpu-count 2 --enclave-cid 6 --memory 256 --debug-mode
```


### Run Client

```bash
 ./proxy-reencyption-enclave-app client --cid 6 --port 5005
🔧 Configured for production.
    => address: 0.0.0.0
    => port: 8000
    => log: critical
    => workers: 1
    => secret key: generated
    => limits: forms = 32KiB
    => keep-alive: 5s
    => read timeout: 5s
    => write timeout: 5s
    => tls: disabled
Warning: environment is 'production', but no `secret_key` is configured
🚀 Rocket has launched from http://0.0.0.0:8000

```

## CI/CD

This project uses GitHub Actions for continuous integration and deployment. The following workflows are configured:

### Workflows

#### 🚀 CI (`ci.yml`)
- **Triggers**: Push to `main`/`develop`, Pull Requests
- **Platforms**: Ubuntu, macOS, Windows
- **Rust Versions**: Stable, Beta, Nightly (Ubuntu only)
- **Checks**:
  - Code formatting (`cargo fmt`)
  - Linting (`cargo clippy`)
  - Build verification
  - Unit tests
  - Integration tests
  - Security audit
  - Documentation build

#### 🔒 Code Quality (`code-quality.yml`)
- **Triggers**: Push to `main`/`develop`, Pull Requests
- **Checks**:
  - Strict clippy with pedantic warnings
  - Unused dependencies check
  - Security vulnerabilities scan
  - License compatibility check
  - MSRV (Minimum Supported Rust Version) validation
  - Documentation completeness

#### 📋 Pull Request Checks (`pr-checks.yml`)
- **Triggers**: Pull Requests
- **Checks**:
  - Commit message format validation
  - PR size limits
  - Dependency review
  - Test coverage requirements
  - TODO comment detection

#### 📦 Release (`release.yml`)
- **Triggers**: Git tags matching `v*.*.*`
- **Actions**:
  - Build release binaries
  - Create GitHub releases
  - Publish to crates.io (if configured)

#### ⏰ Scheduled (`scheduled.yml`)
- **Triggers**: Daily at 2 AM UTC, Manual trigger
- **Checks**:
  - Nightly Rust compatibility
  - Dependency health monitoring
  - Performance baseline tracking

### Dependencies

#### Automated Updates
Dependencies are automatically updated via [Dependabot](https://github.com/dependabot) with weekly checks for:
- Security updates
- Minor version updates
- Patch updates

#### Required Secrets
For publishing releases, configure these GitHub secrets:
- `CRATES_IO_TOKEN`: For publishing to crates.io

### Local Development

```bash
# Run all tests locally
cargo test

# Run with coverage (requires cargo-tarpaulin)
cargo install cargo-tarpaulin
cargo tarpaulin

# Run clippy with pedantic checks
cargo clippy --all-targets --all-features -- -W clippy::pedantic

# Check formatting
cargo fmt --all -- --check

# Security audit
cargo install cargo-audit
cargo audit
```

### Branch Protection

It's recommended to configure branch protection rules for `main`:
- Require status checks to pass
- Require up-to-date branches
- Include administrators in restrictions

### Badges

Add these badges to your README:

```markdown
[![CI](https://github.com/your-username/proxy-reencryption-enclave-app/actions/workflows/ci.yml/badge.svg)](https://github.com/your-username/proxy-reencryption-enclave-app/actions/workflows/ci.yml)
[![Code Quality](https://github.com/your-username/proxy-reencryption-enclave-app/actions/workflows/code-quality.yml/badge.svg)](https://github.com/your-username/proxy-reencryption-enclave-app/actions/workflows/code-quality.yml)
[![Security Audit](https://github.com/your-username/proxy-reencryption-enclave-app/actions/workflows/ci.yml/badge.svg?event=schedule)](https://github.com/your-username/proxy-reencryption-enclave-app/actions/workflows/scheduled.yml)
```