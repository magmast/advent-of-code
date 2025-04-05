use anyhow::{Context, Result};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

const INPUT: u32 = 29_000_000;

pub async fn p1() -> Result<()> {
    let answer = (1..INPUT)
        .into_par_iter()
        .find_first(|&house_index| inf_presents_count(house_index) >= INPUT)
        .context("Failed to find the house number")?;
    println!("Answer: {}", answer);
    Ok(())
}

fn inf_presents_count(house_index: u32) -> u32 {
    divisors(house_index).map(|d| d * 10).sum()
}

pub async fn p2() -> Result<()> {
    let answer = (1..INPUT)
        .into_par_iter()
        .find_first(|&house_index| fin_presents_count(house_index) >= INPUT)
        .context("Failed to find the house number")?;
    println!("Answer: {}", answer);
    Ok(())
}

fn fin_presents_count(house_index: u32) -> u32 {
    divisors(house_index)
        .filter(|&elf| house_index <= elf * 50)
        .map(|d| d * 11)
        .sum()
}

/// Calculates all divisors of a number.
fn divisors(n: u32) -> impl Iterator<Item = u32> {
    assert!(n > 0, "n must be greater than 0");

    let sqrt_n = (n as f64).sqrt() as u32;

    (1..=sqrt_n)
        .filter(move |&i| n % i == 0)
        .flat_map(move |i| if i * i == n { vec![i] } else { vec![i, n / i] })
}
