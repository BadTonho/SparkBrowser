#[cfg(any(windows, test))]
use super::releases::{Release, REPOSITORY};
#[cfg(any(windows, test))]
use std::time::Duration;

#[derive(Debug)]
pub struct CheckError {
    pub message: String,
    pub retry_after: Option<i64>,
}
impl CheckError {
    pub fn plain(message: impl ToString) -> Self {
        Self {
            message: message.to_string(),
            retry_after: None,
        }
    }
}
pub fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}
#[cfg(any(windows, test))]
pub struct GitHub {
    client: reqwest::Client,
    releases_url: String,
}
#[cfg(any(windows, test))]
impl GitHub {
    #[cfg_attr(not(windows), allow(dead_code))]
    pub fn new(version: &str) -> Result<Self, CheckError> {
        Ok(Self {
            client: reqwest::Client::builder()
                .https_only(true)
                .connect_timeout(Duration::from_secs(10))
                .timeout(Duration::from_secs(30))
                .user_agent(format!("VeloceBrowser/{version}"))
                .build()
                .map_err(CheckError::plain)?,
            releases_url: format!("https://api.github.com/repos/{REPOSITORY}/releases"),
        })
    }
    pub async fn json(&self, url: &str) -> Result<serde_json::Value, CheckError> {
        let mut response = self
            .client
            .get(url)
            .header("Accept", "application/vnd.github+json")
            .send()
            .await
            .map_err(|_| {
                CheckError::plain(
                    "Could not connect to GitHub. Check your connection and try again.",
                )
            })?;
        let status = response.status();
        if status.as_u16() == 429
            || (status.as_u16() == 403
                && (response.headers().contains_key("retry-after")
                    || response
                        .headers()
                        .get("x-ratelimit-remaining")
                        .is_some_and(|v| v == "0")))
        {
            let deadline = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<i64>().ok())
                .map(|s| now().saturating_add(s.max(1)))
                .or_else(|| {
                    response
                        .headers()
                        .get("x-ratelimit-reset")
                        .and_then(|v| v.to_str().ok())
                        .and_then(|v| v.parse().ok())
                })
                .unwrap_or_else(|| now() + 60);
            return Err(CheckError {
                message: "GitHub rate limit reached. Try again after the indicated time.".into(),
                retry_after: Some(deadline.max(now() + 1)),
            });
        }
        if !status.is_success() {
            return Err(CheckError::plain(format!(
                "GitHub returned HTTP {}. Check that the release is public and complete.",
                status.as_u16()
            )));
        }
        const LIMIT: usize = 4 * 1024 * 1024;
        let mut bytes = Vec::new();
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|_| CheckError::plain("GitHub response was interrupted. Try again."))?
        {
            if bytes.len().saturating_add(chunk.len()) > LIMIT {
                return Err(CheckError::plain(
                    "GitHub metadata exceeds the supported size",
                ));
            }
            bytes.extend_from_slice(&chunk);
        }
        serde_json::from_slice(&bytes)
            .map_err(|_| CheckError::plain("GitHub returned invalid release metadata"))
    }
    pub async fn releases(&self) -> Result<Vec<Release>, CheckError> {
        let mut releases = Vec::new();
        for page in 1..=20 {
            let data = self
                .json(&format!("{}?per_page=100&page={page}", self.releases_url))
                .await?;
            let items: Vec<Release> = serde_json::from_value(data)
                .map_err(|_| CheckError::plain("Invalid GitHub releases response"))?;
            let complete = items.len() < 100;
            releases.extend(items);
            if complete {
                return Ok(releases);
            }
        }
        Err(CheckError::plain(
            "Release history exceeds the lookup limit; could not determine the newest version",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::{Read, Write},
        net::TcpListener,
    };
    #[tokio::test]
    async fn pagination_and_rate_limit_are_not_reported_as_up_to_date() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            for page in 1..=3 {
                let (mut stream, _) = listener.accept().unwrap();
                let mut buffer = [0; 4096];
                let count = stream.read(&mut buffer).unwrap();
                let request = String::from_utf8_lossy(&buffer[..count]);
                let (status, extra, body) = if page == 3 {
                    (
                        "429 Too Many Requests",
                        "Retry-After: 60\r\n",
                        "{}".to_owned(),
                    )
                } else {
                    assert!(request.contains(&format!("page={page}")));
                    let row = serde_json::json!({"tag_name":"v1.0.0","draft":false,"prerelease":false,"published_at":"2026-01-01T00:00:00Z","body":null,"assets":[]});
                    (
                        "200 OK",
                        "",
                        serde_json::to_string(&vec![row; if page == 1 { 100 } else { 1 }]).unwrap(),
                    )
                };
                write!(stream, "HTTP/1.1 {status}\r\n{extra}Content-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
            }
        });
        let api = GitHub {
            client: reqwest::Client::new(),
            releases_url: format!("http://{address}/releases"),
        };
        assert_eq!(api.releases().await.unwrap().len(), 101);
        let error = api.json(&api.releases_url).await.unwrap_err();
        assert!(error.retry_after.unwrap() >= now() + 59);
        server.join().unwrap();
        let offline = api.json(&api.releases_url).await.unwrap_err();
        assert!(offline.message.contains("connect"));
    }
}
