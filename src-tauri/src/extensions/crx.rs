use flate2::read::DeflateDecoder;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Duration;

const MAX_ARCHIVE_BYTES: u64 = 64 * 1024 * 1024;
const MAX_ENTRY_BYTES: u64 = 32 * 1024 * 1024;
const MAX_EXTRACTED_BYTES: u64 = 128 * 1024 * 1024;
const MAX_ENTRIES: usize = 2048;

fn read_bounded(reader: impl Read, limit: u64) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    reader
        .take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > limit {
        return Err("Extension archive exceeds the size limit".into());
    }
    Ok(bytes)
}

pub fn read_archive(path: &Path) -> Result<Vec<u8>, String> {
    let file = fs::File::open(path).map_err(|e| format!("Failed to read archive: {e}"))?;
    read_bounded(file, MAX_ARCHIVE_BYTES)
}

fn decompress_entry(
    data: &[u8],
    method: u16,
    declared: usize,
    total: &mut u64,
) -> Result<Vec<u8>, String> {
    if declared as u64 > MAX_ENTRY_BYTES {
        return Err("Extension entry exceeds the size limit".into());
    }
    let limit = MAX_ENTRY_BYTES.min(MAX_EXTRACTED_BYTES.saturating_sub(*total));
    let result = match method {
        0 => read_bounded(data, limit)?,
        8 => read_bounded(DeflateDecoder::new(data), limit)?,
        _ => return Err(format!("Unsupported compression method in ZIP: {method}")),
    };
    if result.len() != declared {
        return Err("Extension entry size does not match its header".into());
    }
    *total += result.len() as u64;
    Ok(result)
}

fn percent_decode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '%' {
            let h1 = chars.next();
            let h2 = chars.next();
            if let (Some(h1), Some(h2)) = (h1, h2) {
                if let Ok(byte) = u8::from_str_radix(&format!("{h1}{h2}"), 16) {
                    result.push(byte as char);
                    continue;
                }
                result.push('%');
                result.push(h1);
                result.push(h2);
                continue;
            }
        }
        result.push(c);
    }
    result
}

/// Extracts a 32-character Chrome extension ID from a URL, download query string, or raw ID string.
pub fn parse_webstore_id(input: &str) -> Option<String> {
    let clean = input.trim();
    // Check if entire input is already a 32-char ID [a-p]{32}
    if clean.len() == 32 && clean.chars().all(|c| ('a'..='p').contains(&c)) {
        return Some(clean.to_string());
    }

    let decoded = if clean.contains('%') {
        percent_decode(clean)
    } else {
        clean.to_string()
    };

    // Match segment in URL (e.g. /detail/<name>/<id> or /detail/<id> or x=id%3D<id>)
    for text in [&decoded, &clean.to_string()] {
        for part in text.split(['/', '?', '&', '=', '%', '#']) {
            let candidate = part.trim();
            if candidate.len() == 32 && candidate.chars().all(|c| ('a'..='p').contains(&c)) {
                return Some(candidate.to_string());
            }
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
        .connect_timeout(Duration::from_secs(15))
        .timeout(Duration::from_secs(60))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36")
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let response = client
        .get(&url)
        .send()
        .map_err(|e| format!("Failed to download extension: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Download failed with HTTP status {}",
            response.status()
        ));
    }

    read_bounded(response, MAX_ARCHIVE_BYTES)
}

/// Extracts a `.crx` or standard `.zip` archive into target_dir.
pub fn extract_crx_or_zip(bytes: &[u8], target_dir: &Path) -> Result<(), String> {
    if bytes.len() as u64 > MAX_ARCHIVE_BYTES {
        return Err("Extension archive exceeds the size limit".into());
    }
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
                let pubkey_len =
                    u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]) as usize;
                let sig_len =
                    u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]) as usize;
                let offset = 16 + pubkey_len + sig_len;
                if bytes.len() < offset {
                    return Err("CRX2 payload offset truncated".into());
                }
                &bytes[offset..]
            }
            3 => {
                let header_len =
                    u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]) as usize;
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
    // Try to extract using the ZIP Central Directory (EOCD).
    // The Central Directory contains authoritative compressed_size, uncompressed_size,
    // and offsets even when streaming data descriptors (flags bit 3) are used in CRX archives.
    let mut eocd_offset = None;
    if bytes.len() >= 22 {
        let max_search = bytes.len().saturating_sub(65535 + 22);
        for i in (max_search..=bytes.len() - 22).rev() {
            if &bytes[i..i + 4] == b"PK\x05\x06" {
                eocd_offset = Some(i);
                break;
            }
        }
    }

    if let Some(eocd) = eocd_offset {
        let cd_size = u32::from_le_bytes([
            bytes[eocd + 12],
            bytes[eocd + 13],
            bytes[eocd + 14],
            bytes[eocd + 15],
        ]) as usize;
        let cd_offset = u32::from_le_bytes([
            bytes[eocd + 16],
            bytes[eocd + 17],
            bytes[eocd + 18],
            bytes[eocd + 19],
        ]) as usize;

        if cd_offset + cd_size <= bytes.len() {
            return extract_from_central_directory(bytes, cd_offset, cd_size, target_dir);
        }
    }

    // Fallback: sequential local headers (for simple archives without EOCD)
    extract_local_headers_fallback(bytes, target_dir)
}

