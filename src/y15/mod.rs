pub mod d01;
pub mod d02;

#[derive(clap::Subcommand)]
enum Subcommand {
    D01(d01::Args),
    D02(d02::Args),
}

#[derive(clap::Args)]
pub struct Args {
    #[command(subcommand)]
    command: Subcommand,
}

impl Args {
    pub async fn run(&self) -> anyhow::Result<()> {
        match &self.command {
            Subcommand::D01(args) => args.run().await,
            Subcommand::D02(args) => args.run().await,
        }
    }
}
