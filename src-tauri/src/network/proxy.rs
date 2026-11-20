//! A local HTTP proxy (loopback only) that resolves every host name through DNS over HTTPS.
//!
//! The web engines send page traffic through it. `CONNECT` tunnels carry HTTPS and WebSockets; plain
//! HTTP requests are forwarded one per connection. A name that cannot be resolved is an error answer:
//! the proxy never asks the system resolver.
use super::resolver::Resolve;
use std::{net::IpAddr, sync::Arc, time::Duration};
use tokio::{
    io::{copy_bidirectional, AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    time::timeout,
};

const MAX_HEAD: usize = 32 * 1024;
const HEAD_TIMEOUT: Duration = Duration::from_secs(15);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug)]
enum Failure {
    /// The request itself is wrong (400).
    Bad(&'static str),
    /// The name or the server could not be reached (502).
    Upstream(String),
}

#[derive(Debug, PartialEq, Eq)]
enum Target {
    Tunnel {
        host: String,
        port: u16,
    },
    Forward {
        host: String,
        port: u16,
        origin: String,
    },
}

#[derive(Debug, PartialEq, Eq)]
struct Request {
    method: String,
    version: String,
    target: Target,
    headers: Vec<(String, String)>,
}

pub async fn serve(listener: TcpListener, resolver: Arc<dyn Resolve>) {
    loop {
        match listener.accept().await {
            Ok((stream, peer)) => {
                if !peer.ip().is_loopback() {
                    continue;
                }
                let resolver = resolver.clone();
                tauri::async_runtime::spawn(async move {
                    handle(stream, resolver).await;
                });
            }
            Err(error) => {
                crate::error_log::report(
                    crate::error_log::Level::Warning,
                    "native",
                    "DNS over HTTPS proxy could not accept a connection",
                    &error.to_string(),
                );
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        }
    }
}

async fn handle(mut client: TcpStream, resolver: Arc<dyn Resolve>) {
    match route(&mut client, resolver.as_ref()).await {
        Ok(()) => {}
        Err(Failure::Bad(reason)) => answer(&mut client, 400, "Bad Request", reason).await,
        Err(Failure::Upstream(reason)) => {
            // The host name is not written: it is what the user browsed.
            crate::error_log::report(
                crate::error_log::Level::Warning,
                "native",
                "DNS over HTTPS could not reach a site",
                &reason,
            );
            answer(
                &mut client,
                502,
                "Bad Gateway",
                "DNS over HTTPS could not look up or reach this site. The browser did not fall back to the system DNS. You can turn DNS over HTTPS off in Settings > Privacy & Security.",
            )
            .await;
        }
    }
}

async fn answer(client: &mut TcpStream, code: u16, text: &str, body: &str) {
    let response = format!(
        "HTTP/1.1 {code} {text}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    crate::error_log::ignored(client.write_all(response.as_bytes()).await);
    crate::error_log::ignored(client.shutdown().await);
}

async fn route(client: &mut TcpStream, resolver: &dyn Resolve) -> Result<(), Failure> {
    let (head, extra) = timeout(HEAD_TIMEOUT, read_head(client))
        .await
        .map_err(|_| Failure::Bad("Request timed out"))??;
    let request = parse_head(&head)?;
    let (host, port, opening) = match &request.target {
        Target::Tunnel { host, port } => (host, *port, None),
        Target::Forward { host, port, .. } => (host, *port, Some(forward_head(&request))),
    };
    let mut upstream = connect(resolver, host, port).await?;
    let upstream_failed = |e: std::io::Error| Failure::Upstream(e.to_string());
    match opening {
        None => client
            .write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n")
            .await
            .map_err(upstream_failed)?,
        Some(head) => upstream.write_all(&head).await.map_err(upstream_failed)?,
    }
    if !extra.is_empty() {
        upstream.write_all(&extra).await.map_err(upstream_failed)?;
    }
    // The connection ends when either side closes; that is not a failure.
    crate::error_log::ignored(
        copy_bidirectional(client, &mut upstream)
            .await
            .map(|_| ())
            .or_else(|e| match e.kind() {
                std::io::ErrorKind::ConnectionReset
                | std::io::ErrorKind::ConnectionAborted
                | std::io::ErrorKind::BrokenPipe
                | std::io::ErrorKind::UnexpectedEof => Ok(()),
                _ => Err(e),
            }),
    );
    Ok(())
}

/// Reads until the blank line that ends the headers. Bytes after it (the start of a body) are returned.
async fn read_head(client: &mut TcpStream) -> Result<(Vec<u8>, Vec<u8>), Failure> {
    let mut buffer = Vec::with_capacity(1024);
    let mut chunk = [0u8; 2048];
    loop {
        if let Some(end) = buffer.windows(4).position(|w| w == b"\r\n\r\n") {
            let extra = buffer.split_off(end + 4);
            buffer.truncate(end);
            return Ok((buffer, extra));
        }
        if buffer.len() > MAX_HEAD {
            return Err(Failure::Bad("Request headers are too large"));
        }
        let read = client
            .read(&mut chunk)
            .await
            .map_err(|_| Failure::Bad("Request could not be read"))?;
        if read == 0 {
            return Err(Failure::Bad("Request is incomplete"));
        }
        buffer.extend_from_slice(&chunk[..read]);
    }
}

fn split_host_port(authority: &str) -> Option<(String, u16)> {
    // A CONNECT target always names its port, even when it is the default one.
    let (_, port_text) = authority.rsplit_once(':')?;
    let port = port_text.parse::<u16>().ok()?;
    let url = url::Url::parse(&format!("http://{authority}/")).ok()?;
    let host = url
        .host_str()?
        .trim_matches(|c| c == '[' || c == ']')
        .to_string();
    Some((host, port))
}

fn parse_head(head: &[u8]) -> Result<Request, Failure> {
    let text = std::str::from_utf8(head).map_err(|_| Failure::Bad("Request is not text"))?;
    let mut lines = text.split("\r\n");
    let first = lines.next().unwrap_or_default();
    let mut parts = first.split(' ');
    let (Some(method), Some(target), Some(version), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return Err(Failure::Bad("Request line is not valid"));
    };
    if !version.starts_with("HTTP/1.") {
        return Err(Failure::Bad("Only HTTP/1 requests are supported"));
    }
    let mut headers = Vec::new();
    for line in lines.filter(|line| !line.is_empty()) {
        let (name, value) = line
            .split_once(':')
            .ok_or(Failure::Bad("A header is not valid"))?;
        headers.push((name.trim().to_string(), value.trim().to_string()));
    }
    let target = if method.eq_ignore_ascii_case("CONNECT") {
        let (host, port) =
            split_host_port(target).ok_or(Failure::Bad("CONNECT target is not valid"))?;
        Target::Tunnel { host, port }
    } else {
        let url = url::Url::parse(target).map_err(|_| Failure::Bad("Target is not valid"))?;
        if url.scheme() != "http" || !url.username().is_empty() || url.password().is_some() {
            return Err(Failure::Bad("Only plain HTTP addresses are forwarded"));
        }
        let host = url
            .host_str()
            .ok_or(Failure::Bad("Target has no host"))?
            .trim_matches(|c| c == '[' || c == ']')
            .to_string();
        let mut origin = url.path().to_string();
        if let Some(query) = url.query() {
            origin.push('?');
            origin.push_str(query);
        }
        Target::Forward {
            host,
            port: url.port_or_known_default().unwrap_or(80),
            origin,
        }
    };
    Ok(Request {
        method: method.to_string(),
        version: version.to_string(),
        target,
        headers,
    })
}

/// The request as the origin server should see it: origin-form target, no proxy headers, and one
/// request per connection.
fn forward_head(request: &Request) -> Vec<u8> {
    let Target::Forward { origin, .. } = &request.target else {
        return Vec::new();
    };
    let mut head = format!("{} {} {}\r\n", request.method, origin, request.version);
    for (name, value) in &request.headers {
        let lower = name.to_ascii_lowercase();
        if matches!(
            lower.as_str(),
            "proxy-connection" | "proxy-authorization" | "connection" | "keep-alive"
        ) {
            continue;
        }
        head.push_str(&format!("{name}: {value}\r\n"));
    }
    head.push_str("Connection: close\r\n\r\n");
    head.into_bytes()
}

async fn connect(resolver: &dyn Resolve, host: &str, port: u16) -> Result<TcpStream, Failure> {
    let addresses = match host.parse::<IpAddr>() {
        Ok(ip) => vec![ip],
        Err(_) => resolver
            .resolve(host.to_string())
            .await
            .map_err(Failure::Upstream)?,
    };
    let mut last = "The name has no address".to_string();
    for ip in addresses {
        match timeout(CONNECT_TIMEOUT, TcpStream::connect((ip, port))).await {
            Ok(Ok(stream)) => return Ok(stream),
            Ok(Err(error)) => last = error.to_string(),
            Err(_) => last = "Connection timed out".to_string(),
        }
    }
    Err(Failure::Upstream(last))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{future::Future, pin::Pin};

    struct Fixed(Option<IpAddr>);
    impl Resolve for Fixed {
        fn resolve(
            &self,
            host: String,
        ) -> Pin<Box<dyn Future<Output = Result<Vec<IpAddr>, String>> + Send + '_>> {
            Box::pin(async move {
                match (&self.0, host.as_str()) {
                    (Some(ip), "site.test") => Ok(vec![*ip]),
                    _ => Err("lookup failed".to_string()),
                }
            })
        }
    }

    fn bad(result: Result<Request, Failure>) -> bool {
        matches!(result, Err(Failure::Bad(_)))
    }

    #[test]
    fn connect_and_absolute_requests_are_parsed() {
        let connect =
            parse_head(b"CONNECT site.test:443 HTTP/1.1\r\nHost: site.test:443").unwrap();
        assert_eq!(
            connect.target,
            Target::Tunnel {
                host: "site.test".into(),
                port: 443
            }
        );
        let v6 = parse_head(b"CONNECT [2001:db8::1]:8443 HTTP/1.1\r\n").unwrap();
        assert_eq!(
            v6.target,
            Target::Tunnel {
                host: "2001:db8::1".into(),
                port: 8443
            }
        );
        let get = parse_head(
            b"GET http://site.test:8080/a/b?q=1 HTTP/1.1\r\nHost: site.test:8080\r\nAccept: */*",
        )
        .unwrap();
        assert_eq!(
            get.target,
            Target::Forward {
                host: "site.test".into(),
                port: 8080,
                origin: "/a/b?q=1".into()
            }
        );
        let root = parse_head(b"GET http://site.test HTTP/1.1\r\n").unwrap();
        assert_eq!(
            root.target,
            Target::Forward {
                host: "site.test".into(),
                port: 80,
                origin: "/".into()
            }
        );
    }

    #[test]
    fn requests_that_are_not_proxy_requests_are_refused() {
        assert!(bad(parse_head(b"GET /relative HTTP/1.1\r\n")));
        assert!(bad(parse_head(b"GET https://site.test/ HTTP/1.1\r\n")));
        assert!(bad(parse_head(b"GET http://user:pw@site.test/ HTTP/1.1\r\n")));
        assert!(bad(parse_head(b"CONNECT site.test HTTP/1.1")));
        assert!(parse_head(b"CONNECT site.test:80 HTTP/1.1").is_ok());
        assert!(bad(parse_head(b"CONNECT [2001:db8::1] HTTP/1.1\r\n")));
        assert!(bad(parse_head(b"GET http://site.test/ HTTP/2\r\n")));
        assert!(bad(parse_head(b"nonsense\r\n")));
        assert!(bad(parse_head(
            b"GET http://site.test/ HTTP/1.1\r\nbroken header\r\n"
        )));
    }

    #[test]
    fn the_forwarded_request_is_origin_form_with_one_request_per_connection() {
        let request = parse_head(
            b"POST http://site.test/p?x=1 HTTP/1.1\r\nHost: site.test\r\nProxy-Connection: keep-alive\r\nConnection: keep-alive\r\nProxy-Authorization: Basic abc\r\nContent-Length: 3",
        )
        .unwrap();
        let head = String::from_utf8(forward_head(&request)).unwrap();
        assert_eq!(
            head,
            "POST /p?x=1 HTTP/1.1\r\nHost: site.test\r\nContent-Length: 3\r\nConnection: close\r\n\r\n"
        );
    }

    async fn proxy(resolver: Fixed) -> std::net::SocketAddr {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let resolver: Arc<dyn Resolve> = Arc::new(resolver);
            while let Ok((stream, _)) = listener.accept().await {
                let resolver = resolver.clone();
                tokio::spawn(async move { handle(stream, resolver).await });
            }
        });
        address
    }