fn extract_from_central_directory(
    bytes: &[u8],
    cd_offset: usize,
    cd_size: usize,
    target_dir: &Path,
) -> Result<(), String> {
    let mut cursor = cd_offset;
    let cd_end = cd_offset + cd_size;
    let mut total = 0;
    let mut entries = 0;

    while cursor + 46 <= cd_end {
        entries += 1;
        if entries > MAX_ENTRIES {
            return Err("Extension archive has too many entries".into());
        }
        if &bytes[cursor..cursor + 4] != b"PK\x01\x02" {
            break;
        }

        let compression_method = u16::from_le_bytes([bytes[cursor + 10], bytes[cursor + 11]]);
        let compressed_size = u32::from_le_bytes([
            bytes[cursor + 20],
            bytes[cursor + 21],
            bytes[cursor + 22],
            bytes[cursor + 23],
        ]) as usize;
        let uncompressed_size = u32::from_le_bytes([
            bytes[cursor + 24],
            bytes[cursor + 25],
            bytes[cursor + 26],
            bytes[cursor + 27],
        ]) as usize;
        let fn_len = u16::from_le_bytes([bytes[cursor + 28], bytes[cursor + 29]]) as usize;
        let extra_len = u16::from_le_bytes([bytes[cursor + 30], bytes[cursor + 31]]) as usize;
        let comment_len = u16::from_le_bytes([bytes[cursor + 32], bytes[cursor + 33]]) as usize;
        let local_offset = u32::from_le_bytes([
            bytes[cursor + 42],
            bytes[cursor + 43],
            bytes[cursor + 44],
            bytes[cursor + 45],
        ]) as usize;

        let name_start = cursor + 46;
        let name_end = name_start + fn_len;
        if name_end > cd_end {
            return Err("ZIP filename in central directory truncated".into());
        }

        let raw_name = String::from_utf8_lossy(&bytes[name_start..name_end]).to_string();
        cursor = name_end + extra_len + comment_len;

        if local_offset + 30 > bytes.len() {
            return Err("ZIP local header offset out of bounds".into());
        }
        if &bytes[local_offset..local_offset + 4] != b"PK\x03\x04" {
            return Err("Invalid ZIP local header signature".into());
        }

        let local_fn_len =
            u16::from_le_bytes([bytes[local_offset + 26], bytes[local_offset + 27]]) as usize;
        let local_extra_len =
            u16::from_le_bytes([bytes[local_offset + 28], bytes[local_offset + 29]]) as usize;

        let data_start = local_offset + 30 + local_fn_len + local_extra_len;
        let data_end = data_start + compressed_size;
        if data_end > bytes.len() {
            return Err("ZIP entry data out of bounds".into());
        }

        let entry_data = &bytes[data_start..data_end];
        let safe_name = raw_name.replace('\\', "/");
        if !safe_name.contains("..") {
            let out_path = safe_path(target_dir, &safe_name)?;
            if safe_name.ends_with('/') {
                fs::create_dir_all(&out_path).map_err(|e| e.to_string())?;
            } else {
                if let Some(parent) = out_path.parent() {
                    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }

                let decompressed = decompress_entry(
                    entry_data,
                    compression_method,
                    uncompressed_size,
                    &mut total,
                )?;

                fs::write(&out_path, decompressed).map_err(|e| e.to_string())?;
            }
        }
    }

    Ok(())
}

fn extract_local_headers_fallback(bytes: &[u8], target_dir: &Path) -> Result<(), String> {
    let mut cursor = 0;
    let mut total = 0;
    let mut entries = 0;
    while cursor + 30 <= bytes.len() {
        // Local file header signature 0x04034b50
        if &bytes[cursor..cursor + 4] != b"PK\x03\x04" {
            // Reached Central Directory or end of entries
            break;
        }
        entries += 1;
        if entries > MAX_ENTRIES {
            return Err("Extension archive has too many entries".into());
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
            let out_path = safe_path(target_dir, &safe_name)?;

            if safe_name.ends_with('/') {
                fs::create_dir_all(&out_path).map_err(|e| e.to_string())?;
            } else {
                if let Some(parent) = out_path.parent() {
                    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }

                let decompressed = decompress_entry(
                    entry_data,
                    compression_method,
                    uncompressed_size,
                    &mut total,
                )?;

                fs::write(&out_path, decompressed).map_err(|e| e.to_string())?;
            }
        }

        cursor = data_end;
    }

    Ok(())
}

