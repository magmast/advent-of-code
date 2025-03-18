pub mod y15;

use clap::Parser;

#[derive(Parser)]
#[command(name = "aoc")]
pub enum Cli {
    Y15(y15::Args),
}

impl Cli {
    pub async fn run(&self) -> anyhow::Result<()> {
        match self {
            Cli::Y15(args) => args.run().await,
        }
    }
}
