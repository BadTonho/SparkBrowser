//! The name of this computer, written into every backup so the sync panel can say which computer
//! sent the file in the cloud ("sent by NOTEBOOK").
use crate::error_log::{self, Level};

/// The most characters a device name keeps; longer names are cut.
pub const MAX_DEVICE_CHARS: usize = 64;

/// This computer's name, or "Spark" when the system does not give one (the failure is logged).
pub fn name() -> String {
    match raw_name() {
        Some(raw) => match clean(&raw) {
            Some(name) => name,
            None => fallback("the computer name has no usable characters"),
        },
        None => fallback("the computer name is unavailable"),
    }
}

fn fallback(why: &str) -> String {
    error_log::report(
        Level::Info,
        "backup",
        why,
        "using \"Spark\" as the device name in backups",
    );
    "Spark".into()
}

#[cfg(windows)]
fn raw_name() -> Option<String> {
    std::env::var("COMPUTERNAME").ok()
}

#[cfg(not(windows))]
fn raw_name() -> Option<String> {
    std::fs::read_to_string("/etc/hostname")
        .ok()
        .filter(|text| !text.trim().is_empty())
        .or_else(|| std::env::var("HOSTNAME").ok())
}

/// Trimmed, without control characters and at most [`MAX_DEVICE_CHARS`] long; `None` when nothing is left.
pub fn clean(raw: &str) -> Option<String> {
    let cleaned: String = raw
        .trim()
        .chars()
        .filter(|c| !c.is_control())
        .take(MAX_DEVICE_CHARS)
        .collect();
    let cleaned = cleaned.trim().to_string();
    (!cleaned.is_empty()).then_some(cleaned)
}

/// A device name read from a file: valid when it is what [`clean`] would keep.
pub fn valid(value: &str) -> bool {
    clean(value).as_deref() == Some(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_trimmed_bounded_and_free_of_control_characters() {
        assert_eq!(clean("  NOTEBOOK \n").as_deref(), Some("NOTEBOOK"));
        assert_eq!(clean("PC\u{7}-SALA").as_deref(), Some("PC-SALA"));
        assert_eq!(clean(" \t ").as_deref(), None);
        assert_eq!(
            clean(&"x".repeat(100)).map(|n| n.chars().count()),
            Some(MAX_DEVICE_CHARS)
        );
        assert!(valid("PC-SALA"));
        assert!(!valid(" PC"));
        assert!(!valid(&"x".repeat(MAX_DEVICE_CHARS + 1)));
        assert!(!name().is_empty());
    }
}
