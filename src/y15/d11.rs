use anyhow::{Context, Result};
use itertools::Itertools;

const INPUT: &str = "vzbxkghb";

const INVALID_CHARS: &str = "iol";

fn contains_invalid_chars(s: &str) -> bool {
    s.contains(|c| INVALID_CHARS.contains(c))
}

fn has_sequential_triplet(s: &str) -> bool {
    s.chars()
        .tuple_windows()
        .map(|(a, b, c)| (a as u8, b as u8, c as u8))
        .any(|(a, b, c)| a + 1 == b && b + 1 == c)
}

fn has_two_distinct_pairs(s: &str) -> bool {
    s.chars()
        .tuple_windows()
        .filter(|(a, b)| a == b)
        .map(|(a, _)| a)
        .dedup()
        .count()
        >= 2
}

fn is_password_valid(s: &str) -> bool {
    !contains_invalid_chars(s) && has_sequential_triplet(s) && has_two_distinct_pairs(s)
}

struct PasswordIterator {
    current: String,
}

impl PasswordIterator {
    fn new(current: impl ToString) -> Self {
        Self {
            current: current.to_string(),
        }
    }
}

fn increment_password(bytes: impl AsRef<[u8]>) -> String {
    let mut bytes = bytes.as_ref().to_owned();
    while let Some((i, byte)) = bytes
        .iter()
        .find_position(|c| INVALID_CHARS.as_bytes().contains(c))
    {
        bytes[i] = byte + 1;
        for byte in &mut bytes[(i + 1)..] {
            *byte = b'a';
        }
    }

    if let Some(i) = bytes
        .iter()
        .enumerate()
        .rfind(|(_, b)| **b < b'z')
        .map(|(i, _)| i)
    {
        bytes[i] += 1;
        for byte in &mut bytes[(i + 1)..] {
            *byte = b'a';
        }
        String::from_utf8(bytes).unwrap()
    } else {
        for byte in &mut bytes {
            *byte = b'a';
        }
        bytes.insert(0, b'a');
        String::from_utf8(bytes).unwrap()
    }
}

impl Iterator for PasswordIterator {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let pass = increment_password(&self.current);
        self.current = pass.clone();
        Some(pass)
    }
}

fn find_next_password(curr: &str) -> Result<String> {
    PasswordIterator::new(curr)
        .find(|password| is_password_valid(password))
        .context("No valid password found")
}

fn find_first_password() -> Result<String> {
    find_next_password(INPUT)
}

pub async fn p1() -> Result<()> {
    let password = find_first_password()?;
    println!("Answer: {}", password);
    Ok(())
}

pub async fn p2() -> Result<()> {
    let password = find_first_password().and_then(|pass| find_next_password(&pass))?;
    println!("Answer: {}", password);
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::y15::d11::{PasswordIterator, is_password_valid};

    #[test]
    fn test_password_iterator() {
        assert_eq!(PasswordIterator::new("a").next(), Some("b".to_string()));
        assert_eq!(PasswordIterator::new("z").next(), Some("aa".to_string()));
        assert_eq!(PasswordIterator::new("zz").next(), Some("aaa".to_string()));
        assert_eq!(PasswordIterator::new("azc").next(), Some("azd".to_string()));
    }

    #[test]
    fn test_is_password_valid() {
        assert!(is_password_valid("abcdffaa"));
        assert!(is_password_valid("ghjaabcc"));
        assert!(!is_password_valid("hijklmmn"));
    }
}
