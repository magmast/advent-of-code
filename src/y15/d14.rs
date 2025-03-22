use std::collections::HashMap;

use anyhow::{Context, Result};
use itertools::Itertools;
use nom::Finish;
use nom_language::error::VerboseError;

const RUN_DURATION: Seconds = 2503;

mod parser {
    use nom::{
        AsChar, Compare, IResult, Input, Parser,
        bytes::complete::tag,
        character::complete::{alpha1, char, newline, u64},
        combinator::map,
        error::ParseError,
        multi::separated_list1,
        sequence::delimited,
    };

    use crate::y15::ws;

    use super::Reindeer;

    fn name<I, E>(input: I) -> IResult<I, I, E>
    where
        I: Input,
        I::Item: AsChar,
        E: ParseError<I>,
    {
        alpha1(input)
    }

    fn reindeer<I, E>(input: I) -> IResult<I, Reindeer, E>
    where
        I: Input + for<'a> Compare<&'a str>,
        I::Item: AsChar,
        E: ParseError<I>,
    {
        map(
            (
                delimited(
                    (ws(name), ws(tag("can")), ws(tag("fly"))),
                    ws(u64),
                    ws(tag("km/s")),
                ),
                delimited(ws(tag("for")), ws(u64), ws(tag("seconds"))),
                delimited(
                    (
                        ws(char(',')),
                        ws(tag("but")),
                        ws(tag("then")),
                        ws(tag("must")),
                        ws(tag("rest")),
                        ws(tag("for")),
                    ),
                    ws(u64),
                    (ws(tag("seconds")), ws(char('.'))),
                ),
            ),
            |(speed, move_dur, rest_dur)| Reindeer {
                speed,
                r#move: move_dur,
                rest: rest_dur,
            },
        )
        .parse(input)
    }

    pub fn reindeers<I, E>(input: I) -> IResult<I, Vec<Reindeer>, E>
    where
        I: Input + for<'a> Compare<&'a str>,
        I::Item: AsChar,
        E: ParseError<I>,
    {
        separated_list1(newline, reindeer).parse(input)
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
    parser::reindeers::<_, VerboseError<_>>(input.as_str())
        .finish()
        .map(|(_, v)| v)
        .map_err(VerboseError::<String>::from)
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
