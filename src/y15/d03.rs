use std::{collections::HashSet, ops::AddAssign};

use anyhow::anyhow;

use super::Vec2;

enum Direction {
    North,
    South,
    East,
    West,
}

impl TryFrom<char> for Direction {
    type Error = anyhow::Error;

    fn try_from(ch: char) -> Result<Self, Self::Error> {
        match ch {
            '^' => Ok(Direction::North),
            '>' => Ok(Direction::East),
            'v' => Ok(Direction::South),
            '<' => Ok(Direction::West),
            _ => Err(anyhow!("Invalid direction: {}", ch)),
        }
    }
}

impl AddAssign<Direction> for Vec2 {
    fn add_assign(&mut self, rhs: Direction) {
        match rhs {
            Direction::North => self.y += 1,
            Direction::South => self.y -= 1,
            Direction::East => self.x += 1,
            Direction::West => self.x -= 1,
        }
    }
}

struct State {
    visited: HashSet<Vec2>,
    current: Vec2,
}

impl State {
    fn new() -> Self {
        let mut visited = HashSet::new();
        visited.insert(Vec2::ORIGIN);
        Self {
            visited,
            current: Vec2::ORIGIN,
        }
    }

    fn translate(&mut self, dir: Direction) {
        self.current += dir;
        self.visited.insert(self.current);
    }
}

#[derive(clap::Subcommand)]
enum Subcommand {
    P1,
    P2,
}

#[derive(clap::Args)]
pub struct Args {
    #[command(subcommand)]
    command: Subcommand,
}

impl Args {
    pub async fn run(&self) -> anyhow::Result<()> {
        match &self.command {
            Subcommand::P1 => {
                let input = tokio::fs::read_to_string("inputs/y15_d03.txt").await?;
                let state = input.chars().map(|ch| Direction::try_from(ch)).try_fold(
                    State::new(),
                    |mut state, dir| {
                        state.translate(dir?);
                        Ok::<_, anyhow::Error>(state)
                    },
                )?;
                println!("Answer: {}", state.visited.len());
                Ok(())
            }
            Subcommand::P2 => todo!(),
        }
    }
}
