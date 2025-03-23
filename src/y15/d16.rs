use std::collections::HashMap;

use anyhow::{Context, Result};
use nom::Finish;
use nom_language::error::VerboseError;

mod parser {
    use nom::{
        IResult, Parser,
        bytes::complete::tag,
        character::complete::{alpha1, char, newline, u32},
        combinator::map,
        error::ParseError,
        multi::separated_list1,
        sequence::{delimited, separated_pair},
    };

    use super::Aunt;
    use crate::y15::ws;

    fn property_name<'a, E>(input: &'a str) -> IResult<&'a str, &'a str, E>
    where
        E: ParseError<&'a str>,
    {
        alpha1(input)
    }

    fn property<'a, E>(input: &'a str) -> IResult<&'a str, (&'a str, u32), E>
    where
        E: ParseError<&'a str>,
    {
        separated_pair(ws(property_name), ws(char(':')), ws(u32)).parse(input)
    }

    fn aunt<'a, E>(input: &'a str) -> IResult<&'a str, Aunt<'a>, E>
    where
        E: ParseError<&'a str>,
    {
        map(
            (
                delimited(ws(tag("Sue")), u32, ws(char(':'))),
                separated_list1(ws(char(',')), property),
            ),
            |(num, properties)| Aunt {
                num,
                properties: properties.into_iter().collect(),
            },
        )
        .parse(input)
    }

    pub fn aunts<'a, E>(input: &'a str) -> IResult<&'a str, Vec<Aunt<'a>>, E>
    where
        E: ParseError<&'a str>,
    {
        separated_list1(newline, aunt).parse(input)
    }
}

#[derive(Debug)]
struct Aunt<'a> {
    num: u32,
    properties: HashMap<&'a str, u32>,
}

enum Condition {
    Equal(u32),
    Greater(u32),
    Less(u32),
}

impl Condition {
    fn satisfies(&self, value: u32) -> bool {
        match self {
            Self::Equal(v) => *v == value,
            Self::Greater(v) => *v < value,
            Self::Less(v) => *v > value,
        }
    }
}

/// Helper function which reads and parses the input and returns the number of the best aunt
async fn find_best_aunt<F>(score_fn: F) -> Result<u32>
where
    F: Fn(&Aunt) -> usize,
{
    let input = tokio::fs::read_to_string("inputs/y15_d16.txt").await?;
    let aunts = parser::aunts::<VerboseError<_>>(&input)
        .finish()
        .map(|(_, aunts)| aunts)
        .map_err(VerboseError::<String>::from)?;
    let best_aunt = aunts
        .into_iter()
        .max_by(|a, b| score_fn(a).cmp(&score_fn(b)))
        .context("Aunts doesn't exist!")?;
    Ok(best_aunt.num)
}

pub async fn p1() -> Result<()> {
    let required_props: HashMap<&str, u32> = vec![
        ("children", 3),
        ("cats", 7),
        ("samoyeds", 2),
        ("pomeranians", 3),
        ("akitas", 0),
        ("vizslas", 0),
        ("goldfish", 5),
        ("trees", 3),
        ("cars", 2),
        ("perfumes", 1),
    ]
    .into_iter()
    .collect();

    let score_aunt = |aunt: &Aunt| {
        aunt.properties
            .iter()
            .filter(|(k, v)| required_props.get(*k).is_some_and(|&req| req == **v))
            .count()
    };

    println!("Answer: {}", find_best_aunt(score_aunt).await?);
    Ok(())
}

pub async fn p2() -> Result<()> {
    let required_props: HashMap<&str, Condition> = vec![
        ("children", Condition::Equal(3)),
        ("cats", Condition::Greater(7)),
        ("samoyeds", Condition::Equal(2)),
        ("pomeranians", Condition::Less(3)),
        ("akitas", Condition::Equal(0)),
        ("vizslas", Condition::Equal(0)),
        ("goldfish", Condition::Less(5)),
        ("trees", Condition::Greater(3)),
        ("cars", Condition::Equal(2)),
        ("perfumes", Condition::Equal(1)),
    ]
    .into_iter()
    .collect();

    let score_aunt = |aunt: &Aunt| {
        aunt.properties
            .iter()
            .filter(|(k, v)| {
                required_props
                    .get(*k)
                    .is_some_and(|cond| cond.satisfies(**v))
            })
            .count()
    };

    println!("Answer: {}", find_best_aunt(score_aunt).await?);
    Ok(())
}
