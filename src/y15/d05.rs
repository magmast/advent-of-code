use futures::{StreamExt, TryStreamExt, future};
use tokio::{
    fs::File,
    io::{AsyncBufReadExt, BufReader},
};
use tokio_stream::wrappers::LinesStream;

const VOWELS: &str = "aeiou";

const DISALLOWED_SEQUENCES: &[&str] = &["ab", "cd", "pq", "xy"];

async fn answer(is_nice: impl Fn(&str) -> bool) -> anyhow::Result<()> {
    let input = File::open("inputs/y15_d05.txt").await?;
    let input = BufReader::new(input);
    let nice_count = LinesStream::new(input.lines())
        .try_filter(|line| future::ready(is_nice(line)))
        .count()
        .await;
    println!("Answer: {}", nice_count);
    Ok(())
}

fn is_p1_nice(line: &str) -> bool {
    if DISALLOWED_SEQUENCES.iter().any(|seq| line.contains(seq)) {
        return false;
    }

    if line.chars().filter(|ch| VOWELS.contains(*ch)).count() < 3 {
        return false;
    }

    let mut chars = line.chars();
    let prev_char = chars.next();
    for curr in chars {
        if prev_char == Some(curr) {
            return true;
        }
    }

    false
}

pub async fn p1() -> anyhow::Result<()> {
    answer(is_p1_nice).await
}

fn is_p2_nice(line: &str) -> bool {
    let line = line.as_bytes();

    let mut has_interloped = false;
    for i in 0..(line.len() - 2) {
        if line[i] == line[i + 2] {
            has_interloped = true;
            break;
        }
    }
    if !has_interloped {
        return false;
    }

    let mut pairs = vec![];
    for i in 0..(line.len() - 1) {
        pairs.push(&line[i..=i + 1]);
    }

    pairs
        .iter()
        .enumerate()
        .any(|(i, a)| pairs.iter().skip(i + 2).any(|b| a == b))
}

pub async fn p2() -> anyhow::Result<()> {
    answer(is_p2_nice).await
}

#[cfg(test)]
mod tests {
    use crate::y15::d05::is_p2_nice;

    #[test]
    fn test_is_p2_nice() {
        assert!(is_p2_nice("qjhvhtzxzqqjkmpb"));
        assert!(is_p2_nice("xxyxx"));
        assert!(!is_p2_nice("uurcxstgmygtbstg"));
        assert!(!is_p2_nice("ieodomkazucvgmuy"));
    }
}
