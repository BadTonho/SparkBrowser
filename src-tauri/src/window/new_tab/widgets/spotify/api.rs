use super::super::http;
use base64::{engine::general_purpose::STANDARD, Engine};
use serde::Serialize;
use serde_json::Value;
use std::io::Read;

#[derive(Clone, Debug)]
pub struct Failure {
    pub code: String,
    pub retry: u64,
}
impl From<String> for Failure {
    fn from(code: String) -> Self {
        if let Some(retry) = code
            .strip_prefix("spotify_rate_limited:")
            .and_then(|v| v.parse::<u64>().ok())
        {
            return Self {
                code: "rate_limited".into(),
                retry,
            };
        }
        Self { code, retry: 30 }
    }
}
#[derive(Clone, Default, Serialize)]
pub struct Playback {
    pub title: String,
    pub artist: String,
    pub device: String,
    pub image: Option<String>,
    pub playing: bool,
    pub progress: u64,
    pub duration: u64,
    pub can_play: bool,
    pub can_pause: bool,
    pub can_next: bool,
    pub can_previous: bool,
}

pub fn checked(
    response: reqwest::blocking::Response,
) -> Result<reqwest::blocking::Response, Failure> {
    if response.status().is_success() {
        return Ok(response);
    }
    let retry = response
        .headers()
        .get("retry-after")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.parse::<u64>().ok())
        .unwrap_or(30)
        .max(1);
    let code = match response.status().as_u16() {
        401 => "reconnect",
        403 => "spotify_access_denied",
        404 => "spotify_no_device",
        429 => "rate_limited",
        _ => "network_error",
    };
    Err(Failure {
        code: code.into(),
        retry,
    })
}
pub fn playback(
    endpoint: &str,
    token: &str,
    previous: Option<&Playback>,
) -> Result<Option<Playback>, Failure> {
    let response = checked(http::send(
        http::client()?
            .get(format!("{endpoint}/me/player"))
            .query(&[("additional_types", "track,episode")])
            .bearer_auth(token),
    )?)?;
    if response.status().as_u16() == 204 {
        return Ok(None);
    }
    let value: Value = http::json(response)?;
    let mut data = parse(&value)?;
    // Reuse artwork for unchanged content, including an earlier failed image fetch.
    data.image = if previous.is_some_and(|p| p.title == data.title && p.artist == data.artist) {
        previous.and_then(|p| p.image.clone())
    } else {
        let item = &value["item"];
        let images = if item["type"] == "episode" {
            &item["images"]
        } else {
            &item["album"]["images"]
        };
        images
            .as_array()
            .and_then(|a| a.last())
            .and_then(|i| i["url"].as_str())
            .and_then(artwork)
    };
    Ok(Some(data))
}
fn label(value: &Value) -> String {
    value
        .as_str()
        .unwrap_or_default()
        .chars()
        .take(500)
        .collect()
}
pub fn parse(value: &Value) -> Result<Playback, String> {
    if !value.is_object() || !value["device"].is_object() {
        return Err("invalid_response".into());
    }
    let item = &value["item"];
    let artist = if item["type"] == "episode" {
        label(&item["show"]["name"])
    } else {
        item["artists"]
            .as_array()
            .map(|items| {
                items
                    .iter()
                    .take(20)
                    .map(|v| label(&v["name"]))
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default()
    };
    let restricted = value["device"]["is_restricted"] == true;
    let allowed = |name: &str| !restricted && value["actions"]["disallows"][name] != true;
    Ok(Playback {
        title: label(&item["name"]),
        artist,
        device: label(&value["device"]["name"]),
        playing: value["is_playing"] == true,
        progress: value["progress_ms"].as_u64().unwrap_or(0),
        duration: item["duration_ms"].as_u64().unwrap_or(0),
        can_play: allowed("resuming"),
        can_pause: allowed("pausing"),
        can_next: allowed("skipping_next"),
        can_previous: allowed("skipping_prev"),
        ..Default::default()
    })
}
pub fn control(endpoint: &str, token: &str, action: &str) -> Result<(), Failure> {
    let client = http::client()?;
    let request = match action {
        "spotify_play" => client.put(format!("{endpoint}/me/player/play")),
        "spotify_pause" => client.put(format!("{endpoint}/me/player/pause")),
        "spotify_next" => client.post(format!("{endpoint}/me/player/next")),
        "spotify_previous" => client.post(format!("{endpoint}/me/player/previous")),
        _ => return Err("invalid_action".to_owned().into()),
    };
    checked(http::send(
        request.bearer_auth(token).header("Content-Length", "0"),
    )?)?;
    Ok(())
}
fn artwork(raw: &str) -> Option<String> {
    let url = url::Url::parse(raw).ok()?;
    if url.scheme() != "https"
        || url.host_str() != Some("i.scdn.co")
        || !url.username().is_empty()
        || url.password().is_some()
        || url.port().is_some()
        || !url.path().starts_with("/image/")
    {
        return None;
    }
    let response = http::client()
        .ok()?
        .get(url)
        .send()
        .ok()?
        .error_for_status()
        .ok()?;
    let mime = response
        .headers()
        .get("content-type")?
        .to_str()
        .ok()?
        .split(';')
        .next()?;
    if !matches!(mime, "image/jpeg" | "image/png" | "image/webp") {
        return None;
    }
    let mime = mime.to_owned();
    let mut bytes = Vec::new();
    response.take(262_145).read_to_end(&mut bytes).ok()?;
    let valid = (mime == "image/jpeg" && bytes.starts_with(&[0xff, 0xd8, 0xff]))
        || (mime == "image/png" && bytes.starts_with(b"\x89PNG\r\n\x1a\n"))
        || (mime == "image/webp"
            && bytes.starts_with(b"RIFF")
            && bytes.get(8..12) == Some(b"WEBP"));
    if bytes.len() > 262_144 || !valid {
        return None;
    }
    Some(format!("data:{mime};base64,{}", STANDARD.encode(bytes)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn playback_preserves_text_and_respects_device_and_action_restrictions() {
        let mut v = serde_json::json!({"device":{"name":"Phone","is_restricted":false},"is_playing":true,"item":{"name":"<script>","type":"track","artists":[{"name":"Artist"}],"duration_ms":200000},"actions":{"disallows":{"skipping_next":true}}});
        let p = parse(&v).unwrap();
        assert_eq!(p.title, "<script>");
        assert_eq!(p.artist, "Artist");
        assert!(p.can_pause);
        assert!(!p.can_next);
        v["device"]["is_restricted"] = true.into();
        assert!(!parse(&v).unwrap().can_play);
        v["item"] = serde_json::json!({"type":"episode","name":"Episode","show":{"name":"Show"}});
        assert_eq!(parse(&v).unwrap().artist, "Show");
        v["item"] = Value::Null;
        assert_eq!(parse(&v).unwrap().title, "");
        assert!(parse(&Value::Null).is_err());
    }
    #[test]
    fn artwork_rejects_untrusted_urls_without_network() {
        for url in [
            "http://i.scdn.co/image/a",
            "https://evil.test/image/a",
            "https://i.scdn.co@evil.test/image/a",
            "https://i.scdn.co:9000/image/a",
            "https://i.scdn.co/other",
        ] {
            assert!(artwork(url).is_none());
        }
    }
}
