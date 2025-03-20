pub mod d01;
pub mod d02;
pub mod d03;
pub mod d04;
pub mod d05;

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub struct Vec2 {
    x: i32,
    y: i32,
}

impl Vec2 {
    const ORIGIN: Self = Self { x: 0, y: 0 };

    pub fn area(&self) -> i32 {
        self.x * self.y
    }

    pub fn perimeter(&self) -> i32 {
        2 * (self.x + self.y)
    }
}

#[derive(clap::Subcommand)]
enum Subcommand {
    D01(d01::Args),
    D02(d02::Args),
    D03(d03::Args),
    D04(d04::Args),
    D05(d05::Args),
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
            Subcommand::D03(args) => args.run().await,
            Subcommand::D04(args) => args.run().await,
            Subcommand::D05(args) => args.run().await,
        }
    }
}
