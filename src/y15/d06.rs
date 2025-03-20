use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

use anyhow::anyhow;
use futures::{TryStreamExt, future};
use nom::Parser;
use tokio::{
    fs::File,
    io::{AsyncBufReadExt, BufReader},
};
use tokio_stream::wrappers::LinesStream;

use super::{PointRangeInclusive, Vec2};

mod parser {
    use nom::{
        Parser,
        branch::alt,
        bytes::complete::tag,
        character::complete::{digit1, space1},
        combinator::{map, value},
        error::Error,
        sequence::separated_pair,
    };

    use crate::y15::{PointRangeInclusive, Vec2};

    use super::{Action, Instruction};

    fn action<'a>() -> impl Parser<&'a [u8], Output = Action, Error = Error<&'a [u8]>> {
        alt((
            value(Action::TurnOn, tag("turn on")),
            value(Action::TurnOff, tag("turn off")),
            value(Action::Toggle, tag("toggle")),
        ))
    }

    fn range<'a>()
    -> impl Parser<&'a [u8], Output = PointRangeInclusive<usize>, Error = Error<&'a [u8]>> {
        fn coord<'a>() -> impl Parser<&'a [u8], Output = Vec2<usize>, Error = Error<&'a [u8]>> {
            map(separated_pair(digit1, tag(","), digit1), |(a, b)| {
                Vec2::new(
                    std::str::from_utf8(a).unwrap().parse().unwrap(),
                    std::str::from_utf8(b).unwrap().parse().unwrap(),
                )
            })
        }

        map(
            separated_pair(coord(), (space1, tag("through"), space1), coord()),
            |(begin, end)| begin.points_to_inclusive(end),
        )
    }

    pub fn instruction<'a>() -> impl Parser<&'a [u8], Output = Instruction, Error = Error<&'a [u8]>>
    {
        map(
            separated_pair(action(), space1, range()),
            |(action, range)| Instruction { action, range },
        )
    }
}

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

trait Grid {
    fn exec(&mut self, instruction: &Instruction);
    fn result(&self) -> impl Display;
}

async fn answer<G: Grid + Default>() -> anyhow::Result<()> {
    let input = File::open("inputs/y15_d06.txt").await?;
    let input = BufReader::new(input);
    let grid = LinesStream::new(input.lines())
        .map_err(anyhow::Error::from)
        .and_then(async |line| {
            let (_, instruction) = parser::instruction()
                .parse(line.as_ref())
                .map_err(|_| anyhow!("Line parsing failed"))?;
            Ok(instruction)
        })
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
