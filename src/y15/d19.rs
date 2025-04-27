use std::str::FromStr;

use anyhow::{Context, Error, Result, anyhow};
use itertools::Itertools;
use winnow::Parser;

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

        let input = input
            .parse()
            .map_err(|err| anyhow!("Failed to parse input: {}", err))?;

        Ok(input)
    }
}

impl FromStr for Input {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        parser::input.parse(s).map_err(|err| anyhow!("{err}"))
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
    use winnow::{
        Parser, Result,
        ascii::newline,
        combinator::{alt, opt, repeat, separated, separated_pair},
        token::any,
    };

    use crate::y15::ws;

    use super::{Atom, Input};

    fn atom(input: &mut &str) -> Result<Atom> {
        (
            any.verify(|ch: &char| ch.is_ascii_uppercase()),
            opt(any.verify(|ch: &char| ch.is_ascii_lowercase())),
        )
            .take()
            .map(|s: &str| s.to_owned())
            .parse_next(input)
    }

    fn replacement(input: &mut &str) -> Result<(Atom, Vec<Atom>)> {
        separated_pair(alt((atom, 'e'.map(Into::into))), ws("=>"), molecule).parse_next(input)
    }

    fn molecule(input: &mut &str) -> Result<Vec<Atom>> {
        repeat(1.., atom).parse_next(input)
    }

    pub fn input(input: &mut &str) -> Result<Input> {
        separated_pair(
            separated(1.., replacement, newline),
            (newline, newline),
            molecule,
        )
        .map(|(replacements, molecule)| Input {
            replacements,
            molecule,
        })
        .parse_next(input)
    }
}
