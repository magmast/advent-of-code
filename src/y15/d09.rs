use std::{
    collections::HashSet,
    fmt::{Display, Formatter},
};

use anyhow::{Context, Result};
use nom::Finish;
use nom_language::error::VerboseError;

mod parser {
    use nom::{
        AsChar, IResult, Input, Parser,
        bytes::complete::{tag, take_while1},
        character::complete::u32,
        combinator::map,
        error::ParseError,
        sequence::separated_pair,
    };

    use crate::y15::ws;

    use super::Connection;

    fn city<I, E>(input: I) -> IResult<I, I, E>
    where
        I: Input,
        <I as Input>::Item: AsChar,
        E: ParseError<I>,
    {
        take_while1(|i: I::Item| i.as_char().is_ascii_alphabetic()).parse(input)
    }

    pub fn connection<'a, E>(input: &'a str) -> IResult<&'a str, Connection<'a>, E>
    where
        E: ParseError<&'a str>,
    {
        map(
            separated_pair(separated_pair(city, ws(tag("to")), city), ws(tag("=")), u32),
            |((from, to), distance)| Connection::new(from, to, distance),
        )
        .parse(input)
    }
}

#[derive(Debug)]
struct Connection<'a> {
    from: &'a str,
    to: &'a str,
    distance: u32,
}

impl<'a> Connection<'a> {
    fn new(from: &'a str, to: &'a str, distance: u32) -> Self {
        Self { from, to, distance }
    }
}

#[derive(Debug)]
struct World<'a> {
    connections: Vec<Connection<'a>>,
}

impl<'a> FromIterator<Connection<'a>> for World<'a> {
    fn from_iter<T: IntoIterator<Item = Connection<'a>>>(iter: T) -> Self {
        Self {
            connections: iter.into_iter().collect(),
        }
    }
}

impl<'a> World<'a> {
    fn distance(&self, from: &'a str, to: &'a str) -> Option<u32> {
        self.connections.iter().find_map(|c| {
            if (c.from == from && c.to == to) || (c.from == to && c.to == from) {
                Some(c.distance)
            } else {
                None
            }
        })
    }

    fn cities(&self) -> HashSet<&'a str> {
        self.connections
            .iter()
            .flat_map(|c| [c.from, c.to])
            .collect()
    }
}

#[derive(Clone, Debug)]
struct Route<'a, 'w> {
    world: &'w World<'a>,
    initial: &'a str,
    stops: Vec<(u32, &'a str)>,
}

impl Display for Route<'_, '_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.initial)?;
        for (distance, stop) in &self.stops {
            write!(f, " -({})> {}", distance, stop)?;
        }
        write!(f, " = {}", self.distance())
    }
}

impl<'a, 'w> Route<'a, 'w> {
    fn new(world: &'w World<'a>, initial: &'a str) -> Self {
        Self {
            world,
            initial,
            stops: vec![],
        }
    }

    fn last_stop(&self) -> &'a str {
        self.stops
            .last()
            .map(|(_, stop)| stop)
            .unwrap_or(&self.initial)
    }

    fn distance(&self) -> u32 {
        self.stops.iter().map(|(d, _)| *d).sum()
    }

    fn add_stop(&mut self, to: &'a str) {
        let last_stop = self.last_stop();
        let distance = self
            .world
            .distance(last_stop, to)
            .unwrap_or_else(|| panic!("No direct connection between {} and {}", last_stop, to));
        self.stops.push((distance, to));
    }

    fn unvisited_cities(&self) -> HashSet<&'a str> {
        let mut visited: HashSet<_> = self.stops.iter().map(|(_, stop)| *stop).collect();
        visited.insert(self.initial);

        self.world.cities().difference(&visited).cloned().collect()
    }
}

async fn answer(f: impl FnOnce(Vec<Route>) -> Option<u32>) -> Result<()> {
    let input = tokio::fs::read_to_string("inputs/y15_d09.txt").await?;

    let input = input.lines();
    let world: World = input
        .enumerate()
        .map(|(index, line)| {
            parser::connection::<VerboseError<_>>(line)
                .finish()
                .map_err(VerboseError::<String>::from)
                .context(format!("Failed to parse line {}", index))
                .map(|(_, conn)| conn)
        })
        .try_collect()?;

    let mut queue: Vec<_> = world
        .cities()
        .into_iter()
        .map(|city| Route::new(&world, city))
        .collect();

    let mut done = vec![];

    while let Some(route) = queue.pop() {
        let unvisited_cities = route.unvisited_cities();
        for city in unvisited_cities.into_iter() {
            let mut route = route.clone();
            route.add_stop(city);
            if route.unvisited_cities().is_empty() {
                done.push(route);
            } else {
                queue.push(route);
            }
        }
    }

    let answer = f(done).context("Valid route not found")?;
    println!("Answer: {}", answer);

    Ok(())
}

pub async fn p1() -> Result<()> {
    answer(|routes| routes.into_iter().map(|route| route.distance()).min()).await
}

pub async fn p2() -> Result<()> {
    answer(|routes| routes.into_iter().map(|route| route.distance()).max()).await
}
