# Dockerfile for AWS Nitro Enclave
FROM amazonlinux:2

# Install required packages
RUN yum update -y && \
    yum install -y \
    aws-nitro-enclaves-cli \
    awscli \
    && yum clean all

# Create app directory
WORKDIR /app

# Copy the compiled binary (this will be mounted by nitro-cli)
COPY proxy_reencyption_enclave_app /app/

# Ensure the binary is executable
RUN chmod +x /app/proxy_reencyption_enclave_app

# Create non-root user
RUN useradd -r -s /bin/false appuser && \
    chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

# Expose port for Vsock communication
EXPOSE 5005

# Health check (optional - enclave health checks work differently)
# HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
#     CMD /app/proxy_reencyption_enclave_app --version || exit 1

# Default command to run the enclave server
CMD ["/app/proxy_reencyption_enclave_app", "server", "--port", "5005"]
