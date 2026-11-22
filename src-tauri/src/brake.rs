//! The brake on wrong guesses, shared by everything that asks for a secret (the password vault and
//! the app lock): a few free attempts, then every further attempt has to wait, the wait doubling up
//! to a minute.

/// Wrong guesses allowed before every further attempt has to wait.
pub const FREE_ATTEMPTS: u32 = 3;
pub const MAX_DELAY_SECONDS: u64 = 60;

/// How long to wait after `failures` wrong guesses in a row, or `None` while attempts are still free.
pub fn delay_after(failures: u32) -> Option<u64> {
    if failures < FREE_ATTEMPTS {
        return None;
    }
    let exponent = (failures - FREE_ATTEMPTS).min(6);
    Some((1u64 << exponent).min(MAX_DELAY_SECONDS))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_first_attempts_are_free_and_the_wait_doubles_up_to_a_minute() {
        assert_eq!(delay_after(0), None);
        assert_eq!(delay_after(FREE_ATTEMPTS - 1), None);
        let waits: Vec<_> = (FREE_ATTEMPTS..FREE_ATTEMPTS + 8)
            .map(|failures| delay_after(failures).unwrap())
            .collect();
        assert_eq!(waits, [1, 2, 4, 8, 16, 32, 60, 60]);
        assert_eq!(delay_after(u32::MAX), Some(MAX_DELAY_SECONDS));
    }
}
