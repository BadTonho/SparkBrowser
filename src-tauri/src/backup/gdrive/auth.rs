use super::*;

/// FIPS 180-4 SHA-1 implementation used for Google SAPISIDHASH header generation.
pub fn sha1_hex(data: &[u8]) -> String {
    let mut h0: u32 = 0x67452301;
    let mut h1: u32 = 0xEFCDAB89;
    let mut h2: u32 = 0x98BADCFE;
    let mut h3: u32 = 0x10325476;
    let mut h4: u32 = 0xC3D2E1F0;

    let bit_len = (data.len() as u64).wrapping_mul(8);
    let mut msg = data.to_vec();
    msg.push(0x80);
    while (msg.len() % 64) != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in msg.chunks_exact(64) {
        let mut w = [0u32; 80];
        for (i, word) in chunk.chunks_exact(4).enumerate() {
            w[i] = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
        }
        for i in 16..80 {
            w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
        }

        let (mut a, mut b, mut c, mut d, mut e) = (h0, h1, h2, h3, h4);
        for (i, &wi) in w.iter().enumerate() {
            let (f, k) = match i {
                0..=19 => ((b & c) | ((!b) & d), 0x5A827999u32),
                20..=39 => (b ^ c ^ d, 0x6ED9EBA1u32),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1BBCDCu32),
                _ => (b ^ c ^ d, 0xCA62C1D6u32),
            };
            let temp = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(wi);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }

        h0 = h0.wrapping_add(a);
        h1 = h1.wrapping_add(b);
        h2 = h2.wrapping_add(c);
        h3 = h3.wrapping_add(d);
        h4 = h4.wrapping_add(e);
    }

    format!("{h0:08x}{h1:08x}{h2:08x}{h3:08x}{h4:08x}")
}

pub fn build_sapisidhash_authorization(
    cookies: &HashMap<String, String>,
    timestamp: u64,
) -> Result<String, String> {
    let origin = "https://drive.google.com";
    let mut tokens = Vec::new();
    if let Some(s) = cookies
        .get("SAPISID")
        .or_else(|| cookies.get("__Secure-3PAPISID"))
    {
        let hash = sha1_hex(format!("{timestamp} {s} {origin}").as_bytes());
        tokens.push(format!("SAPISIDHASH {timestamp}_{hash}"));
    }
    if let Some(s1) = cookies.get("__Secure-1PAPISID") {
        let hash = sha1_hex(format!("{timestamp} {s1} {origin}").as_bytes());
        tokens.push(format!("SAPISID1PHASH {timestamp}_{hash}"));
    }
    if let Some(s3) = cookies.get("__Secure-3PAPISID") {
        let hash = sha1_hex(format!("{timestamp} {s3} {origin}").as_bytes());
        tokens.push(format!("SAPISID3PHASH {timestamp}_{hash}"));
    }
    if tokens.is_empty() {
        return Err("backup.sync_gdrive_login_required".into());
    }
    Ok(tokens.join(" "))
}

pub fn extract_api_keys_from_html(html: &str) -> Vec<String> {
    let mut keys = Vec::new();
    let bytes = html.as_bytes();
    if bytes.len() < 40 {
        return keys;
    }
    for i in 0..=(bytes.len() - 39) {
        if &bytes[i..i + 4] == b"AIza" {
            let prev_ok =
                i == 0 || matches!(bytes[i - 1], b'"' | b'\'' | b'=' | b',' | b' ' | b'[');
            let next_ok = i + 39 == bytes.len()
                || matches!(
                    bytes[i + 39],
                    b'"' | b'\'' | b'&' | b',' | b' ' | b']' | b'}' | b'\\'
                );
            if prev_ok
                && next_ok
                && bytes[i + 4..i + 39]
                    .iter()
                    .all(|b| b.is_ascii_alphanumeric() || *b == b'_' || *b == b'-')
            {
                let candidate = String::from_utf8_lossy(&bytes[i..i + 39]).to_string();
                let ctx_start = i.saturating_sub(140);
                let ctx_end = (i + 180).min(bytes.len());
                let ctx = String::from_utf8_lossy(&bytes[ctx_start..ctx_end]);
                let is_drive = ctx.contains("clients6.google.com") || ctx.contains("/drive/");
                if let Some(pos) = keys.iter().position(|k| k == &candidate) {
                    if is_drive && pos > 0 {
                        keys.remove(pos);
                        keys.insert(0, candidate);
                    }
                } else if is_drive {
                    keys.insert(0, candidate);
                } else {
                    keys.push(candidate);
                }
            }
        }
    }
    keys
}

pub fn extract_email_from_text(text: &str) -> Option<String> {
    for suffix in ["@gmail.com", "@googlemail.com"] {
        if let Some(at_pos) = text.find(suffix) {
            let prefix = &text[..at_pos];
            let user_part: String = prefix
                .chars()
                .rev()
                .take_while(|c| {
                    c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '%' | '+' | '-')
                })
                .collect::<String>()
                .chars()
                .rev()
                .collect();
            if !user_part.is_empty() && user_part.len() <= 64 {
                return Some(format!("{user_part}{suffix}"));
            }
        }
    }
    None
}

pub fn build_folder_metadata_json(is_v2: bool) -> serde_json::Value {
    if is_v2 {
        serde_json::json!({
            "title": SYNC_FOLDER_NAME,
            "mimeType": FOLDER_MIME_TYPE,
            "parents": [{ "id": "root" }]
        })
    } else {
        serde_json::json!({
            "name": SYNC_FOLDER_NAME,
            "mimeType": FOLDER_MIME_TYPE,
            "parents": ["root"]
        })
    }
}
