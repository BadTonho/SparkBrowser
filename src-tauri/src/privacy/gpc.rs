use super::tracking_parameters::TrackingSettings;

/// Page-visible signal. Cross-origin frames evaluate their own host exceptions.
pub fn initializer(settings: &TrackingSettings) -> String {
    if !settings.enabled {
        return String::new();
    }
    let exceptions = serde_json::to_string(&settings.exceptions).expect("domains are serializable");
    format!(
        r#"(() => {{
        if (!['http:', 'https:'].includes(location.protocol)) return;
        const host = location.hostname.toLowerCase().replace(/\.$/, '');
        let topHost;
        try {{ topHost = window.top.location.hostname.toLowerCase().replace(/\.$/, ''); }}
        catch {{ return; }} // Do not disclose a signal when the top-site exception is unknown.
        if ({exceptions}.some(domain => [host, topHost].some(value => value === domain || value.endsWith('.' + domain)))) return;
        Object.defineProperty(navigator, 'globalPrivacyControl', {{ configurable: false, get: () => true }});
    }})();"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn disabled_has_no_property_script() {
        assert!(initializer(&TrackingSettings::default()).is_empty());
    }
}
