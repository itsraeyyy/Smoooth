use thiserror::Error;

#[derive(Error, Debug)]
pub enum SmooothError {
    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Policy violation: {0}")]
    PolicyViolation(String),

    #[error("Idempotency conflict: {0}")]
    IdempotencyConflict(String),

    #[error("Fencing lease conflict: {0}")]
    LeaseConflict(String),

    #[error("Provider fencing violation: {0}")]
    FencingViolation(String),

    #[error("Cryptographic security error: {0}")]
    CryptoError(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Adapter error: {0}")]
    Adapter(String),

    #[error("Docker daemon unavailable: {0}")]
    DockerUnavailable(String),

    #[error("Container failure: {0}")]
    ContainerFailed(String),

    #[error("PostgreSQL failure: {0}")]
    PostgresFailed(String),

    #[error("Operation timeout: {0}")]
    Timeout(String),

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, SmooothError>;
