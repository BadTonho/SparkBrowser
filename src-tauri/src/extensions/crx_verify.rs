//! Signature check of CRX3 extension packages, as Chromium does it.
//!
//! A CRX3 file is `Cr24`, the version (3), the header length, a `CrxFileHeader` protobuf and the ZIP
//! archive. The header carries key proofs (a public key and a signature, RSA or ECDSA, both over
//! SHA-256) and `signed_header_data`, whose `crx_id` is the first 16 bytes of the SHA-256 of the
//! developer's public key. Every proof signs `"CRX3 SignedData\0"`, the little-endian length of the
//! signed header data, that data and the archive. The extension id is `crx_id` written with the
//! letters `a`–`p`, so only the holder of the developer key can produce a package for an id.
use ring::signature::{UnparsedPublicKey, ECDSA_P256_SHA256_ASN1, RSA_PKCS1_2048_8192_SHA256};
use sha2::{Digest, Sha256};

const MAGIC: &[u8] = b"Cr24";
const SIGNATURE_CONTEXT: &[u8] = b"CRX3 SignedData\x00";
const FIELD_RSA: u64 = 2;
const FIELD_ECDSA: u64 = 3;
const FIELD_SIGNED_HEADER_DATA: u64 = 10000;
const MAX_PROOFS: usize = 16;

/// A checked package: its id and its ZIP archive.
#[derive(Debug)]
pub struct Verified<'a> {
    pub id: String,
    pub archive: &'a [u8],
}

/// Whether the bytes are a CRX package of any version.
pub fn is_crx(bytes: &[u8]) -> bool {
    bytes.starts_with(MAGIC)
}

/// Checks a CRX3 package. With `expected_id`, the package must also belong to that extension.
pub fn verify<'a>(bytes: &'a [u8], expected_id: Option<&str>) -> Result<Verified<'a>, String> {
    if !is_crx(bytes) || bytes.len() < 12 {
        return Err("Invalid extension package: not a CRX package".into());
    }
    let version = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    if version != 3 {
        return Err(format!(
            "Unsupported extension package: CRX version {version} is not accepted (only CRX3)"
        ));
    }
    let header_len = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]) as usize;
    let header_end = 12usize
        .checked_add(header_len)
        .filter(|end| *end <= bytes.len())
        .ok_or("Invalid extension package: header longer than the file")?;
    let (header, archive) = (&bytes[12..header_end], &bytes[header_end..]);

    let mut proofs = Vec::new();
    let mut signed_header_data = None;
    for (field, value) in fields(header)? {
        match field {
            FIELD_RSA | FIELD_ECDSA => {
                if proofs.len() == MAX_PROOFS {
                    return Err("Invalid extension package: too many key proofs".into());
                }
                proofs.push((field, proof(value)?));
            }
            FIELD_SIGNED_HEADER_DATA => signed_header_data = Some(value),
            _ => {}
        }
    }
    let signed_header_data =
        signed_header_data.ok_or("Invalid extension package: no signed header data")?;
    let crx_id = fields(signed_header_data)?
        .into_iter()
        .find(|(field, _)| *field == 1)
        .map(|(_, value)| value)
        .filter(|value| value.len() == 16)
        .ok_or("Invalid extension package: no package id")?;

    let mut message = Vec::with_capacity(SIGNATURE_CONTEXT.len() + 4 + signed_header_data.len());
    message.extend_from_slice(SIGNATURE_CONTEXT);
    message.extend_from_slice(&(signed_header_data.len() as u32).to_le_bytes());
    message.extend_from_slice(signed_header_data);
    message.extend_from_slice(archive);

    // Every proof present must hold (as in Chromium), and one of them must be the developer key.
    let mut developer_key = false;
    for (field, (spki, signature)) in &proofs {
        let key = subject_public_key(spki)?;
        let valid = if *field == FIELD_RSA {
            UnparsedPublicKey::new(&RSA_PKCS1_2048_8192_SHA256, key).verify(&message, signature)
        } else {
            UnparsedPublicKey::new(&ECDSA_P256_SHA256_ASN1, key).verify(&message, signature)
        };
        if valid.is_err() {
            return Err(
                "The extension package signature is invalid: the file was altered or damaged"
                    .into(),
            );
        }
        developer_key |= Sha256::digest(spki)[..16] == *crx_id;
    }
    if !developer_key {
        return Err("The extension package is not signed by the key of its extension ID".into());
    }
    let id = extension_id(crx_id);
    if let Some(expected) = expected_id {
        if id != expected {
            return Err(format!(
                "The downloaded package is a different extension (expected {expected}, got {id})"
            ));
        }
    }
    Ok(Verified { id, archive })
}

