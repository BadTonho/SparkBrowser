//! Two-factor codes (TOTP, RFC 6238) for the logins in the vault. The 2FA secret of a login is
//! optional: it is kept in the encrypted entry as an `otpauth://` link or as the plain setup key
//! the website shows, and the six-digit code is computed here, never in the interface.
use hmac::{Hmac, Mac};
use serde::Serialize;
use sha1::Sha1;
use sha2::{Sha256, Sha512};

/// The longest text accepted for a 2FA secret or `otpauth://` link.
pub const MAX_TOTP: usize = 1024;
const MAX_SECRET_BYTES: usize = 128;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Algorithm {
    Sha1,
    Sha256,
    Sha512,
}

/// A parsed 2FA secret with the settings that shape its codes.
#[derive(Debug, PartialEq)]
pub struct Totp {
    secret: Vec<u8>,
    algorithm: Algorithm,
    digits: u32,
    period: u64,
}

/// What the interface gets: the code and how long it stays valid. Never the secret.
#[derive(Debug, Serialize, PartialEq)]
pub struct TotpCode {
    pub code: String,
    pub seconds_left: u64,
    pub period: u64,
}

/// Decodes RFC 4648 base32, ignoring case, spaces, dashes and padding (as websites format keys).
fn base32_decode(text: &str) -> Option<Vec<u8>> {
    let mut bits = 0u32;
    let mut count = 0u32;
    let mut out = Vec::new();
    for c in text.chars() {
        if c.is_whitespace() || c == '-' || c == '=' {
            continue;
        }
        let value = match c.to_ascii_uppercase() {
            letter @ 'A'..='Z' => letter as u32 - 'A' as u32,
            digit @ '2'..='7' => digit as u32 - '2' as u32 + 26,
            _ => return None,
        };
        bits = (bits << 5) | value;
        count += 5;
        if count >= 8 {
            count -= 8;
            out.push((bits >> count) as u8);
            bits &= (1 << count) - 1;
        }
    }
    (!out.is_empty() && out.len() <= MAX_SECRET_BYTES).then_some(out)
}

fn mac<M: Mac + hmac::digest::KeyInit>(secret: &[u8], counter: u64) -> Vec<u8> {
    let mut mac = <M as Mac>::new_from_slice(secret).expect("HMAC accepts any key length");
    mac.update(&counter.to_be_bytes());
    mac.finalize().into_bytes().to_vec()
}

impl Totp {
    /// Reads an `otpauth://totp/...` link or a bare base32 setup key.
    pub fn parse(input: &str) -> Result<Totp, String> {
        let input = input.trim();
        if input.is_empty() || input.chars().count() > MAX_TOTP {
            return Err("vault.invalid_totp".into());
        }
        if !input.to_ascii_lowercase().starts_with("otpauth://") {
            return Ok(Totp {
                secret: base32_decode(input).ok_or("vault.invalid_totp")?,
                algorithm: Algorithm::Sha1,
                digits: 6,
                period: 30,
            });
        }
        let url = url::Url::parse(input).map_err(|_| "vault.invalid_totp")?;
        if url.scheme() != "otpauth" || url.host_str() != Some("totp") {
            return Err("vault.invalid_totp".into());
        }
        let mut totp = Totp {
            secret: Vec::new(),
            algorithm: Algorithm::Sha1,
            digits: 6,
            period: 30,
        };
        for (key, value) in url.query_pairs() {
            match key.to_ascii_lowercase().as_str() {
                "secret" => totp.secret = base32_decode(&value).ok_or("vault.invalid_totp")?,
                "algorithm" => {
                    totp.algorithm = match value.to_ascii_uppercase().as_str() {
                        "SHA1" => Algorithm::Sha1,
                        "SHA256" => Algorithm::Sha256,
                        "SHA512" => Algorithm::Sha512,
                        _ => return Err("vault.invalid_totp".into()),
                    }
                }
                "digits" => match value.parse::<u32>() {
                    Ok(digits @ 6..=8) => totp.digits = digits,
                    _ => return Err("vault.invalid_totp".into()),
                },
                "period" => match value.parse::<u64>() {
                    Ok(period @ 1..=300) => totp.period = period,
                    _ => return Err("vault.invalid_totp".into()),
                },
                _ => {}
            }
        }
        if totp.secret.is_empty() {
            return Err("vault.invalid_totp".into());
        }
        Ok(totp)
    }

