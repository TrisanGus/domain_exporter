use crate::error::{DomainError, Result};
use chrono::{DateTime, Utc, TimeZone, NaiveDate, NaiveDateTime};
use whois_rust::{WhoIs, WhoIsLookupOptions};
use std::time::Duration;
use tokio::time::{timeout, sleep};
use tracing::{info, warn};
use rust_embed::RustEmbed;

const WHOIS_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_RETRIES: u32 = 3;
const RETRY_DELAY: Duration = Duration::from_secs(2);

#[derive(RustEmbed)]
#[folder = "assets/"]
struct Asset;

pub struct DomainInfo {
    pub expiry_date: DateTime<Utc>,
}

pub async fn query_domain(domain: &str) -> Result<DomainInfo> {
    let mut retries = 0;
    
    loop {
        match query_domain_internal(domain).await {
            Ok(info) => return Ok(info),
            Err(e) => {
                match e {
                    DomainError::TimeoutError | DomainError::ServerBusyError => {
                        retries += 1;
                        if retries >= MAX_RETRIES {
                            warn!("Max retries ({}) reached for domain: {}", MAX_RETRIES, domain);
                            return Err(e);
                        }
                        warn!("Retry {}/{} for domain: {}", retries, MAX_RETRIES, domain);
                        sleep(RETRY_DELAY).await;
                        continue;
                    }
                    _ => return Err(e),
                }
            }
        }
    }
}

async fn query_domain_internal(domain: &str) -> Result<DomainInfo> {
    info!("Querying domain: {}", domain);

    // Read servers.json from embedded resources
    let servers_json = Asset::get("servers.json")
        .ok_or_else(|| DomainError::Other("Could not find servers.json".to_string()))?;
    
    // Convert bytes to string
    let servers_str = std::str::from_utf8(&servers_json.data)
        .map_err(|e| DomainError::Other(e.to_string()))?;
    // Create WHOIS client using string
    let whois = WhoIs::from_string(servers_str)?;
    
    // Clone domain string for the closure
    let domain = domain.to_string();
    
    // Query domain with timeout
    let lookup_result = timeout(
        WHOIS_TIMEOUT,
        tokio::task::spawn_blocking(move || {
            whois.lookup(WhoIsLookupOptions::from_string(&domain)?)
        })
    ).await;

    // Handle timeout and lookup errors
    let raw_text = match lookup_result {
        Ok(result) => {
            let text = result.map_err(|e| DomainError::Other(e.to_string()))?.map_err(|e| e)?;
            
            // Check for server busy response
            if text.contains("Server is busy") {
                warn!("WHOIS server is busy");
                return Err(DomainError::ServerBusyError);
            }
            
            text
        },
        Err(_) => {
            warn!("Domain query timed out after {:?}", WHOIS_TIMEOUT);
            return Err(DomainError::TimeoutError);
        }
    };

    // Parse expiry date
    let expiry_date = parse_expiry_date(&raw_text)
        .ok_or(DomainError::ExpiryDateParseError)?;

    Ok(DomainInfo {
        expiry_date,
    })
}

fn parse_expiry_date(whois_text: &str) -> Option<DateTime<Utc>> {
    // Common expiry date fields
    let expiry_patterns = [
        "Expiry Date:",
        "Registry Expiry Date:",
        "Expiration Date:",
        "Registrar Registration Expiration Date:",
    ];

    for line in whois_text.lines() {
        for pattern in expiry_patterns.iter() {
            if line.contains(pattern) {
                if let Some(date_str) = line.split(pattern).nth(1) {
                    // Clean date string
                    let date_str = date_str.trim();
                    
                    // Try parsing different date formats
                    let parsed_date = try_parse_date(date_str);
                    if parsed_date.is_some() {
                        return parsed_date;
                    }
                }
            }
        }
    }
    
    None
}

fn try_parse_date(date_str: &str) -> Option<DateTime<Utc>> {
    // Clean up the date string
    let date_str = date_str.trim()
        .split(" (")
        .next()
        .unwrap_or(date_str)
        .trim();

    // Special handling for dates with .0Z format
    if date_str.ends_with(".0Z") {
        let without_ms = date_str.replace(".0Z", "Z");
        if let Ok(dt) = DateTime::parse_from_rfc3339(&without_ms) {
            return Some(dt.with_timezone(&Utc));
        }
    }

    // Common date formats
    let formats = [
        "%Y-%m-%dT%H:%M:%SZ",           // 2024-03-21T15:30:00Z
        "%Y-%m-%dT%H:%M:%S%:z",         // 2024-03-21T15:30:00+00:00
        "%Y-%m-%d %H:%M:%S %Z",         // 2024-03-21 15:30:00 UTC
        "%Y-%m-%d %H:%M:%S",            // 2024-03-21 15:30:00
        "%d-%b-%Y %H:%M:%S %Z",         // 21-Mar-2024 15:30:00 UTC
        "%d-%b-%Y",                      // 21-Mar-2024
        "%Y-%m-%d",                      // 2024-03-21
        "%Y.%m.%d",                      // 2024.03.21
        "%d.%m.%Y",                      // 21.03.2024
        "%Y/%m/%d",                      // 2024/03/21
    ];

    // Try parsing with standard formats
    for format in formats.iter() {
        if let Ok(dt) = DateTime::parse_from_str(date_str, format) {
            return Some(dt.with_timezone(&Utc));
        }
        if let Ok(dt) = NaiveDateTime::parse_from_str(date_str, format) {
            return Some(Utc.from_utc_datetime(&dt));
        }
    }

    // If that fails, try parsing as NaiveDate and set time to midnight UTC
    let date_formats = [
        "%Y-%m-%d",
        "%d-%b-%Y",
        "%Y.%m.%d",
        "%d.%m.%Y",
        "%Y/%m/%d",
    ];

    for format in date_formats.iter() {
        if let Ok(date) = NaiveDate::parse_from_str(date_str, format) {
            return Some(Utc.from_utc_datetime(&date.and_hms_opt(0, 0, 0).unwrap()));
        }
    }

    None
}
