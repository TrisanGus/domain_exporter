use thiserror::Error;

#[derive(Error, Debug)]
pub enum DomainError {
    #[error("WHOIS query failed: {0}")]
    WhoisError(#[from] whois_rust::WhoIsError),
    
    #[error("Could not parse expiry date")]
    ExpiryDateParseError,
    
    #[error("Domain query timeout")]
    TimeoutError,
    
    #[error("Server is busy")]
    ServerBusyError,
    
    #[error("Other error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, DomainError>;
