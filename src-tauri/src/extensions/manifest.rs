use super::types::{ExtensionAction, ExtensionManifest};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn parse_manifest(manifest_json: &str) -> Result<ExtensionManifest, String> {
    let v: Value =
        serde_json::from_str(manifest_json).map_err(|e| format!("Invalid JSON: {}", e))?;

    let manifest_version = v
        .get("manifest_version")
        .and_then(|mv| mv.as_u64())
        .unwrap_or(2) as u32;

    let name = v
        .get("name")
        .and_then(|n| n.as_str())
        .unwrap_or("Unnamed Extension")
        .to_string();

    let version = v
        .get("version")
        .and_then(|ver| ver.as_str())
        .unwrap_or("0.0.1")
        .to_string();

    let description = v
        .get("description")
        .and_then(|d| d.as_str())
        .map(String::from);

    let permissions = v
        .get("permissions")
        .and_then(|p| p.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|val| val.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let host_permissions = v
        .get("host_permissions")
        .and_then(|hp| hp.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|val| val.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    // Parse action / browser_action / page_action
    let action_val = v
        .get("action")
        .or_else(|| v.get("browser_action"))
        .or_else(|| v.get("page_action"));

    let action = action_val.map(|act| {
        let default_title = act
            .get("default_title")
            .and_then(|t| t.as_str())
            .map(String::from);
        let default_popup = act
            .get("default_popup")
            .and_then(|p| p.as_str())
            .map(String::from);

        let default_icon = if let Some(icon_obj) = act.get("default_icon") {
            if let Some(s) = icon_obj.as_str() {
                Some(s.to_string())
            } else if let Some(map) = icon_obj.as_object() {
                ["128", "48", "32", "24", "19", "16"]
                    .iter()
                    .find_map(|size| map.get(*size).and_then(|s| s.as_str()).map(String::from))
                    .or_else(|| map.values().find_map(|s| s.as_str().map(String::from)))
            } else {
                None
            }
        } else {
            None
        };

        ExtensionAction {
            default_title,
            default_popup,
            default_icon,
        }
    });

    // Parse icons
    let mut icons = HashMap::new();
    if let Some(icons_obj) = v.get("icons").and_then(|i| i.as_object()) {
        for (k, val) in icons_obj {
            if let Some(path_str) = val.as_str() {
                icons.insert(k.clone(), path_str.to_string());
            }
        }
    }

    let options_page = v
        .get("options_ui")
        .and_then(|opt| opt.get("page"))
        .and_then(|p| p.as_str())
        .map(String::from)
        .or_else(|| {
            v.get("options_page")
                .and_then(|p| p.as_str())
                .map(String::from)
        });

    Ok(ExtensionManifest {
        manifest_version,
        name,
        version,
        description,
        permissions,
        host_permissions,
        action,
        options_page,
        icons,
    })
}

pub fn read_manifest_from_directory(dir: &Path) -> Result<(ExtensionManifest, String), String> {
    let manifest_path = dir.join("manifest.json");
    if !manifest_path.exists() {
        return Err(format!("manifest.json not found in {}", dir.display()));
    }
    let content = fs::read_to_string(&manifest_path)
        .map_err(|e| format!("Failed to read manifest.json: {}", e))?;
    let manifest = parse_manifest(&content)?;
    Ok((manifest, content))
}

pub fn load_icon_as_data_url(extension_dir: &Path, icon_relative_path: &str) -> Option<String> {
    let clean_rel = icon_relative_path.trim_start_matches(['/', '\\']);
    let icon_path = extension_dir.join(clean_rel);
    if !icon_path.exists() {
        return None;
    }
    let bytes = fs::read(&icon_path).ok()?;
    let mime = if icon_path.extension().and_then(|e| e.to_str()) == Some("svg") {
        "image/svg+xml"
    } else {
        "image/png"
    };
    Some(format!("data:{};base64,{}", mime, STANDARD.encode(bytes)))
}

pub fn find_best_icon_url(extension_dir: &Path, manifest: &ExtensionManifest) -> Option<String> {
    // Try manifest icons first (128 -> 48 -> 32 -> 16 -> any)
    for size in &["128", "48", "32", "24", "16"] {
        if let Some(rel_path) = manifest.icons.get(*size) {
            if let Some(url) = load_icon_as_data_url(extension_dir, rel_path) {
                return Some(url);
            }
        }
    }
    // Try any other manifest icon
    for rel_path in manifest.icons.values() {
        if let Some(url) = load_icon_as_data_url(extension_dir, rel_path) {
            return Some(url);
        }
    }
    // Try action icon
    if let Some(action) = &manifest.action {
        if let Some(rel_path) = &action.default_icon {
            if let Some(url) = load_icon_as_data_url(extension_dir, rel_path) {
                return Some(url);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_manifest_v2_correctly() {
        let json = r#"{
            "manifest_version": 2,
            "name": "Dark Mode",
            "version": "1.2.3",
            "description": "Inverts colors for reading at night",
            "permissions": ["storage", "tabs"],
            "browser_action": {
                "default_popup": "popup.html",
                "default_icon": {
                    "16": "icons/16.png",
                    "48": "icons/48.png"
                }
            },
            "icons": {
                "48": "icon48.png"
            }
        }"#;

        let manifest = parse_manifest(json).unwrap();
        assert_eq!(manifest.manifest_version, 2);
        assert_eq!(manifest.name, "Dark Mode");
        assert_eq!(manifest.version, "1.2.3");
        assert_eq!(manifest.permissions, vec!["storage", "tabs"]);
        assert!(manifest.action.is_some());
        let action = manifest.action.unwrap();
        assert_eq!(action.default_popup.as_deref(), Some("popup.html"));
        assert_eq!(action.default_icon.as_deref(), Some("icons/48.png"));
    }

    #[test]
    fn parses_manifest_v3_correctly() {
        let json = r#"{
            "manifest_version": 3,
            "name": "Ad Blocker Lite",
            "version": "2.0.0",
            "action": {
                "default_popup": "popup.html"
            },
            "host_permissions": ["*://*/*"]
        }"#;

        let manifest = parse_manifest(json).unwrap();
        assert_eq!(manifest.manifest_version, 3);
        assert_eq!(manifest.name, "Ad Blocker Lite");
        assert_eq!(manifest.host_permissions, vec!["*://*/*"]);
    }
}
