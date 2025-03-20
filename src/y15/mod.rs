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
enum DaySubcommand {
    P1,
    P2,
}

#[derive(clap::Args)]
struct DayArgs {
    #[command(subcommand)]
    command: DaySubcommand,
}

#[derive(clap::Subcommand)]
enum Subcommand {
    D01(DayArgs),
    D02(DayArgs),
    D03(DayArgs),
    D04(DayArgs),
    D05(DayArgs),
}

#[derive(clap::Args)]
pub struct Args {
    #[command(subcommand)]
    command: Subcommand,
}

impl Args {
    pub async fn run(&self) -> anyhow::Result<()> {
        match &self.command {
            Subcommand::D01(args) => match args.command {
                DaySubcommand::P1 => d01::p1().await,
                DaySubcommand::P2 => d01::p2().await,
            },
            Subcommand::D02(args) => match args.command {
                DaySubcommand::P1 => d02::p1().await,
                DaySubcommand::P2 => d02::p2().await,
            },
            Subcommand::D03(args) => match args.command {
                DaySubcommand::P1 => d03::p1().await,
                DaySubcommand::P2 => d03::p2().await,
            },
            Subcommand::D04(args) => match args.command {
                DaySubcommand::P1 => d04::p1(),
                DaySubcommand::P2 => d04::p2(),
            },
            Subcommand::D05(args) => match args.command {
                DaySubcommand::P1 => d05::p1().await,
                DaySubcommand::P2 => d05::p2().await,
            },
        }
    }
}
