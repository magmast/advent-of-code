use std::{collections::HashMap, ops::Mul};

use anyhow::{Context, Result};
use nom::Finish;
use nom_language::error::VerboseError;

mod parser {
    use nom::{
        AsChar, Compare, IResult, Input, Offset, Parser,
        bytes::complete::{tag, take_while1},
        character::complete::{char, i32, newline, satisfy},
        combinator::{map, recognize},
        error::ParseError,
        multi::separated_list1,
        sequence::preceded,
    };

    use crate::y15::ws;

    use super::Ingredient;

    fn name<I, E>(input: I) -> IResult<I, I, E>
    where
        I: Input + Offset,
        I::Item: AsChar,
        E: ParseError<I>,
    {
        recognize((
            satisfy(|i| i.as_char().is_ascii_uppercase()),
            take_while1(|i: I::Item| i.as_char().is_ascii_alphabetic()),
        ))
        .parse(input)
    }

    fn property<I, E>(name: &'static str) -> impl Parser<I, Output = i32, Error = E>
    where
        I: Input + for<'a> Compare<&'a str> + for<'a> Compare<&'a [u8]>,
        I::Item: AsChar,
        E: ParseError<I>,
    {
        preceded(ws(tag(name)), ws(i32))
    }

    fn ingredient<I, E>(input: I) -> IResult<I, Ingredient, E>
    where
        I: Input + Offset + for<'a> Compare<&'a str> + for<'a> Compare<&'a [u8]>,
        I::Item: AsChar,
        E: ParseError<I>,
    {
        map(
            (
                preceded((ws(name), ws(char(':'))), property("capacity")),
                preceded(ws(char(',')), property("durability")),
                preceded(ws(char(',')), property("flavor")),
                preceded(ws(char(',')), property("texture")),
                preceded(ws(char(',')), property("calories")),
            ),
            |(capacity, durability, flavor, texture, calories)| Ingredient {
                capacity,
                durability,
                flavor,
                texture,
                calories,
            },
        )
        .parse(input)
    }

    pub fn ingredients<I, E>(input: I) -> IResult<I, Vec<Ingredient>, E>
    where
        I: Input + Offset + for<'a> Compare<&'a str> + for<'a> Compare<&'a [u8]>,
        I::Item: AsChar,
        E: ParseError<I>,
    {
        separated_list1(newline, ingredient).parse(input)
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
struct Ingredient {
    capacity: i32,
    durability: i32,
    flavor: i32,
    texture: i32,
    calories: i32,
}

impl Mul<u32> for &Ingredient {
    type Output = Ingredient;

    fn mul(self, rhs: u32) -> Self::Output {
        Ingredient {
            capacity: self.capacity * rhs as i32,
            durability: self.durability * rhs as i32,
            flavor: self.flavor * rhs as i32,
            texture: self.texture * rhs as i32,
            calories: self.calories * rhs as i32,
        }
    }
}

async fn read_ingredients() -> Result<Vec<Ingredient>> {
    let input = tokio::fs::read_to_string("inputs/y15_d15.txt").await?;
    let ingredients = parser::ingredients::<_, VerboseError<_>>(input.as_str())
        .finish()
        .map(|(_, ingredients)| ingredients)
        .map_err(VerboseError::<String>::from)?;
    Ok(ingredients)
}

#[derive(Debug)]
struct Cookie<'a> {
    ingredients: HashMap<&'a Ingredient, u32>,
}

impl Cookie<'_> {
    fn score(&self) -> i32 {
        let total_capacity = self
            .ingredients
            .iter()
            .map(|(ingredient, &count)| ingredient.capacity * count as i32)
            .sum::<i32>()
            .max(0);

        let total_durability = self
            .ingredients
            .iter()
            .map(|(ingredient, &count)| ingredient.durability * count as i32)
            .sum::<i32>()
            .max(0);

        let total_flavor = self
            .ingredients
            .iter()
            .map(|(ingredient, &count)| ingredient.flavor * count as i32)
            .sum::<i32>()
            .max(0);

        let total_texture = self
            .ingredients
            .iter()
            .map(|(ingredient, &count)| ingredient.texture * count as i32)
            .sum::<i32>()
            .max(0);

        total_capacity * total_durability * total_flavor * total_texture
    }
}

impl<'a> Cookie<'a> {
    pub fn all_from(ingredients: &'a [Ingredient]) -> Vec<Cookie<'a>> {
        let n = ingredients.len();
        let mut result = Vec::new();
        let mut distribution = vec![0u32; n];
        Self::distribute(ingredients, &mut distribution, 0, 100, &mut result);
        result
    }

    fn distribute(
        ingredients: &'a [Ingredient],
        distribution: &mut [u32],
        ingredient_index: usize,
        spoons_left: u32,
        cookies: &mut Vec<Cookie<'a>>,
    ) {
        if ingredient_index == ingredients.len() - 1 {
            distribution[ingredient_index] = spoons_left;
            let map = ingredients
                .iter()
                .zip(distribution.iter())
                .map(|(ing, &count)| (ing, count))
                .collect();
            cookies.push(Cookie { ingredients: map });
        } else {
            for amt in 0..=spoons_left {
                distribution[ingredient_index] = amt;
                Self::distribute(
                    ingredients,
                    distribution,
                    ingredient_index + 1,
                    spoons_left - amt,
                    cookies,
                );
            }
        }
    }

    fn calories(&self) -> i32 {
        self.ingredients
            .iter()
            .map(|(ingredient, &count)| ingredient.calories * count as i32)
            .sum::<i32>()
    }
}

impl<'a> From<Vec<&'a Ingredient>> for Cookie<'a> {
    fn from(ingredients: Vec<&'a Ingredient>) -> Self {
        assert_eq!(ingredients.len(), 100);
        let mut map = HashMap::new();
        for ingredient in ingredients {
            *map.entry(ingredient).or_insert(0) += 1;
        }
        Self { ingredients: map }
    }
}

pub async fn p1() -> Result<()> {
    let ingredients = read_ingredients().await?;
    let best_score = Cookie::all_from(&ingredients)
        .into_iter()
        .map(|cookie| cookie.score())
        .max()
        .context("No cookie found?")?;
    println!("{:?}", best_score);
    Ok(())
}

pub async fn p2() -> Result<()> {
    let ingredients = read_ingredients().await?;
    let best_score = Cookie::all_from(&ingredients)
        .into_iter()
        .filter(|cookie| cookie.calories() == 500)
        .map(|cookie| cookie.score())
        .max()
        .context("No cookie found?")?;
    println!("{:?}", best_score);
    Ok(())
}
