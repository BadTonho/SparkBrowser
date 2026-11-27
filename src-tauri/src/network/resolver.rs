//! DNS over HTTPS lookups.
//!
//! The DoH server is reached by a fixed IP address (the name in the URL is only used for the TLS
//! certificate check), so asking for the provider never uses the system DNS. A failed lookup is an
//! error: there is no fallback to the system resolver.
use super::Provider;
use crate::sync::Mutex;
use std::{
    collections::HashMap,
    future::Future,
    net::{IpAddr, SocketAddr},
    pin::Pin,
    time::{Duration, Instant},
};

const TIMEOUT: Duration = Duration::from_secs(5);
const MIN_TTL: u64 = 30;
const MAX_TTL: u64 = 300;
const CACHE_LIMIT: usize = 1024;

/// Turns a host name into addresses.
pub trait Resolve: Send + Sync + 'static {
    fn resolve(
        &self,
        host: String,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<IpAddr>, String>> + Send + '_>>;
}

pub struct DohResolver {
    client: reqwest::Client,
    endpoint: String,
    cache: Mutex<HashMap<String, (Vec<IpAddr>, Instant)>>,
}

impl DohResolver {
    pub fn new(provider: Provider) -> Result<Self, String> {
        let addresses: Vec<SocketAddr> = provider
            .bootstrap()
            .iter()
            .map(|ip| SocketAddr::new(*ip, 443))
            .collect();
        let client = reqwest::Client::builder()
            .resolve_to_addrs(provider.host(), &addresses)
            .timeout(TIMEOUT)
            .redirect(reqwest::redirect::Policy::none())
            .no_proxy()
            .user_agent("Mozilla/5.0")
            .build()
            .map_err(|e| e.to_string())?;
        Ok(Self {
            client,
            endpoint: provider.endpoint().to_string(),
            cache: Mutex::new(HashMap::new()),
        })
    }

    async fn query(&self, host: &str, kind: &str) -> Result<(Vec<IpAddr>, u64), String> {
        let response = self
            .client
            .get(&self.endpoint)
            .query(&[("name", host), ("type", kind)])
            .header("accept", "application/dns-json")
            .send()
            .await
            .map_err(|e| format!("DNS over HTTPS request failed: {e}"))?;
        if !response.status().is_success() {
            return Err(format!(
                "DNS over HTTPS server answered {}",
                response.status()
            ));
        }
        let body = response
            .text()
            .await
            .map_err(|e| format!("DNS over HTTPS answer could not be read: {e}"))?;
        parse_answer(&body)
    }
}

/// `Status` 0 is success; 3 means the name does not exist. Only A and AAAA answers are used (a CNAME
/// chain is already followed by the server).
fn parse_answer(body: &str) -> Result<(Vec<IpAddr>, u64), String> {
    let value: serde_json::Value =
        serde_json::from_str(body).map_err(|_| "DNS over HTTPS answer is not valid".to_string())?;
    match value.get("Status").and_then(|s| s.as_u64()) {
        Some(0) => {}
        Some(3) => return Err("The name does not exist".to_string()),
        Some(code) => return Err(format!("DNS over HTTPS answered with status {code}")),
        None => return Err("DNS over HTTPS answer is not valid".to_string()),
    }
    let mut addresses = Vec::new();
    let mut ttl = MAX_TTL;
    for record in value
        .get("Answer")
        .and_then(|a| a.as_array())
        .into_iter()
        .flatten()
    {
        let kind = record.get("type").and_then(|t| t.as_u64());
        if !matches!(kind, Some(1) | Some(28)) {
            continue;
        }
        if let Some(ip) = record
            .get("data")
            .and_then(|d| d.as_str())
            .and_then(|d| d.parse::<IpAddr>().ok())
        {
            addresses.push(ip);
            if let Some(record_ttl) = record.get("TTL").and_then(|t| t.as_u64()) {
                ttl = ttl.min(record_ttl);
            }
        }
    }
    Ok((addresses, ttl.clamp(MIN_TTL, MAX_TTL)))
}

impl Resolve for DohResolver {
    fn resolve(
        &self,
        host: String,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<IpAddr>, String>> + Send + '_>> {
        Box::pin(async move {
            let host = host.trim_end_matches('.').to_ascii_lowercase();
            if host.is_empty() {
                return Err("Empty host name".to_string());
            }
            if let Ok(cache) = self.cache.lock() {
                if let Some((addresses, until)) = cache.get(&host) {
                    if *until > Instant::now() {
                        return Ok(addresses.clone());
                    }
                }
            }
            let (v4, v6) = tokio::join!(self.query(&host, "A"), self.query(&host, "AAAA"));
            let mut addresses = Vec::new();
            let mut ttl = MAX_TTL;
            let mut failure = None;
            for answer in [v4, v6] {
                match answer {
                    Ok((found, answer_ttl)) => {
                        addresses.extend(found);
                        ttl = ttl.min(answer_ttl);
                    }
                    Err(error) => failure = failure.or(Some(error)),
                }
            }
            if addresses.is_empty() {
                return Err(failure.unwrap_or_else(|| "The name has no address".to_string()));
            }
            if let Ok(mut cache) = self.cache.lock() {
                if cache.len() >= CACHE_LIMIT {
                    cache.clear();
                }
                cache.insert(
                    host,
                    (addresses.clone(), Instant::now() + Duration::from_secs(ttl)),
                );
            }
            Ok(addresses)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn answers_keep_only_addresses_and_the_shortest_ttl() {
        let body = r#"{"Status":0,"Answer":[
            {"name":"a.test","type":5,"TTL":10,"data":"b.test."},
            {"name":"b.test","type":1,"TTL":120,"data":"192.0.2.7"},
            {"name":"b.test","type":28,"TTL":60,"data":"2001:db8::7"},
            {"name":"b.test","type":1,"TTL":90,"data":"not an address"}]}"#;
        let (addresses, ttl) = parse_answer(body).unwrap();
        assert_eq!(
            addresses,
            vec![
                "192.0.2.7".parse::<IpAddr>().unwrap(),
                "2001:db8::7".parse::<IpAddr>().unwrap()
            ]
        );
        assert_eq!(ttl, 60);
    }

    #[test]
    fn the_ttl_is_kept_between_the_limits() {
        let low = r#"{"Status":0,"Answer":[{"type":1,"TTL":1,"data":"192.0.2.1"}]}"#;
        assert_eq!(parse_answer(low).unwrap().1, MIN_TTL);
        let none = r#"{"Status":0}"#;
        assert_eq!(parse_answer(none).unwrap(), (vec![], MAX_TTL));
    }

    #[test]
    fn failures_are_errors_never_empty_successes() {
        assert!(parse_answer(r#"{"Status":3}"#)
            .unwrap_err()
            .contains("does not exist"));
        assert!(parse_answer(r#"{"Status":2}"#)
            .unwrap_err()
            .contains("status 2"));
        assert!(parse_answer("<html>").is_err());
        assert!(parse_answer(r#"{"Answer":[]}"#).is_err());
    }
}