/// The id of an extension from the 16 bytes of its `crx_id`: each nibble as a letter `a`–`p`.
pub fn extension_id(crx_id: &[u8]) -> String {
    crx_id
        .iter()
        .flat_map(|byte| [byte >> 4, byte & 0x0f])
        .map(|nibble| (b'a' + nibble) as char)
        .collect()
}

/// The length-delimited fields of a protobuf message, as (field number, bytes). Other wire types are
/// skipped; a malformed message is refused.
fn fields(mut bytes: &[u8]) -> Result<Vec<(u64, &[u8])>, String> {
    const BAD: &str = "Invalid extension package: malformed header";
    let mut found = Vec::new();
    while !bytes.is_empty() {
        let key = varint(&mut bytes).ok_or(BAD)?;
        match key & 7 {
            0 => {
                varint(&mut bytes).ok_or(BAD)?;
            }
            1 => bytes = bytes.get(8..).ok_or(BAD)?,
            5 => bytes = bytes.get(4..).ok_or(BAD)?,
            2 => {
                let len = usize::try_from(varint(&mut bytes).ok_or(BAD)?).map_err(|_| BAD)?;
                if len > bytes.len() {
                    return Err(BAD.into());
                }
                let (value, rest) = bytes.split_at(len);
                found.push((key >> 3, value));
                bytes = rest;
            }
            _ => return Err(BAD.into()),
        }
    }
    Ok(found)
}

fn varint(bytes: &mut &[u8]) -> Option<u64> {
    let mut value = 0u64;
    for shift in (0..64).step_by(7) {
        let (&byte, rest) = bytes.split_first()?;
        *bytes = rest;
        value |= u64::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            return Some(value);
        }
    }
    None
}

/// `AsymmetricKeyProof { public_key = 1; signature = 2 }`.
fn proof(bytes: &[u8]) -> Result<(&[u8], &[u8]), String> {
    let (mut key, mut signature) = (None, None);
    for (field, value) in fields(bytes)? {
        match field {
            1 => key = Some(value),
            2 => signature = Some(value),
            _ => {}
        }
    }
    key.zip(signature)
        .ok_or_else(|| "Invalid extension package: incomplete key proof".into())
}

/// The key inside a DER `SubjectPublicKeyInfo` (`SEQUENCE { algorithm SEQUENCE, BIT STRING }`), in
/// the form ring expects: the PKCS#1 `RSAPublicKey` or the uncompressed EC point.
fn subject_public_key(spki: &[u8]) -> Result<&[u8], String> {
    const BAD: &str = "Invalid extension package: malformed public key";
    let (tag, body, rest) = der(spki).ok_or(BAD)?;
    if tag != 0x30 || !rest.is_empty() {
        return Err(BAD.into());
    }
    let (tag, _, rest) = der(body).ok_or(BAD)?;
    if tag != 0x30 {
        return Err(BAD.into());
    }
    let (tag, bits, rest) = der(rest).ok_or(BAD)?;
    if tag != 0x03 || !rest.is_empty() {
        return Err(BAD.into());
    }
    match bits.split_first() {
        Some((0, key)) if !key.is_empty() => Ok(key),
        _ => Err(BAD.into()),
    }
}

