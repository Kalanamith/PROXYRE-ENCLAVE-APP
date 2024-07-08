# proxy-reencyption enclave app

This app uses AWS Nitro Enclaves to create an environment where a client can securely request the creation of cryptographic keys inside an enclave.

It enables content reencryption using Rust's recrypt library.

## Run server
```bash
cargo run -- server --port 5005
```

## Run client

```bash
cargo run -- client --cid 3 --port 5005
```

# Deployment

```bashls
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
