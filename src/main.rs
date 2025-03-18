use advent_of_code::Cli;
use clap::Parser as _;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Cli::parse().run().await
}