/// One DER element: (tag, contents, what follows).
fn der(bytes: &[u8]) -> Option<(u8, &[u8], &[u8])> {
    let (&tag, rest) = bytes.split_first()?;
    let (&first, mut rest) = rest.split_first()?;
    let len = if first < 0x80 {
        usize::from(first)
    } else {
        let count = usize::from(first & 0x7f);
        if count == 0 || count > 4 || rest.len() < count {
            return None;
        }
        let len = rest[..count]
            .iter()
            .fold(0usize, |len, byte| (len << 8) | usize::from(*byte));
        rest = &rest[count..];
        len
    };
    (len <= rest.len()).then(|| (tag, &rest[..len], &rest[len..]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ring::rand::SystemRandom;
    use ring::signature::{EcdsaKeyPair, KeyPair, ECDSA_P256_SHA256_ASN1_SIGNING};

    // DER prefix of a P-256 SubjectPublicKeyInfo, followed by the 65-byte uncompressed point.
    const P256_SPKI_PREFIX: &[u8] = &[
        0x30, 0x59, 0x30, 0x13, 0x06, 0x07, 0x2a, 0x86, 0x48, 0xce, 0x3d, 0x02, 0x01, 0x06, 0x08,
        0x2a, 0x86, 0x48, 0xce, 0x3d, 0x03, 0x01, 0x07, 0x03, 0x42, 0x00,
    ];

    fn key() -> EcdsaKeyPair {
        let rng = SystemRandom::new();
        let pkcs8 = EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_ASN1_SIGNING, &rng).unwrap();
        EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_ASN1_SIGNING, pkcs8.as_ref(), &rng).unwrap()
    }

    fn spki(key: &EcdsaKeyPair) -> Vec<u8> {
        [P256_SPKI_PREFIX, key.public_key().as_ref()].concat()
    }

    fn field(number: u64, value: &[u8]) -> Vec<u8> {
        fn push_varint(out: &mut Vec<u8>, mut value: u64) {
            while value >= 0x80 {
                out.push((value as u8) | 0x80);
                value >>= 7;
            }
            out.push(value as u8);
        }
        let mut out = Vec::new();
        push_varint(&mut out, (number << 3) | 2);
        push_varint(&mut out, value.len() as u64);
        out.extend_from_slice(value);
        out
    }

    /// A CRX3 package signed by `signer`, claiming the id of `owner`.
    fn package(signer: &EcdsaKeyPair, owner: &EcdsaKeyPair, archive: &[u8]) -> Vec<u8> {
        let crx_id = Sha256::digest(spki(owner))[..16].to_vec();
        let signed = field(1, &crx_id);
        let mut message = SIGNATURE_CONTEXT.to_vec();
        message.extend_from_slice(&(signed.len() as u32).to_le_bytes());
        message.extend_from_slice(&signed);
        message.extend_from_slice(archive);
        let signature = signer.sign(&SystemRandom::new(), &message).unwrap();
        let proof = [field(1, &spki(signer)), field(2, signature.as_ref())].concat();
        let header = [
            field(FIELD_ECDSA, &proof),
            field(FIELD_SIGNED_HEADER_DATA, &signed),
        ]
        .concat();
        let mut bytes = MAGIC.to_vec();
        bytes.extend_from_slice(&3u32.to_le_bytes());
        bytes.extend_from_slice(&(header.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&header);
        bytes.extend_from_slice(archive);
        bytes
    }

    #[test]
    fn a_package_signed_by_the_key_of_its_id_is_accepted() {
        let developer = key();
        let bytes = package(&developer, &developer, b"PK\x03\x04archive");
        let checked = verify(&bytes, None).unwrap();
        assert_eq!(checked.archive, b"PK\x03\x04archive");
        assert_eq!(checked.id.len(), 32);
        assert!(checked.id.bytes().all(|c| (b'a'..=b'p').contains(&c)));
        assert!(verify(&bytes, Some(&checked.id)).is_ok());
    }

    #[test]
    fn altered_misattributed_or_foreign_packages_are_refused() {
        let developer = key();
        let bytes = package(&developer, &developer, b"PK\x03\x04archive");
        let id = verify(&bytes, None).unwrap().id;

        let mut altered = bytes.clone();
        *altered.last_mut().unwrap() ^= 1;
        assert!(verify(&altered, None).unwrap_err().contains("sign"));

        // Signed by someone else while claiming the developer's id.
        let impostor = package(&key(), &developer, b"PK\x03\x04archive");
        assert!(verify(&impostor, None).unwrap_err().contains("sign"));

        // A valid package of another extension when this one was asked for.
        let other = key();
        let foreign = package(&other, &other, b"PK\x03\x04archive");
        assert!(verify(&foreign, Some(&id))
            .unwrap_err()
            .contains("different extension"));
    }

    #[test]
    fn old_truncated_and_malformed_packages_are_refused() {
        let mut crx2 = MAGIC.to_vec();
        crx2.extend_from_slice(&2u32.to_le_bytes());
        crx2.extend_from_slice(&[0; 8]);
        assert!(verify(&crx2, None).unwrap_err().contains("CRX version 2"));
        let developer = key();
        let bytes = package(&developer, &developer, b"PK");
        for cut in [4, 11, 12, 20, 40] {
            assert!(verify(&bytes[..cut], None).is_err(), "cut at {cut}");
        }
        let mut huge_header = bytes.clone();
        huge_header[8..12].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(verify(&huge_header, None).is_err());
        assert!(verify(b"PK\x03\x04", None).is_err());
    }

    #[test]
    fn ids_use_the_letters_a_to_p() {
        assert_eq!(extension_id(&[0x00, 0x1f, 0xff]), "aabppp");
    }

    /// Downloads a real extension from the Chrome Web Store and checks it (needs the network).
    #[test]
    #[ignore]
    fn a_real_web_store_package_verifies() {
        let id = "ddkjiahejlhfcafbddmgiahcphecmpfh";
        let bytes = super::super::crx::download_webstore_crx(id).unwrap();
        assert_eq!(verify(&bytes, Some(id)).unwrap().id, id);
    }
}