    async fn echo_server() -> std::net::SocketAddr {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move {
            while let Ok((mut stream, _)) = listener.accept().await {
                tokio::spawn(async move {
                    let mut buffer = vec![0u8; 4096];
                    let read = stream.read(&mut buffer).await.unwrap_or(0);
                    let _ = stream.write_all(&buffer[..read]).await;
                });
            }
        });
        address
    }

    const ESTABLISHED: &[u8] = b"HTTP/1.1 200 Connection Established\r\n\r\n";

    async fn exchange(
        proxy: std::net::SocketAddr,
        request: &[u8],
        after: Option<&[u8]>,
    ) -> Vec<u8> {
        let mut stream = TcpStream::connect(proxy).await.unwrap();
        stream.write_all(request).await.unwrap();
        if let Some(after) = after {
            let mut head = vec![0u8; ESTABLISHED.len()];
            stream.read_exact(&mut head).await.unwrap();
            assert_eq!(head, ESTABLISHED);
            stream.write_all(after).await.unwrap();
        }
        let mut answer = Vec::new();
        let _ = timeout(Duration::from_secs(3), async {
            let mut chunk = [0u8; 1024];
            // The echo sends one answer; read it and stop without waiting for the close.
            let read = stream.read(&mut chunk).await.unwrap_or(0);
            answer.extend_from_slice(&chunk[..read]);
        })
        .await;
        answer
    }

    #[tokio::test]
    async fn a_tunnel_reaches_the_address_the_resolver_gave() {
        let server = echo_server().await;
        let proxy = proxy(Fixed(Some(server.ip()))).await;
        let request = format!("CONNECT site.test:{} HTTP/1.1\r\n\r\n", server.port());
        let answer = exchange(proxy, request.as_bytes(), Some(b"hello")).await;
        assert_eq!(answer, b"hello");
    }

    #[tokio::test]
    async fn a_plain_request_reaches_the_server_as_origin_form() {
        let server = echo_server().await;
        let proxy = proxy(Fixed(Some(server.ip()))).await;
        let request = format!(
            "GET http://site.test:{}/path HTTP/1.1\r\nHost: site.test\r\n\r\n",
            server.port()
        );
        let answer = String::from_utf8(exchange(proxy, request.as_bytes(), None).await).unwrap();
        assert!(answer.starts_with("GET /path HTTP/1.1\r\nHost: site.test\r\nConnection: close"));
    }

    #[tokio::test]
    async fn a_failed_lookup_is_a_gateway_error_and_never_falls_back_to_system_dns() {
        let proxy = proxy(Fixed(None)).await;
        for request in [
            "CONNECT unknown.test:443 HTTP/1.1\r\n\r\n",
            "GET http://unknown.test/ HTTP/1.1\r\n\r\n",
        ] {
            let answer =
                String::from_utf8(exchange(proxy, request.as_bytes(), None).await).unwrap();
            assert!(answer.starts_with("HTTP/1.1 502 Bad Gateway"), "{answer}");
            assert!(answer.contains("did not fall back to the system DNS"));
        }
    }

    #[tokio::test]
    async fn a_literal_address_needs_no_lookup() {
        let server = echo_server().await;
        let proxy = proxy(Fixed(None)).await;
        let request = format!("CONNECT {}:{} HTTP/1.1\r\n\r\n", server.ip(), server.port());
        assert_eq!(
            exchange(proxy, request.as_bytes(), Some(b"ping")).await,
            b"ping"
        );
    }

    #[tokio::test]
    async fn a_malformed_request_is_a_bad_request() {
        let proxy = proxy(Fixed(None)).await;
        let answer =
            String::from_utf8(exchange(proxy, b"GET /x HTTP/1.1\r\n\r\n", None).await).unwrap();
        assert!(answer.starts_with("HTTP/1.1 400 Bad Request"));
    }
}
