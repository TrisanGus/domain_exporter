use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use whois_rust::{WhoIs, WhoIsLookupOptions};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "assets/"]
struct Asset;

pub struct DomainInfo {
    pub expiry_date: DateTime<Utc>,
}

pub async fn query_domain(domain: &str) -> Result<DomainInfo> {
    // Read servers.json from embedded resources
    let servers_json = Asset::get("servers.json")
        .ok_or_else(|| anyhow!("Could not find servers.json"))?;
    
    // Convert bytes to string
    let servers_str = std::str::from_utf8(&servers_json.data)?;
    // Create WHOIS client using string
    let whois = WhoIs::from_string(servers_str)?;
    // Query domain
    let raw_text = whois.lookup(WhoIsLookupOptions::from_string(domain)?)?;
    // Parse expiry date
    let expiry_date = parse_expiry_date(&raw_text)
        .ok_or_else(|| anyhow!("Could not parse expiry date"))?;

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
