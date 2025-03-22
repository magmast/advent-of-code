use std::ops::{Deref, DerefMut};

use anyhow::{Context, Result};
use itertools::Itertools;
use nom::Finish;
use nom_language::error::VerboseError;
use rayon::iter::{ParallelBridge, ParallelIterator};

mod parser {
    use nom::{
        IResult, Parser,
        branch::alt,
        bytes::complete::{tag, take_while1},
        character::complete::{char, newline, satisfy, u32},
        combinator::{map, recognize},
        error::ParseError,
        multi::separated_list1,
        sequence::{preceded, terminated},
    };

    use crate::y15::ws;

    use super::{Relation, Relations};

    fn name<'a, E>(input: &'a str) -> IResult<&'a str, &'a str, E>
    where
        E: ParseError<&'a str>,
    {
        recognize((
            satisfy(|c| c.is_ascii_uppercase()),
            take_while1(|c: char| c.is_ascii_lowercase()),
        ))
        .parse(input)
    }

    fn happiness<'a, E>(input: &'a str) -> IResult<&'a str, i32, E>
    where
        E: ParseError<&'a str>,
    {
        terminated(
            alt((
                map(preceded(ws(tag("gain")), ws(u32)), |n| n as i32),
                map(preceded(ws(tag("lose")), ws(u32)), |n| -(n as i32)),
            )),
            (ws(tag("happiness")), ws(tag("units"))),
        )
        .parse(input)
    }

    fn relation<'a, E>(input: &'a str) -> IResult<&'a str, Relation<'a>, E>
    where
        E: ParseError<&'a str>,
    {
        map(
            (
                ws(name),
                ws(tag("would")),
                happiness,
                ws(tag("by")),
                ws(tag("sitting")),
                ws(tag("next")),
                ws(tag("to")),
                ws(name),
                ws(char('.')),
            ),
            |(a, _, happiness, _, _, _, _, b, _)| Relation(a, happiness, b),
        )
        .parse(input)
    }

    pub fn relations<'a, E>(input: &'a str) -> IResult<&'a str, Relations<'a>, E>
    where
        E: ParseError<&'a str>,
    {
        map(separated_list1(newline, relation), |relations| {
            Relations(relations)
        })
        .parse(input)
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

async fn read_relations(input: &str) -> Result<Relations> {
    parser::relations::<VerboseError<_>>(input)
        .finish()
        .map(|(_, v)| v)
        .map_err(VerboseError::<String>::from)
        .map_err(anyhow::Error::from)
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
    let relations = read_relations(&input).await?;
    answer(relations)
}

pub async fn p2() -> Result<()> {
    let input = tokio::fs::read_to_string("inputs/y15_d13.txt").await?;
    let mut relations = read_relations(&input).await?;
    let my_relations = relations
        .people()
        .into_iter()
        .flat_map(|person| [Relation("Maciek", 0, person), Relation(person, 0, "Maciek")]);
    relations.extend(my_relations);
    answer(relations)
}
