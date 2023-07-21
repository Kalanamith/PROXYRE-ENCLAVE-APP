FROM  rustlang/rust:nightly AS builder

WORKDIR /proxy-reencyption
COPY . /proxy-reencyption

ARG RUST_VERSION=1.64.0-nightly
ARG RUST_NIGHTLY=nightly-2022-05-15
RUN apt-get update && \
    apt-get -y install apt-utils cmake pkg-config libssl-dev git clang libclang-dev && \
    rustup uninstall nightly && \
    rustup install $RUST_NIGHTLY && \
    mv /usr/local/rustup/toolchains/nightly* /usr/local/rustup/toolchains/nightly-x86_64-unknown-linux-gnu && \
    mkdir -p /proxy-reencyption/.cargo

ENV CARGO_HOME=/proxy-reencyption/.cargo

RUN cargo build --release
EXPOSE 8000 5005
ENTRYPOINT ["./target/release/proxy-reencyption-enclave-app", "client", "--cid", "3", "--port", "5005"]
##CMD ["./target/release/proxy-reencyption-enclave-app client --cid 3 --port 5005"]
#FROM debian:buster-slim
#LABEL maintainer="dev@proxy-reencyption.io"
#
##RUN apt-get update && \
##    apt-get install -y ca-certificates openssl curl && \
##    mkdir -p /root/.local/share/proxy-reencyption && \
##    ln -s /root/.local/share/proxy-reencyption /data
#
#COPY --from=builder /target/release/proxy-reencyption-enclave-app .
#
#
#
#CMD ["proxy-reencyption-enclave-app -- client --cid 3 --port 5005"]