fn safe_path(base: &Path, rel: &str) -> Result<PathBuf, String> {
    if rel.starts_with('/') || rel.contains(':') {
        return Err("Invalid extension archive path".into());
    }
    let mut path = base.to_path_buf();
    for part in rel.split('/') {
        if part.is_empty() || part == "." || part == ".." {
            continue;
        }
        path.push(part);
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_oversized_and_misrepresented_entries() {
        use flate2::{write::DeflateEncoder, Compression};
        use std::io::Write;
        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&vec![b'a'; 1024 * 1024]).unwrap();
        let compressed = encoder.finish().unwrap();
        let mut total = MAX_EXTRACTED_BYTES - 1024;
        assert!(decompress_entry(&compressed, 8, 1024 * 1024, &mut total).is_err());
        assert!(decompress_entry(&compressed, 8, 1, &mut 0).is_err());
        assert!(decompress_entry(&[], 0, MAX_ENTRY_BYTES as usize + 1, &mut 0).is_err());
        assert!(safe_path(Path::new("base"), "C:/escape").is_err());
        assert!(safe_path(Path::new("base"), "/escape").is_err());
    }

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

        let download_url = "https://clients2.google.com/service/update2/crx?response=redirect&os=win&arch=x86-64&os_arch=x86-64&nacl_arch=x86-64&prod=chromecrx&prodchannel=&prodversion=130.0&lang=pt-BR&acceptformat=crx2,crx3&x=id%3Dcjpalhdlnbpafiamejdnhcphjbkeiagm%26installsource%3Dondemand%26uc";
        assert_eq!(
            parse_webstore_id(download_url),
            Some("cjpalhdlnbpafiamejdnhcphjbkeiagm".to_string())
        );

        assert_eq!(parse_webstore_id("https://google.com"), None);
    }

    #[test]
    fn extracts_zip_with_central_directory() {
        use flate2::write::DeflateEncoder;
        use flate2::Compression;
        use std::io::Write;

        let content = b"# MIT License\nCopyright (c) 2026 Test\n";
        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(content).unwrap();
        let compressed = encoder.finish().unwrap();

        let filename = b"LICENSE.md";
        let mut zip = Vec::new();

        // Local header (simulating streaming with size = 0 in local header)
        zip.extend_from_slice(b"PK\x03\x04");
        zip.extend_from_slice(&[20, 0]); // version needed
        zip.extend_from_slice(&[8, 0]); // flags: bit 3 set
        zip.extend_from_slice(&[8, 0]); // compression method: 8 (deflate)
        zip.extend_from_slice(&[0, 0, 0, 0]); // mod time/date
        zip.extend_from_slice(&[0, 0, 0, 0]); // crc32 (zero in local header)
        zip.extend_from_slice(&[0, 0, 0, 0]); // compressed size (zero in local header)
        zip.extend_from_slice(&[0, 0, 0, 0]); // uncompressed size (zero in local header)
        zip.extend_from_slice(&(filename.len() as u16).to_le_bytes());
        zip.extend_from_slice(&[0, 0]); // extra len
        zip.extend_from_slice(filename);
        zip.extend_from_slice(&compressed);

        // Central directory offset
        let cd_offset = zip.len();

        // Central directory header (authoritative sizes)
        zip.extend_from_slice(b"PK\x01\x02");
        zip.extend_from_slice(&[20, 0]); // version made by
        zip.extend_from_slice(&[20, 0]); // version needed
        zip.extend_from_slice(&[8, 0]); // flags
        zip.extend_from_slice(&[8, 0]); // compression: deflate
        zip.extend_from_slice(&[0, 0, 0, 0]); // mod time/date
        zip.extend_from_slice(&[0x12, 0x34, 0x56, 0x78]); // crc32
        zip.extend_from_slice(&(compressed.len() as u32).to_le_bytes());
        zip.extend_from_slice(&(content.len() as u32).to_le_bytes());
        zip.extend_from_slice(&(filename.len() as u16).to_le_bytes());
        zip.extend_from_slice(&[0, 0]); // extra len
        zip.extend_from_slice(&[0, 0]); // comment len
        zip.extend_from_slice(&[0, 0]); // disk start
        zip.extend_from_slice(&[0, 0]); // internal attr
        zip.extend_from_slice(&[0, 0, 0, 0]); // external attr
        zip.extend_from_slice(&[0, 0, 0, 0]); // local header offset = 0
        zip.extend_from_slice(filename);

        let cd_size = zip.len() - cd_offset;

        // End of central directory record (EOCD)
        zip.extend_from_slice(b"PK\x05\x06");
        zip.extend_from_slice(&[0, 0]); // disk num
        zip.extend_from_slice(&[0, 0]); // cd start disk
        zip.extend_from_slice(&[1, 0]); // cd entries on disk
        zip.extend_from_slice(&[1, 0]); // total cd entries
        zip.extend_from_slice(&(cd_size as u32).to_le_bytes());
        zip.extend_from_slice(&(cd_offset as u32).to_le_bytes());
        zip.extend_from_slice(&[0, 0]); // comment len

        let temp_dir = tempfile::tempdir().unwrap();
        extract_crx_or_zip(&zip, temp_dir.path()).unwrap();

        let extracted_file = temp_dir.path().join("LICENSE.md");
        assert!(extracted_file.exists());
        let read_content = fs::read_to_string(&extracted_file).unwrap();
        assert_eq!(read_content, "# MIT License\nCopyright (c) 2026 Test\n");
    }
}
