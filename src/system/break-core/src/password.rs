use sha2::{Digest, Sha256};

const SCHEME: &str = "sha256i";
const ROUNDS: u32 = 200_000;
const SALT_BYTES: usize = 16;
pub const MIN_LENGTH: usize = 4;

#[derive(Debug, thiserror::Error)]
pub enum PasswordError {
    #[error("비밀번호는 {MIN_LENGTH}자 이상이어야 합니다")]
    TooShort,
    #[error("난수를 만들지 못했습니다: {0}")]
    Random(String),
}

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn from_hex(text: &str) -> Option<Vec<u8>> {
    if text.len() % 2 != 0 {
        return None;
    }
    (0..text.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&text[index..index + 2], 16).ok())
        .collect()
}

fn stretch(salt: &[u8], password: &str, rounds: u32) -> Vec<u8> {
    let mut digest = Sha256::new()
        .chain_update(salt)
        .chain_update(password.as_bytes())
        .finalize()
        .to_vec();
    for _ in 1..rounds {
        digest = Sha256::new()
            .chain_update(salt)
            .chain_update(&digest)
            .finalize()
            .to_vec();
    }
    digest
}

pub fn hash(password: &str) -> Result<String, PasswordError> {
    if password.chars().count() < MIN_LENGTH {
        return Err(PasswordError::TooShort);
    }
    let mut salt = [0u8; SALT_BYTES];
    getrandom::getrandom(&mut salt).map_err(|err| PasswordError::Random(err.to_string()))?;
    let digest = stretch(&salt, password, ROUNDS);
    Ok(format!(
        "{SCHEME}${ROUNDS}${}${}",
        to_hex(&salt),
        to_hex(&digest)
    ))
}

pub fn verify(password: &str, stored: &str) -> bool {
    let parts: Vec<&str> = stored.split('$').collect();
    let [scheme, rounds, salt, digest] = parts.as_slice() else {
        return false;
    };
    if *scheme != SCHEME {
        return false;
    }
    let (Ok(rounds), Some(salt), Some(expected)) =
        (rounds.parse::<u32>(), from_hex(salt), from_hex(digest))
    else {
        return false;
    };
    let actual = stretch(&salt, password, rounds);
    actual.len() == expected.len()
        && actual
            .iter()
            .zip(expected.iter())
            .fold(0u8, |acc, (left, right)| acc | (left ^ right))
            == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_right_password_verifies() {
        let stored = hash("오늘은참자").expect("hash");
        assert!(verify("오늘은참자", &stored));
    }

    #[test]
    fn a_wrong_password_does_not_verify() {
        let stored = hash("오늘은참자").expect("hash");
        assert!(!verify("오늘은 참자", &stored));
    }

    #[test]
    fn the_same_password_gets_a_different_salt_each_time() {
        assert_ne!(hash("abcd").expect("hash"), hash("abcd").expect("hash"));
    }

    #[test]
    fn a_short_password_is_refused() {
        assert!(matches!(hash("abc"), Err(PasswordError::TooShort)));
    }

    #[test]
    fn a_malformed_hash_never_verifies() {
        assert!(!verify("abcd", "plain-text"));
        assert!(!verify("abcd", "sha256i$x$00$00"));
    }
}
