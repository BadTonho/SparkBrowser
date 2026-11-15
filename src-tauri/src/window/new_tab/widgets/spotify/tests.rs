use super::*;
use std::{
    io::{Read, Write},
    net::TcpListener,
    thread,
};

fn server(
    count: usize,
    handler: impl Fn(String, String) -> String + Send + 'static,
) -> (String, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let origin = format!("http://{}", listener.local_addr().unwrap());
    let task = thread::spawn(move || {
        for _ in 0..count {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(std::time::Duration::from_secs(5)))
                .unwrap();
            let mut bytes = Vec::new();
            let mut buffer = [0; 4096];
            let (end, length) = loop {
                let n = stream.read(&mut buffer).unwrap();
                assert!(n > 0);
                bytes.extend_from_slice(&buffer[..n]);
                if let Some(end) = bytes.windows(4).position(|b| b == b"\r\n\r\n") {
                    let header = String::from_utf8_lossy(&bytes[..end]).to_lowercase();
                    let length = header
                        .lines()
                        .find_map(|l| l.strip_prefix("content-length: "))
                        .and_then(|s| s.parse::<usize>().ok())
                        .unwrap_or(0);
                    break (end + 4, length);
                }
            };
            while bytes.len() < end + length {
                let n = stream.read(&mut buffer).unwrap();
                assert!(n > 0);
                bytes.extend_from_slice(&buffer[..n]);
            }
            let header = String::from_utf8(bytes[..end].to_vec()).unwrap();
            let body = String::from_utf8(bytes[end..end + length].to_vec()).unwrap();
            stream.write_all(handler(header, body).as_bytes()).unwrap();
        }
    });
    (origin, task)
}
fn json(value: serde_json::Value) -> String {
    let body = value.to_string();
    format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len())
}

#[test]
fn oauth_pkce_and_refresh_use_only_the_required_scopes_and_no_secret() {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    use sha2::{Digest, Sha256};
    let expected = Arc::new(Mutex::new(String::new()));
    let challenge = expected.clone();
    let (origin, task) = server(3, move |header, body| {
        if header.starts_with("POST /authorize ") {
            let url = url::Url::parse(&body).unwrap();
            let values: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();
            assert_eq!(values["scope"], SCOPE);
            assert_eq!(values["code_challenge_method"], "S256");
            *challenge.lock().unwrap() = values["code_challenge"].clone();
            let mut callback = url::Url::parse(&values["redirect_uri"]).unwrap();
            callback
                .query_pairs_mut()
                .append_pair("code", "artificial-code")
                .append_pair("state", &values["state"]);
            thread::spawn(move || {
                super::super::http::client()
                    .unwrap()
                    .get(callback)
                    .send()
                    .unwrap()
                    .error_for_status()
                    .unwrap();
            });
            return json(serde_json::json!({}));
        }
        let form: std::collections::HashMap<_, _> = url::form_urlencoded::parse(body.as_bytes())
            .into_owned()
            .collect();
        assert!(!form.contains_key("client_secret"));
        if form["grant_type"] == "authorization_code" {
            assert_eq!(form["code"], "artificial-code");
            assert_eq!(
                URL_SAFE_NO_PAD.encode(Sha256::digest(form["code_verifier"].as_bytes())),
                *challenge.lock().unwrap()
            );
        } else {
            assert_eq!(form["refresh_token"], "artificial-refresh");
        }
        json(
            serde_json::json!({"access_token":"artificial-access","refresh_token":"artificial-refresh","expires_in":3600,"scope":SCOPE}),
        )
    });
    let mut endpoints = endpoints(&None);
    endpoints.token = format!("{origin}/token");
    endpoints.fixture = Some(origin);
    let config = oauth::Config {
        client_id: "a".repeat(32),
        client_secret: String::new(),
    };
    let grant = oauth::connect_scoped(
        &config,
        &endpoints,
        Arc::new(AtomicBool::new(false)),
        SCOPE,
        |_| panic!("real browser must never open"),
    )
    .unwrap();
    assert_eq!(grant.refresh.as_deref(), Some("artificial-refresh"));
    let refreshed = oauth::grant(
        &endpoints.token,
        SCOPE,
        &[
            ("client_id", &config.client_id),
            ("grant_type", "refresh_token"),
            ("refresh_token", "artificial-refresh"),
        ],
    )
    .unwrap();
    assert_eq!(refreshed.access.token, "artificial-access");
    task.join().unwrap();
}
#[test]
fn playback_commands_empty_responses_and_http_errors() {
    let (origin, task) = server(2, |header, _| {
        assert!(header
            .to_lowercase()
            .contains("authorization: bearer artificial-access"));
        assert!(
            header.starts_with("PUT /me/player/pause ") || header.starts_with("GET /me/player?")
        );
        "HTTP/1.1 204 No Content\r\nConnection: close\r\n\r\n".into()
    });
    api::control(&origin, "artificial-access", "spotify_pause").unwrap();
    assert!(api::playback(&origin, "artificial-access", None)
        .unwrap()
        .is_none());
    task.join().unwrap();
    for (status, code) in [
        (401, "reconnect"),
        (403, "spotify_access_denied"),
        (404, "spotify_no_device"),
        (429, "rate_limited"),
        (503, "network_error"),
    ] {
        let (origin, task) = server(1, move |_, _| {
            format!("HTTP/1.1 {status} Error\r\nRetry-After: 120\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
        });
        let failure = api::playback(&origin, "artificial-access", None)
            .err()
            .unwrap();
        assert_eq!(failure.code, code);
        assert_eq!(failure.retry, 120);
        task.join().unwrap();
    }
}
#[test]
fn refresh_rate_limit_and_revoked_authorization() {
    let (origin, task) = server(1, |_, _| {
        "HTTP/1.1 429 Error\r\nRetry-After: 300\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            .into()
    });
    let failure = api::Failure::from(oauth::grant(&origin, SCOPE, &[]).err().unwrap());
    assert_eq!(failure.code, "rate_limited");
    assert_eq!(failure.retry, 300);
    task.join().unwrap();
    let (origin, task) = server(1, |_, _| {
        let body = r#"{"error":"invalid_grant"}"#;
        format!(
            "HTTP/1.1 400 Bad Request\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
    });
    assert_eq!(
        oauth::grant(&origin, SCOPE, &[]).err().unwrap(),
        "reconnect"
    );
    task.join().unwrap();
}
#[test]
fn cancel_invalidates_pending_work_without_dropping_the_saved_connection() {
    let flag = Arc::new(AtomicBool::new(false));
    let mut runtime = Runtime {
        credential: Some(Credential {
            client_id: "a".repeat(32),
            refresh: "artificial-refresh".into(),
        }),
        connection: Some(("owner".into(), flag.clone())),
        view: View {
            connected: true,
            connecting: true,
            busy: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let old_epoch = runtime.epoch;
    cancel(&mut runtime);
    assert!(flag.load(Ordering::SeqCst));
    assert_ne!(runtime.epoch, old_epoch);
    assert!(!runtime.view.busy && !runtime.view.connecting);
    assert!(runtime.credential.is_some());
    let display = serde_json::to_string(&runtime.view).unwrap();
    assert!(!display.contains("artificial-refresh"));
}
