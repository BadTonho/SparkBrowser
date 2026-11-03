use serde::de::DeserializeOwned;
use std::{io::Read, time::Duration};
pub fn client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(12))
        .connect_timeout(Duration::from_secs(5))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| "network_error".into())
}
pub fn json<T: DeserializeOwned>(response: reqwest::blocking::Response) -> Result<T, String> {
    let status = response.status();
    if !status.is_success() {
        return Err(match status.as_u16() {
            401 => "reconnect",
            403 => "access_denied",
            429 => "rate_limited",
            _ => "network_error",
        }
        .into());
    }
    let mut bytes = Vec::new();
    response
        .take(262_145)
        .read_to_end(&mut bytes)
        .map_err(|_| "network_error")?;
    if bytes.len() > 262_144 {
        return Err("invalid_response".into());
    }
    serde_json::from_slice(&bytes).map_err(|_| "invalid_response".into())
}
pub fn send(
    request: reqwest::blocking::RequestBuilder,
) -> Result<reqwest::blocking::Response, String> {
    request.send().map_err(|_| "network_error".into())
}