    fn hotp(&self, counter: u64) -> String {
        let digest = match self.algorithm {
            Algorithm::Sha1 => mac::<Hmac<Sha1>>(&self.secret, counter),
            Algorithm::Sha256 => mac::<Hmac<Sha256>>(&self.secret, counter),
            Algorithm::Sha512 => mac::<Hmac<Sha512>>(&self.secret, counter),
        };
        let offset = (digest[digest.len() - 1] & 0x0f) as usize;
        let binary = u32::from_be_bytes([
            digest[offset] & 0x7f,
            digest[offset + 1],
            digest[offset + 2],
            digest[offset + 3],
        ]);
        let code = binary % 10u32.pow(self.digits);
        format!("{code:0width$}", width = self.digits as usize)
    }

    /// The code valid at `unix_seconds`, and how many seconds are left before it changes.
    pub fn code_at(&self, unix_seconds: u64) -> TotpCode {
        TotpCode {
            code: self.hotp(unix_seconds / self.period),
            seconds_left: self.period - unix_seconds % self.period,
            period: self.period,
        }
    }
}

/// The code for a stored secret right now.
pub fn code_now(secret: &str) -> Result<TotpCode, String> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs());
    Ok(Totp::parse(secret)?.code_at(now))
}

#[cfg(test)]
mod tests {
    use super::*;

    // RFC 6238 appendix B test vectors (8 digits), one secret per algorithm.
    fn rfc(algorithm: &str, secret: &[u8], time: u64) -> String {
        let base32: String = {
            const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
            let mut out = String::new();
            let (mut bits, mut count) = (0u32, 0u32);
            for byte in secret {
                bits = (bits << 8) | u32::from(*byte);
                count += 8;
                while count >= 5 {
                    count -= 5;
                    out.push(ALPHABET[((bits >> count) & 31) as usize] as char);
                }
                bits &= (1 << count) - 1;
            }
            if count > 0 {
                out.push(ALPHABET[((bits << (5 - count)) & 31) as usize] as char);
            }
            out
        };
        let link = format!("otpauth://totp/x?secret={base32}&algorithm={algorithm}&digits=8");
        Totp::parse(&link).unwrap().code_at(time).code
    }

    #[test]
    fn it_matches_the_rfc_6238_test_vectors() {
        let sha1 = b"12345678901234567890";
        let sha256 = b"12345678901234567890123456789012";
        let sha512 = b"1234567890123456789012345678901234567890123456789012345678901234";
        assert_eq!(rfc("SHA1", sha1, 59), "94287082");
        assert_eq!(rfc("SHA1", sha1, 1111111109), "07081804");
        assert_eq!(rfc("SHA256", sha256, 59), "46119246");
        assert_eq!(rfc("SHA512", sha512, 59), "90693936");
    }

    #[test]
    fn a_bare_setup_key_gives_six_digits_every_thirty_seconds() {
        // "12345678901234567890" in base32, spaces and lower case as a website may show it.
        let totp = Totp::parse("gezd gnbv gy3t qojq gezd gnbv gy3t qojq").unwrap();
        let code = totp.code_at(59);
        assert_eq!(code.code, "287082");
        assert_eq!(code.seconds_left, 1);
        assert_eq!(code.period, 30);
        assert_eq!(totp.code_at(60).seconds_left, 30);
    }

    #[test]
    fn an_otpauth_link_may_change_digits_and_period() {
        let link = "otpauth://totp/Example:me%40example.test?secret=GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ&issuer=Example&digits=8&period=60";
        let code = Totp::parse(link).unwrap().code_at(59);
        assert_eq!(code.code.len(), 8);
        assert_eq!(code.period, 60);
        assert_eq!(code.seconds_left, 1);
    }

    #[test]
    fn bad_input_is_refused_with_a_stable_code() {
        for bad in [
            "",
            "   ",
            "not base32 !!",
            "otpauth://hotp/x?secret=GEZDGNBVGY3TQOJQ",
            "otpauth://totp/x",
            "otpauth://totp/x?secret=GEZDGNBVGY3TQOJQ&digits=12",
            "otpauth://totp/x?secret=GEZDGNBVGY3TQOJQ&period=0",
            "otpauth://totp/x?secret=GEZDGNBVGY3TQOJQ&algorithm=MD5",
            &"A".repeat(MAX_TOTP + 1),
        ] {
            assert_eq!(Totp::parse(bad).unwrap_err(), "vault.invalid_totp", "{bad}");
        }
    }
}
