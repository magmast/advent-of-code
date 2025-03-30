use anyhow::{Context, Result};
use itertools::Itertools;
use nom::Finish;
use nom_language::error::VerboseError;

type Molecule = Vec<Atom>;

type Atom = String;

#[derive(Debug)]
struct Input {
    replacements: Vec<(Atom, Molecule)>,
    molecule: Molecule,
}

impl Input {
    pub async fn read() -> Result<Self> {
        let input = tokio::fs::read_to_string("inputs/y15_d19.txt")
            .await
            .context("Failed to read input file")?;

        let (_, input) = parser::input::<VerboseError<_>>(&input)
            .finish()
            .map_err(VerboseError::<String>::from)
            .context("Failed to parse input")?;

        Ok(input)
    }
}

pub async fn p1() -> Result<()> {
    let input = Input::read().await?;

    let replacements = expand_replacements(&input.molecule, &input.replacements)
        .unique()
        .count();

    println!("Answer: {}", replacements);

    Ok(())
}

fn expand_replacements(
    molecule: &[Atom],
    replacements: &[(Atom, Vec<Atom>)],
) -> impl Iterator<Item = Vec<Atom>> {
    replacements
        .iter()
        .flat_map(|(from, to)| expand_replacement(molecule, from, to))
}

fn expand_replacement(
    molecule: &[Atom],
    from: &Atom,
    to: &[Atom],
) -> impl Iterator<Item = Vec<Atom>> {
    (0..molecule.len())
        .filter(move |&i| &molecule[i] == from)
        .map(move |i| {
            let mut new_molecule = molecule.to_vec();
            new_molecule.splice(i..i + 1, to.iter().cloned());
            new_molecule
        })
}

pub async fn p2() -> Result<()> {
    let input = Input::read().await?;
    let steps = count_steps_to_e(&input.molecule);
    println!("Answer: {}", steps);
    Ok(())
}

/// Counts the steps required using the known puzzle property:
/// steps = (#elements) - (#Rn + #Ar) - 2*(#Y) - 1
fn count_steps_to_e(molecule: &Molecule) -> usize {
    let rn_count = molecule.iter().filter(|e| *e == "Rn").count();
    let ar_count = molecule.iter().filter(|e| *e == "Ar").count();
    let y_count = molecule.iter().filter(|e| *e == "Y").count();
    molecule.len() - rn_count - ar_count - 2 * y_count - 1
}

mod parser {
    use nom::{
        IResult, Parser,
        branch::alt,
        bytes::complete::tag,
        character::complete::{char, newline, satisfy},
        combinator::{map, opt, recognize},
        error::ParseError,
        multi::{many1, separated_list1},
        sequence::separated_pair,
    };

    use crate::y15::ws;

    use super::{Atom, Input};

    fn atom<'a, E>(input: &'a str) -> IResult<&'a str, Atom, E>
    where
        E: ParseError<&'a str>,
    {
        map(
            recognize((
                satisfy(|ch| ch.is_ascii_uppercase()),
                opt(satisfy(|ch| ch.is_ascii_lowercase())),
            )),
            |s: &'a str| s.to_owned(),
        )
        .parse(input)
    }

    fn replacement<'a, E>(input: &'a str) -> IResult<&'a str, (Atom, Vec<Atom>), E>
    where
        E: ParseError<&'a str>,
    {
        separated_pair(
            alt((atom, map(char('e'), Into::into))),
            ws(tag("=>")),
            many1(atom),
        )
        .parse(input)
    }

    fn molecule<'a, E>(input: &'a str) -> IResult<&'a str, Vec<Atom>, E>
    where
        E: ParseError<&'a str>,
    {
        many1(atom).parse(input)
    }

    pub fn input<'a, E>(input: &'a str) -> IResult<&'a str, Input, E>
    where
        E: ParseError<&'a str>,
    {
        map(
            separated_pair(
                separated_list1(newline, replacement),
                (newline, newline),
                molecule,
            ),
            |(replacements, molecule)| Input {
                replacements,
                molecule,
            },
        )
        .parse(input)
    }
}
