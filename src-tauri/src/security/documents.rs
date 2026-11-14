use tauri::{AppHandle, Url};

pub fn asset_base(app: &AppHandle) -> Url {
    if tauri::is_dev() {
        if let Some(url) = &app.config().build.dev_url {
            return url.clone();
        }
    }
    Url::parse(if cfg!(windows) {
        "http://tauri.localhost/"
    } else {
        "tauri://localhost/"
    })
    .expect("static asset origin")
}

pub fn shell_url_allowed(base: &Url, url: &Url) -> bool {
    // `Url::origin` is opaque (never equal) for custom schemes such as `tauri://`, so compare the
    // tuple explicitly; for HTTP(S) this is exactly the origin.
    base.scheme() == url.scheme()
        && base.host_str().is_some()
        && base.host_str() == url.host_str()
        && base.port_or_known_default() == url.port_or_known_default()
        && url.username().is_empty()
        && url.password().is_none()
        // Non-HTTP schemes such as `tauri://localhost` (Linux/macOS release builds) are not
        // normalized by `Url`, so the root document has an empty path.
        && matches!(url.path(), "" | "/" | "/index.html")
        && url.query().is_none()
}

pub fn is_shell(app: &AppHandle, url: &Url) -> bool {
    shell_url_allowed(&asset_base(app), url)
}

pub fn same_document(actual: &Url, expected: &Url) -> bool {
    actual == expected && actual.username().is_empty() && actual.password().is_none()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn the_custom_protocol_root_without_a_path_is_the_shell() {
        let base = Url::parse("tauri://localhost/").unwrap();
        for url in [
            "tauri://localhost",
            "tauri://localhost/",
            "tauri://localhost/index.html",
        ] {
            assert!(shell_url_allowed(&base, &Url::parse(url).unwrap()), "{url}");
        }
        for url in [
            "tauri://evil",
            "tauri://localhost/other.html",
            "tauri://localhost?x=1",
        ] {
            assert!(
                !shell_url_allowed(&base, &Url::parse(url).unwrap()),
                "{url}"
            );
        }
    }

    #[test]
    fn only_the_exact_shell_origin_and_entrypoints_are_privileged() {
        let base = Url::parse("http://localhost:1420/").unwrap();
        for url in ["http://localhost:1420/", "http://localhost:1420/index.html"] {
            assert!(shell_url_allowed(&base, &Url::parse(url).unwrap()));
        }
        for url in [
            "http://127.0.0.1:1420/",
            "http://localhost:1421/",
            "https://localhost:1420/",
            "http://localhost:1420/index.html?surface=tools",
            "http://localhost:1420/new-tab.html",
            "http://user@localhost:1420/",
            "about:blank",
            "file:///index.html",
            "https://example.com/",
        ] {
            assert!(!shell_url_allowed(&base, &Url::parse(url).unwrap()));
        }
    }
}
