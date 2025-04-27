use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    str::FromStr,
};

use anyhow::{Error, anyhow};
use futures::{TryStreamExt, future};
use tokio::{
    fs::File,
    io::{AsyncBufReadExt, BufReader},
};
use tokio_stream::wrappers::LinesStream;
use winnow::Parser;

use super::{PointRangeInclusive, Vec2};

#[derive(Debug, Clone)]
enum Action {
    TurnOn,
    TurnOff,
    Toggle,
}

#[derive(Debug)]
struct Instruction {
    action: Action,
    range: PointRangeInclusive<usize>,
}

impl FromStr for Instruction {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parser::instruction.parse(s).map_err(|err| anyhow!("{err}"))
    }
}

trait Grid {
    fn exec(&mut self, instruction: &Instruction);
    fn result(&self) -> impl Display;
}

async fn answer<G: Grid + Default>() -> anyhow::Result<()> {
    let input = File::open("inputs/y15_d06.txt").await?;
    let input = BufReader::new(input);
    let grid = LinesStream::new(input.lines())
        .map_err(anyhow::Error::from)
        .and_then(|line| future::ready(line.parse()))
        .try_fold(G::default(), |mut grid, instruction: Instruction| {
            grid.exec(&instruction);
            future::ok(grid)
        })
        .await?;
    println!("Answer: {}", grid.result());
    Ok(())
}

#[derive(Default)]
struct OnOffGrid {
    lit: HashSet<Vec2<usize>>,
}

impl Grid for OnOffGrid {
    fn exec(&mut self, instruction: &Instruction) {
        for x in instruction.range {
            match instruction.action {
                Action::TurnOn => {
                    self.lit.insert(x);
                }
                Action::TurnOff => {
                    self.lit.remove(&x);
                }
                Action::Toggle => {
                    if self.lit.contains(&x) {
                        self.lit.remove(&x);
                    } else {
                        self.lit.insert(x);
                    }
                }
            }
        }
    }

    fn result(&self) -> impl Display {
        self.lit.len()
    }
}

pub async fn p1() -> anyhow::Result<()> {
    answer::<OnOffGrid>().await
}

#[derive(Default)]
struct BrightnessGrid {
    brightness: HashMap<Vec2<usize>, i32>,
}

impl Grid for BrightnessGrid {
    fn exec(&mut self, instruction: &Instruction) {
        for x in instruction.range {
            let entry = self.brightness.entry(x).or_insert(0);
            match instruction.action {
                Action::TurnOn => {
                    *entry += 1;
                }
                Action::TurnOff => {
                    *entry = (*entry - 1).max(0);
                }
                Action::Toggle => {
                    *entry += 2;
                }
            }
        }
    }

    fn result(&self) -> impl Display {
        self.brightness.values().copied().sum::<i32>()
    }
}

pub async fn p2() -> anyhow::Result<()> {
    answer::<BrightnessGrid>().await
}

mod parser {
    use winnow::{
        Parser, Result,
        ascii::{digit1, multispace1},
        combinator::{alt, separated_pair},
    };

    use crate::y15::{PointRangeInclusive, Vec2, ws};

    use super::{Action, Instruction};

    fn action(input: &mut &str) -> Result<Action> {
        alt((
            "turn on".value(Action::TurnOn),
            "turn off".value(Action::TurnOff),
            "toggle".value(Action::Toggle),
        ))
        .parse_next(input)
    }

    fn range(input: &mut &str) -> Result<PointRangeInclusive<usize>> {
        fn coord(input: &mut &str) -> Result<Vec2<usize>> {
            separated_pair(digit1, ",", digit1)
                .map(|(a, b): (&str, &str)| Vec2::new(a.parse().unwrap(), b.parse().unwrap()))
                .parse_next(input)
        }

        separated_pair(coord, ws("through"), coord)
            .map(|(begin, end)| begin.points_to_inclusive(end))
            .parse_next(input)
    }

    pub fn instruction(input: &mut &str) -> Result<Instruction> {
        separated_pair(action, multispace1, range)
            .map(|(action, range)| Instruction { action, range })
            .parse_next(input)
    }
}
