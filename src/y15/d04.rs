use anyhow::Context;
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};

const INPUT: &str = "iwrupvqb";

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
        let suffix = match &self.subcommand {
            Subcommand::P1 => Self::find_suffix_for_md5_prefix("00000"),
            Subcommand::P2 => Self::find_suffix_for_md5_prefix("000000"),
        };

        println!("Answer: {}", suffix.context("Suffix not found")?);

        Ok(())
    }

    pub fn find_suffix_for_md5_prefix(prefix: &str) -> Option<u32> {
        (0..u32::MAX)
            .into_par_iter()
            .by_exponential_blocks()
            .find_first(|i| {
                format!("{:x}", md5::compute(format!("{}{}", INPUT, i))).starts_with(prefix)
            })
    }
}
