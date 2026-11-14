use super::model::{UpdateChannel, UpdateInfo};
use semver::Version;
use serde::Deserialize;

pub const REPOSITORY: &str = "BadTonho/SparkBrowser";
pub const TARGET: &str = "windows-x86_64";

#[derive(Debug, Clone, Deserialize)]
pub struct Asset {
    pub name: String,
    pub browser_download_url: String,
}
#[derive(Debug, Clone, Deserialize)]
pub struct Release {
    pub tag_name: String,
    pub draft: bool,
    pub prerelease: bool,
    pub published_at: Option<String>,
    #[cfg_attr(not(windows), allow(dead_code))]
    pub body: Option<String>,
    pub assets: Vec<Asset>,
}

pub fn tag_version(tag: &str) -> Option<Version> {
    let raw = tag.strip_prefix('v')?;
    let version = Version::parse(raw).ok()?;
    if !version.build.is_empty() || version.to_string() != raw {
        return None;
    }
    if !version.pre.is_empty() {
        let number = version.pre.as_str().strip_prefix("beta.")?;
        if number.is_empty() || !number.bytes().all(|v| v.is_ascii_digit()) || number == "0" {
            return None;
        }
    }
    Some(version)
}
pub fn candidates(
    releases: Vec<Release>,
    current: &Version,
    channel: UpdateChannel,
) -> Vec<Release> {
    let mut items: Vec<_> = releases
        .into_iter()
        .filter_map(|release| {
            let version = tag_version(&release.tag_name)?;
            if release.draft
                || release.published_at.is_none()
                || release.prerelease == version.pre.is_empty()
                || version <= *current
                || (channel == UpdateChannel::Stable && !version.pre.is_empty())
            {
                return None;
            }
            Some((version, release))
        })
        .collect();
    items.sort_by(|a, b| b.0.cmp(&a.0));
    items.into_iter().map(|(_, release)| release).collect()
}
pub fn asset_url(tag: &str, name: &str) -> String {
    format!("https://github.com/{REPOSITORY}/releases/download/{tag}/{name}")
}
pub fn manifest_url(release: &Release) -> Result<String, String> {
    let expected = asset_url(&release.tag_name, "updater.json");
    if !release
        .assets
        .iter()
        .any(|a| a.name == "updater.json" && a.browser_download_url == expected)
    {
        return Err(format!(
            "Release {} is missing updater.json. Ask the publisher to complete it.",
            release.tag_name
        ));
    }
    Ok(expected)
}
/// Returns false only for a well-formed manifest explicitly targeting another architecture.
pub fn validate_manifest(release: &Release, manifest: &serde_json::Value) -> Result<bool, String> {
    let expected = tag_version(&release.tag_name).ok_or("Invalid release tag")?;
    let version = manifest
        .get("version")
        .and_then(|v| v.as_str())
        .ok_or("Manifest version is missing")?;
    if Version::parse(version.strip_prefix('v').unwrap_or(version)).ok() != Some(expected) {
        return Err("Release tag and manifest version do not match".into());
    }
    let platforms = manifest
        .get("platforms")
        .and_then(|v| v.as_object())
        .ok_or("Manifest platforms are missing")?;
    if platforms.is_empty() {
        return Err("Manifest platforms are empty".into());
    }
    let Some(platform) = platforms.get(TARGET) else {
        return Ok(false);
    };
    let url = platform
        .get("url")
        .and_then(|v| v.as_str())
        .ok_or("Installer URL is missing")?;
    let signature = platform
        .get("signature")
        .and_then(|v| v.as_str())
        .ok_or("Installer signature is missing")?;
    let asset = release
        .assets
        .iter()
        .find(|a| {
            a.browser_download_url == url
                && a.name.ends_with("-setup.exe")
                && url == asset_url(&release.tag_name, &a.name)
        })
        .ok_or("Installer must belong to the selected GitHub release")?;
    if signature.trim().is_empty()
        || !release.assets.iter().any(|a| {
            a.name == format!("{}.sig", asset.name)
                && a.browser_download_url == asset_url(&release.tag_name, &a.name)
        })
    {
        return Err("Release installer signature is missing".into());
    }
    Ok(true)
}
#[cfg_attr(not(windows), allow(dead_code))]
pub fn info(release: &Release) -> UpdateInfo {
    UpdateInfo {
        version: tag_version(&release.tag_name)
            .expect("validated tag")
            .to_string(),
        tag: release.tag_name.clone(),
        notes: release.body.clone().unwrap_or_default(),
        published_at: release.published_at.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn release(tag: &str) -> Release {
        Release {
            tag_name: tag.into(),
            draft: false,
            prerelease: tag.contains('-'),
            published_at: Some("2026-01-01T00:00:00Z".into()),
            body: None,
            assets: vec![],
        }
    }
    #[test]
    fn channels_order_semver_and_never_downgrade() {
        let releases = vec![
            release("v1.1.0-beta.2"),
            release("v1.1.0-beta.10"),
            release("v1.0.0"),
            release("v1.1.0"),
            release("v2.0.0-rc.1"),
        ];
        let current = Version::parse("1.0.0").unwrap();
        assert_eq!(
            candidates(releases.clone(), &current, UpdateChannel::Beta)
                .iter()
                .map(|r| r.tag_name.as_str())
                .collect::<Vec<_>>(),
            ["v1.1.0", "v1.1.0-beta.10", "v1.1.0-beta.2"]
        );
        assert_eq!(
            candidates(releases, &current, UpdateChannel::Stable).len(),
            1
        );
        assert!(candidates(
            vec![release("v1.0.0")],
            &Version::parse("1.1.0-beta.1").unwrap(),
            UpdateChannel::Stable
        )
        .is_empty());
        assert!(tag_version("v1.1.0-beta.01").is_none());
        assert!(tag_version("v1.1.0-beta.0").is_none());
        assert!(tag_version("v1.1.0+build").is_none());
        let mut invalid = release("v2.0.0-beta.1");
        invalid.prerelease = false;
        assert!(candidates(vec![invalid], &current, UpdateChannel::Beta).is_empty());
    }
    #[test]
    fn manifest_must_match_tag_target_and_release_assets() {
        let mut r = release("v1.1.0");
        for name in [
            "updater.json",
            "Spark_1.1.0_x64-setup.exe",
            "Spark_1.1.0_x64-setup.exe.sig",
        ] {
            r.assets.push(Asset {
                name: name.into(),
                browser_download_url: asset_url(&r.tag_name, name),
            });
        }
        let mut m = serde_json::json!({"version":"1.1.0","platforms":{TARGET:{"url":r.assets[1].browser_download_url,"signature":"signature"}}});
        assert!(manifest_url(&r).is_ok());
        assert_eq!(validate_manifest(&r, &m), Ok(true));
        m["version"] = "1.2.0".into();
        assert!(validate_manifest(&r, &m).is_err());
        m["version"] = "1.1.0".into();
        m["platforms"][TARGET]["url"] = "https://other.example/file.exe".into();
        assert!(validate_manifest(&r, &m).is_err());
        m["platforms"] = serde_json::json!({"linux-x86_64": {}});
        assert_eq!(validate_manifest(&r, &m), Ok(false));
        m["platforms"] = serde_json::json!({});
        assert!(validate_manifest(&r, &m).is_err());
    }
}
