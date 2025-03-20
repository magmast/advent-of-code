use futures::{StreamExt, TryStreamExt, future};
use tokio::{
    fs::File,
    io::{AsyncBufReadExt, BufReader},
};
use tokio_stream::wrappers::LinesStream;

const VOWELS: &str = "aeiou";

const DISALLOWED_SEQUENCES: &[&str] = &["ab", "cd", "pq", "xy"];

#[derive(clap::Subcommand)]
enum Subcommand {
    P1,
    P2,
}

#[derive(clap::Args)]
pub struct Args {
    #[command(subcommand)]
    subcommand: Subcommand,
}

impl Args {
    pub async fn run(&self) -> anyhow::Result<()> {
        match &self.subcommand {
            Subcommand::P1 => Self::answer(is_p1_nice).await,
            Subcommand::P2 => Self::answer(is_p2_nice).await,
        }
    }

    async fn answer(mut check_nice: impl FnMut(&str) -> bool) -> anyhow::Result<()> {
        let input = File::open("inputs/y15_d05.txt").await?;
        let input = BufReader::new(input);
        let nice_count = LinesStream::new(input.lines())
            .try_filter(|line| future::ready(check_nice(line)))
            .count()
            .await;
        println!("Answer: {}", nice_count);
        Ok(())
    }
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
    while let Some(curr) = chars.next() {
        if prev_char == Some(curr) {
            return true;
        }
    }

    false
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

#[cfg(test)]
mod tests {
    use crate::y15::d05::is_p2_nice;

    #[test]
    fn test_is_p2_nice() {
        assert_eq!(is_p2_nice("qjhvhtzxzqqjkmpb"), true);
        assert_eq!(is_p2_nice("xxyxx"), true);
        assert_eq!(is_p2_nice("uurcxstgmygtbstg"), false);
        assert_eq!(is_p2_nice("ieodomkazucvgmuy"), false);
    }
}
