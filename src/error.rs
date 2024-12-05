use thiserror::Error;

/// Custom error types for domain operations
#[derive(Error, Debug)]
pub enum DomainError {
    /// Error occurred during WHOIS query
    #[error("WHOIS query failed: {0}")]
    WhoisError(#[from] whois_rust::WhoIsError),
    
    /// Could not parse the expiry date from WHOIS response
    #[error("Could not parse expiry date")]
    ExpiryDateParseError,
    
    /// WHOIS query timed out
    #[error("Domain query timeout")]
    TimeoutError,
    
    /// WHOIS server reported being busy
    #[error("Server is busy")]
    ServerBusyError,
    
    /// Other unexpected errors
    #[error("Other error: {0}")]
    Other(String),
}

/// Result type alias for domain operations
pub type Result<T> = std::result::Result<T, DomainError>;
