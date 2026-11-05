use flate2::read::DeflateDecoder;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

/// Extracts a 32-character Chrome extension ID from a URL or raw ID string.
pub fn parse_webstore_id(input: &str) -> Option<String> {
    let clean = input.trim();
    // Check if entire input is already a 32-char ID [a-p]{32}
    if clean.len() == 32 && clean.chars().all(|c| ('a'..='p').contains(&c)) {
        return Some(clean.to_string());
    }

    // Match segment in URL (e.g. /detail/<name>/<id> or /detail/<id>)
    for part in clean.split(['/', '?', '&', '=']) {
        if part.len() == 32 && part.chars().all(|c| ('a'..='p').contains(&c)) {
            return Some(part.to_string());
        }
    }
    None
}

/// Downloads a CRX package from Google's official update server for a given extension ID.
pub fn download_webstore_crx(extension_id: &str) -> Result<Vec<u8>, String> {
    let url = format!(
        "https://clients2.google.com/service/update2/crx?response=redirect&prodversion=130.0&acceptformat=crx2,crx3&x=id%3D{}%26uc",
        extension_id
    );

    let client = reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36")
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let response = client
        .get(&url)
        .send()
        .map_err(|e| format!("Failed to download extension: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Download failed with HTTP status {}", response.status()));
    }

    response
        .bytes()
        .map(|b| b.to_vec())
        .map_err(|e| format!("Failed to read response body: {}", e))
}

/// Extracts a `.crx` or standard `.zip` archive into target_dir.
pub fn extract_crx_or_zip(bytes: &[u8], target_dir: &Path) -> Result<(), String> {
    if !target_dir.exists() {
        fs::create_dir_all(target_dir).map_err(|e| e.to_string())?;
    }

    let zip_bytes = if bytes.starts_with(b"Cr24") {
        if bytes.len() < 16 {
            return Err("Invalid CRX header".into());
        }
        let version = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        match version {
            2 => {
                let pubkey_len = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]) as usize;
                let sig_len = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]) as usize;
                let offset = 16 + pubkey_len + sig_len;
                if bytes.len() < offset {
                    return Err("CRX2 payload offset truncated".into());
                }
                &bytes[offset..]
            }
            3 => {
                let header_len = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]) as usize;
                let offset = 12 + header_len;
                if bytes.len() < offset {
                    return Err("CRX3 payload offset truncated".into());
                }
                &bytes[offset..]
            }
            _ => return Err(format!("Unsupported CRX version: {}", version)),
        }
    } else if bytes.starts_with(b"PK\x03\x04") {
        bytes
    } else {
        return Err("Unrecognized file format: neither CRX nor ZIP".into());
    };

    extract_zip_stream(zip_bytes, target_dir)
}

fn extract_zip_stream(bytes: &[u8], target_dir: &Path) -> Result<(), String> {
    let mut cursor = 0;
    while cursor + 30 <= bytes.len() {
        // Local file header signature 0x04034b50
        if &bytes[cursor..cursor + 4] != b"PK\x03\x04" {
            // Reached Central Directory or end of entries
            break;
        }

        let compression_method = u16::from_le_bytes([bytes[cursor + 8], bytes[cursor + 9]]);
        let compressed_size = u32::from_le_bytes([
            bytes[cursor + 18],
            bytes[cursor + 19],
            bytes[cursor + 20],
            bytes[cursor + 21],
        ]) as usize;
        let uncompressed_size = u32::from_le_bytes([
            bytes[cursor + 22],
            bytes[cursor + 23],
            bytes[cursor + 24],
            bytes[cursor + 25],
        ]) as usize;
        let fn_len = u16::from_le_bytes([bytes[cursor + 26], bytes[cursor + 27]]) as usize;
        let extra_len = u16::from_le_bytes([bytes[cursor + 28], bytes[cursor + 29]]) as usize;

        let name_start = cursor + 30;
        let name_end = name_start + fn_len;
        if name_end > bytes.len() {
            return Err("ZIP filename truncated".into());
        }

        let raw_name = String::from_utf8_lossy(&bytes[name_start..name_end]).to_string();
        let data_start = name_end + extra_len;
        let data_end = data_start + compressed_size;

        if data_end > bytes.len() {
            return Err("ZIP entry data truncated".into());
        }

        let entry_data = &bytes[data_start..data_end];

        // Sanitize path against directory traversal
        let safe_name = raw_name.replace('\\', "/");
        if !safe_name.contains("..") {
            let out_path = safe_path(target_dir, &safe_name);

            if safe_name.ends_with('/') {
                fs::create_dir_all(&out_path).map_err(|e| e.to_string())?;
            } else {
                if let Some(parent) = out_path.parent() {
                    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }

                let decompressed = match compression_method {
                    0 => entry_data.to_vec(),
                    8 => {
                        let mut decoder = DeflateDecoder::new(entry_data);
                        let mut buf = Vec::with_capacity(uncompressed_size);
                        decoder.read_to_end(&mut buf).map_err(|e| {
                            format!("Failed to decompress {}: {}", safe_name, e)
                        })?;
                        buf
                    }
                    m => return Err(format!("Unsupported compression method in ZIP: {}", m)),
                };

                fs::write(&out_path, decompressed).map_err(|e| e.to_string())?;
            }
        }

        cursor = data_end;
    }

    Ok(())
}

fn safe_path(base: &Path, rel: &str) -> PathBuf {
    let mut path = base.to_path_buf();
    for part in rel.split('/') {
        if part.is_empty() || part == "." || part == ".." {
            continue;
        }
        path.push(part);
    }
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_webstore_ids_correctly() {
        let id = "cjpalhdlnbpafiamejdnhcphjbkeiagm";
        assert_eq!(parse_webstore_id(id), Some(id.to_string()));

        let url = "https://chromewebstore.google.com/detail/ublock-origin/cjpalhdlnbpafiamejdnhcphjbkeiagm";
        assert_eq!(parse_webstore_id(url), Some(id.to_string()));

        let legacy_url = "https://chrome.google.com/webstore/detail/dark-reader/eimadpbcbfnmbkopoojfekhnkhdbieeh?hl=en";
        assert_eq!(
            parse_webstore_id(legacy_url),
            Some("eimadpbcbfnmbkopoojfekhnkhdbieeh".to_string())
        );

        assert_eq!(parse_webstore_id("https://google.com"), None);
    }
}

