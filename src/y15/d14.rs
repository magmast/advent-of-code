use std::collections::HashMap;

use anyhow::{Context, Result, anyhow};
use itertools::Itertools;
use winnow::Parser;

const RUN_DURATION: Seconds = 2503;

mod parser {
    use winnow::{
        Parser, Result,
        ascii::{alpha1, dec_uint, newline},
        combinator::{delimited, separated},
    };

    use crate::y15::ws;

    use super::Reindeer;

    fn name<'a>(input: &mut &'a str) -> Result<&'a str> {
        alpha1(input)
    }

    fn reindeer(input: &mut &str) -> Result<Reindeer> {
        (
            delimited((ws(name), ws("can"), ws("fly")), ws(dec_uint), ws("km/s")),
            delimited(ws("for"), ws(dec_uint), ws("seconds")),
            delimited(
                (
                    ws(','),
                    ws("but"),
                    ws("then"),
                    ws("must"),
                    ws("rest"),
                    ws("for"),
                ),
                ws(dec_uint),
                (ws("seconds"), ws('.')),
            ),
        )
            .map(|(speed, move_dur, rest_dur)| Reindeer {
                speed,
                r#move: move_dur,
                rest: rest_dur,
            })
            .parse_next(input)
    }

    pub fn reindeers(input: &mut &str) -> Result<Vec<Reindeer>> {
        separated(1.., reindeer, newline).parse_next(input)
    }
}

type KmPerSecond = u64;

type Seconds = u64;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
struct Reindeer {
    speed: KmPerSecond,
    r#move: Seconds,
    rest: Seconds,
}

impl Reindeer {
    fn distance(&self, after: Seconds) -> u64 {
        let cycle = self.r#move + self.rest;
        let cycles = after / cycle;
        let remaining = (after % cycle).min(self.r#move);
        (cycles * self.speed * self.r#move) + (self.speed * remaining)
    }
}

async fn read_reindeers() -> Result<Vec<Reindeer>> {
    let input = tokio::fs::read_to_string("inputs/y15_d14.txt").await?;
    parser::reindeers
        .parse(input.as_str())
        .map_err(|err| anyhow!("{err}"))
        .context("Failed to parse reindeers")
}

pub async fn p1() -> Result<()> {
    let max_distance = read_reindeers()
        .await?
        .into_iter()
        .map(|r| r.distance(RUN_DURATION))
        .max()
        .context("No reindeers?")?;
    println!("Answer: {}", max_distance);
    Ok(())
}

pub async fn p2() -> Result<()> {
    let reindeers: HashMap<_, u64> = read_reindeers()
        .await?
        .into_iter()
        .map(|reindeer| (reindeer, 0))
        .collect();
    let max_score = (1..RUN_DURATION)
        .fold(reindeers, |mut scoreboard, passed| {
            let leaders = scoreboard
                .keys()
                .copied()
                .max_set_by(|a, b| a.distance(passed).cmp(&b.distance(passed)));
            for leader in leaders {
                if let Some(score) = scoreboard.get_mut(&leader) {
                    *score += 1;
                }
            }
            scoreboard
        })
        .into_values()
        .max()
        .context("No reindeers?")?;
    println!("Answer: {}", max_score);
    Ok(())
}
