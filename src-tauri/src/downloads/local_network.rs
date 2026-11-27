//! Keeps public pages from making Spark send requests into the computer or the local network.
//!
//! The video search fetches addresses that the page itself reports (sources, posters, playlists and
//! their segments). Without this check a public site could make Spark request `localhost` or a
//! router at `192.168.x.x`, outside the browser's own protections. An address on the local network
//! is still allowed when the page is local too, such as a site under development.
use std::net::{Ipv4Addr, Ipv6Addr};
use tauri::Url;

fn local_v4(ip: Ipv4Addr) -> bool {
    let [a, b, ..] = ip.octets();
    ip.is_loopback()
        || ip.is_private()
        || ip.is_link_local()
        || ip.is_unspecified()
        || ip.is_broadcast()
        // Carrier-grade NAT, 100.64.0.0/10.
        || (a == 100 && (b & 0xc0) == 64)
}

fn local_v6(ip: Ipv6Addr) -> bool {
    if let Some(v4) = ip.to_ipv4_mapped() {
        return local_v4(v4);
    }
    let first = ip.segments()[0];
    ip.is_loopback()
        || ip.is_unspecified()
        // Unique local, fc00::/7.
        || (first & 0xfe00) == 0xfc00
        // Link local, fe80::/10.
        || (first & 0xffc0) == 0xfe80
}

/// The address points to this computer or to the local network.
pub fn is_local(url: &Url) -> bool {
    match url.host() {
        Some(url::Host::Ipv4(ip)) => local_v4(ip),
        Some(url::Host::Ipv6(ip)) => local_v6(ip),
        Some(url::Host::Domain(name)) => {
            let name = name.trim_end_matches('.').to_ascii_lowercase();
            name == "localhost" || name.ends_with(".localhost")
        }
        None => false,
    }
}

/// `target` may be requested on behalf of `page`: either it is public, or the page is local too.
/// Without a page (an address the user gave) everything is allowed.
pub fn allowed(target: &Url, page: Option<&str>) -> bool {
    if !is_local(target) {
        return true;
    }
    match page {
        None => true,
        Some(page) => Url::parse(page).is_ok_and(|page| is_local(&page)),
    }
}

/// A redirect may not lead from a public address into the local network.
pub fn redirect_allowed(first: &Url, next: &Url) -> bool {
    !is_local(next) || is_local(first)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn url(raw: &str) -> Url {
        Url::parse(raw).unwrap()
    }

    #[test]
    fn local_addresses_are_recognized() {
        for raw in [
            "http://localhost/a",
            "http://dev.localhost./a",
            "http://127.0.0.1/a",
            "http://10.1.2.3/a",
            "http://172.16.0.1/a",
            "http://192.168.0.1/a",
            "http://169.254.169.254/latest",
            "http://100.64.0.1/a",
            "http://0.0.0.0/a",
            "http://[::1]/a",
            "http://[fd00::1]/a",
            "http://[fe80::1]/a",
            "http://[::ffff:192.168.0.1]/a",
        ] {
            assert!(is_local(&url(raw)), "{raw}");
        }
        for raw in [
            "https://example.test/a",
            "http://8.8.8.8/a",
            "http://172.32.0.1/a",
            "http://[2001:db8::1]/a",
            "http://localhost.example.test/a",
        ] {
            assert!(!is_local(&url(raw)), "{raw}");
        }
    }

    #[test]
    fn a_public_page_cannot_reach_the_local_network_but_a_local_page_can() {
        let router = url("http://192.168.0.1/video.m3u8");
        assert!(!allowed(&router, Some("https://example.test/watch")));
        assert!(!allowed(&router, Some("not a page")));
        assert!(allowed(&router, Some("http://localhost:5173/")));
        assert!(allowed(&router, None));
        assert!(allowed(
            &url("https://cdn.example.test/v.mp4"),
            Some("http://localhost/")
        ));
        assert!(!redirect_allowed(&url("https://example.test/a"), &router));
        assert!(redirect_allowed(&url("http://127.0.0.1/a"), &router));
        assert!(redirect_allowed(&router, &url("https://example.test/a")));
    }
}
