use anyhow::Context;
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};

const INPUT: &str = "iwrupvqb";

fn find_suffix_for_md5_prefix(prefix: &str) -> Option<u32> {
    (0..u32::MAX)
        .into_par_iter()
        .by_exponential_blocks()
        .find_first(|i| {
            format!("{:x}", md5::compute(format!("{}{}", INPUT, i))).starts_with(prefix)
        })
}

fn answer(prefix: &str) -> anyhow::Result<()> {
    let suffix = find_suffix_for_md5_prefix(prefix).context("Suffix not found")?;

    println!("Answer: {}", suffix);

    Ok(())
}

pub fn p1() -> anyhow::Result<()> {
    answer("00000")
}

pub fn p2() -> anyhow::Result<()> {
    answer("000000")
}
