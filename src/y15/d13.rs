use std::ops::{Deref, DerefMut};

use anyhow::{Context, Error, Result, anyhow};
use itertools::Itertools;
use rayon::iter::{ParallelBridge, ParallelIterator};
use winnow::Parser;

mod parser {
    use winnow::{
        Parser, Result,
        ascii::{dec_uint, newline},
        combinator::{alt, preceded, separated, seq, terminated},
        token::{any, take_while},
    };

    use crate::y15::ws;

    use super::{Relation, Relations};

    fn name<'a>(input: &mut &'a str) -> Result<&'a str> {
        (
            any.verify(|c: &char| c.is_ascii_uppercase()),
            take_while(1.., |c: char| c.is_ascii_lowercase()),
        )
            .take()
            .parse_next(input)
    }

    fn happiness(input: &mut &str) -> Result<i32> {
        terminated(
            alt((
                preceded(ws("gain"), ws(dec_uint)).map(|n: u16| n.into()),
                preceded(ws("lose"), ws(dec_uint)).map(|n: u16| -(i32::from(n))),
            )),
            (ws("happiness"), ws("units")),
        )
        .parse_next(input)
    }

    fn relation<'a>(input: &mut &'a str) -> Result<Relation<'a>> {
        seq!(
            ws(name),
            _: ws("would"),
            happiness,
            _: (ws("by"), ws("sitting"), ws("next"), ws("to")),
            ws(name),
            _: ws('.'),
        )
        .map(|(a, happiness, b)| Relation(a, happiness, b))
        .parse_next(input)
    }

    pub fn relations<'a>(input: &mut &'a str) -> Result<Relations<'a>> {
        separated(1.., relation, newline)
            .map(Relations)
            .parse_next(input)
    }
}

#[derive(Debug)]
struct Relation<'a>(&'a str, i32, &'a str);

#[derive(Debug)]
struct Relations<'a>(Vec<Relation<'a>>);

impl<'a> Relations<'a> {
    fn score(&self, a: &str, b: &str) -> Option<i32> {
        let a_score = self.one_way_score(a, b)?;
        let b_score = self.one_way_score(b, a)?;
        Some(a_score + b_score)
    }

    fn one_way_score(&self, a: &str, b: &str) -> Option<i32> {
        self.iter().find_map(|Relation(from, score, to)| {
            if *from == a && *to == b {
                Some(*score)
            } else {
                None
            }
        })
    }

    fn people(&self) -> Vec<&'a str> {
        self.iter()
            .flat_map(|Relation(a, _, b)| [*a, *b])
            .sorted()
            .dedup()
            .collect()
    }
}

impl<'a> Deref for Relations<'a> {
    type Target = Vec<Relation<'a>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Relations<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a> TryFrom<&'a str> for Relations<'a> {
    type Error = Error;

    fn try_from(value: &'a str) -> std::result::Result<Self, Self::Error> {
        parser::relations
            .parse(value)
            .map_err(|err| anyhow!("{err}"))
    }
}

struct Arrangement<'r, 'a> {
    relations: &'r Relations<'a>,
    order: Vec<&'a str>,
}

impl Arrangement<'_, '_> {
    fn score(&self) -> Option<i32> {
        let base_score = self
            .order
            .iter()
            .tuple_windows()
            .map(|(a, b)| self.relations.score(a, b))
            .fold_options(0, |acc, score| acc + score)?;
        let wrap_score = self
            .relations
            .score(self.order.last()?, self.order.first()?);
        Some(base_score + wrap_score?)
    }
}

fn answer(relations: Relations) -> Result<()> {
    let people = relations.people();
    let people_len = people.len();

    let max_score = people
        .into_iter()
        .permutations(people_len)
        .par_bridge()
        .map(|order| Arrangement {
            relations: &relations,
            order,
        })
        .filter_map(|arrangement| arrangement.score())
        .max()
        .context("No arrangement found")?;

    println!("Answer: {}", max_score);

    Ok(())
}

pub async fn p1() -> Result<()> {
    let input = tokio::fs::read_to_string("inputs/y15_d13.txt").await?;
    let relations = Relations::try_from(input.as_str())?;
    answer(relations)
}

pub async fn p2() -> Result<()> {
    let input = tokio::fs::read_to_string("inputs/y15_d13.txt").await?;
    let mut relations = Relations::try_from(input.as_str())?;
    let my_relations = relations
        .people()
        .into_iter()
        .flat_map(|person| [Relation("Maciek", 0, person), Relation(person, 0, "Maciek")]);
    relations.extend(my_relations);
    answer(relations)
}
