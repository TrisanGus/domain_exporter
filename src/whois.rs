use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use whois_rust::{WhoIs, WhoIsLookupOptions};
use std::path::Path;

pub struct DomainInfo {
    pub expiry_date: DateTime<Utc>,
    pub raw_text: String,
}

pub async fn query_domain(domain: &str) -> Result<DomainInfo> {
    // Create WHOIS client using servers.json file
    let whois = WhoIs::from_path(Path::new("servers.json"))?;
    
    // Query domain
    let raw_text = whois.lookup(WhoIsLookupOptions::from_string(domain)?)?;
    
    // Parse expiry date
    let expiry_date = parse_expiry_date(&raw_text)
        .ok_or_else(|| anyhow!("无法解析过期时间"))?;

    Ok(DomainInfo {
        expiry_date,
        raw_text,
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
                    if let Ok(dt) = DateTime::parse_from_rfc3339(date_str) {
                        return Some(dt.with_timezone(&Utc));
                    }
                    
                    // Add more date format parsing...
                }
            }
        }
    }
    
    None
}
