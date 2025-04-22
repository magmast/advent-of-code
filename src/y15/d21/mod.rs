use std::iter;

use anyhow::{Context, Result, anyhow};
use itertools::Itertools;

mod parser {
    use winnow::{
        Parser,
        ascii::{alpha1, dec_uint, digit1, multispace0, space1},
        combinator::{alt, delimited, opt, repeat, seq},
        error::{ContextError, ParseError, ParserError},
        stream::{AsChar, Stream, StreamIsPartial},
    };

    use super::{Character, Item, ItemKind};

    /// Creates a new [`Parser`] that accepts and trims any number of
    /// whitespace characters around the provided `parser` and returns it's
    /// result.
    fn ws<I, O, E>(parser: impl Parser<I, O, E>) -> impl Parser<I, O, E>
    where
        I: Stream + StreamIsPartial,
        I::Token: AsChar + Clone,
        E: ParserError<I>,
    {
        delimited(multispace0, parser, multispace0)
    }

    /// Parses the `input` string containing a enemy's properties and returns a
    /// new [`Enemy`] instance.
    ///
    /// The `input` must be in the same format as in the Advent of Code puzzle.
    pub fn enemy(input: &str) -> Result<Character, ParseError<&str, ContextError>> {
        let player = Parser::<_, _, ContextError>::parse(
            &mut seq!(
                _: ws("Hit Points:"),
                ws(dec_uint),
                _: ws("Damage:"),
                ws(dec_uint),
                _: ws("Armor:"),
                ws(dec_uint)
            )
            .map(|(health, damage, armor)| Character {
                health,
                damage,
                armor,
            }),
            input,
        )?;

        Ok(player)
    }

    /// Creates a new [`ItemKind`] from the `input` string.
    ///
    /// The `input` string must contain name of the [`ItemKind`] as in the
    /// Advent of Code puzzle shop description. So the valid names are:
    ///
    /// - Weapons
    /// - Armor
    /// - Rings
    fn item_kind(input: &mut &str) -> winnow::Result<ItemKind> {
        alt((
            "Weapons".value(ItemKind::Weapon),
            "Armor".value(ItemKind::Armor),
            "Rings".value(ItemKind::Ring),
        ))
        .parse_next(input)
    }

    /// Parses name of a shop item.
    ///
    /// Valid names are all ASCII alphabetic characters with an optional "+"
    /// followed by at least one digit.
    ///
    /// A first part of the name and +<digits> may be separated by any number of
    /// spaces (including 0), but they mustn't be any spaces between + and
    /// digits.
    fn item_name<'a>(input: &mut &'a str) -> winnow::Result<&'a str> {
        seq!(alpha1, _: space1, opt(("+", digit1)))
            .take()
            .parse_next(input)
    }

    /// Same as the [`Item`] struct, but does not have the [`Item::kind`] field.
    ///
    /// It's required as a shop description does not contain an item kind with
    /// each item, but once per section.
    struct ItemStats {
        cost: u32,
        damage: u8,
        armor: u8,
    }

    /// Parses a single item from a shop description.
    fn item(input: &mut &str) -> winnow::Result<ItemStats> {
        (item_name, ws(dec_uint), ws(dec_uint), ws(dec_uint))
            .map(|(_name, cost, damage, armor)| ItemStats {
                cost,
                damage,
                armor,
            })
            .parse_next(input)
    }

    /// Parses a single section from a shop description and returns it's items.
    fn shop_section(input: &mut &str) -> winnow::Result<Vec<Item>> {
        seq!(
            ws(item_kind),
            _: (ws(":"), ws("Cost"), ws("Damage"), ws("Armor")),
            repeat(1.., item),
        )
        .map(|(kind, partials): (_, Vec<_>)| {
            partials
                .into_iter()
                .map(|partial| Item {
                    kind,
                    cost: partial.cost,
                    damage: partial.damage,
                    armor: partial.armor,
                })
                .collect()
        })
        .parse_next(input)
    }

    /// Parses a whole shop description.
    pub fn shop(input: &str) -> Result<Vec<Item>, ParseError<&str, ContextError>> {
        let items = repeat(1.., shop_section)
            .map(|sections: Vec<_>| sections.into_iter().flatten().collect())
            .parse(input)?;

        Ok(items)
    }
}

/// Solves the first puzzle from the 21st day of the Advent of Code 2015 event.
pub async fn p1() -> Result<()> {
    let shop = Shop::default();
    let enemy = read_enemy().await?;
    let mut equipments: Vec<_> = Equipment::all(&shop).collect();
    equipments.sort_unstable_by_key(|eq| eq.cost());

    let won_eq = equipments.into_iter().find(|eq| {
        find_winner(
            &Character {
                health: 100,
                armor: eq.armor(),
                damage: eq.damage(),
            },
            &enemy,
        ) == Winner::Player
    });

    println!(
        "Answer: {}",
        won_eq.context("Winning equipment not found")?.cost()
    );

    Ok(())
}

/// Solves the second puzzle from the 21st day of the Advent of Code 2015 event.
pub async fn p2() -> Result<()> {
    let shop = Shop::default();
    let enemy = read_enemy().await?;
    let mut equipments: Vec<_> = Equipment::all(&shop).collect();
    equipments.sort_unstable_by_key(|eq| std::cmp::Reverse(eq.cost()));
    let lose_eq = equipments.iter().find(|eq| {
        find_winner(
            &Character {
                health: 100,
                armor: eq.armor(),
                damage: eq.damage(),
            },
            &enemy,
        ) == Winner::Enemy
    });

    println!(
        "Answer: {}",
        lose_eq.context("Winning equipment not found")?.cost()
    );

    Ok(())
}

/// Reads the input data for the puzzle.
async fn read_enemy() -> Result<Character> {
    let input = tokio::fs::read_to_string("inputs/y15_d21.txt").await?;
    parser::enemy(&input).map_err(|err| anyhow!("{}", err))
}

/// Compares statistics of the `player` and the `enemy` [`Character`]s and returns
/// a [`Winner`] enum specifying which of the [`Character`]s would win a fight.
fn find_winner(player: &Character, enemy: &Character) -> Winner {
    let player_damage = (player.damage as i32 - enemy.armor as i32).max(1);
    let enemy_damage = (enemy.damage as i32 - player.armor as i32).max(1);

    let turns_to_kill_enemy = (enemy.health as f32 / player_damage as f32).ceil() as i32;
    let turns_to_kill_player = (100.0 / enemy_damage as f32).ceil() as i32;

    if turns_to_kill_enemy <= turns_to_kill_player {
        Winner::Player
    } else {
        Winner::Enemy
    }
}

/// Describes a winner of a fight of two [`Character`]s.
#[derive(Debug, PartialEq)]
enum Winner {
    Player,
    Enemy,
}

/// Represents a valid collection of items bough from a [`Shop`].
#[derive(Debug)]
struct Equipment<'item>(Vec<&'item Item>);

impl<'item> Equipment<'item> {
    /// Generates all possible equipments that can be bough from the specified
    /// [`Shop`].
    pub fn all(shop: &'item Shop) -> impl Iterator<Item = Self> {
        shop.weapons()
            .map(|weapon| vec![weapon])
            .flat_map(|items| Self::all_with_armors(shop, items))
            .flat_map(|items| Self::all_with_rings(shop, items))
            .map(Self)
    }

    /// Creates an iterator over all possible combinations of the provided
    /// [`Item`]s with armors that are in the [`Shop`].
    fn all_with_armors(
        shop: &'item Shop,
        items: Vec<&'item Item>,
    ) -> impl Iterator<Item = Vec<&'item Item>> {
        assert!(
            items.iter().all(|item| item.kind != ItemKind::Armor),
            "Items must not have any armors"
        );

        iter::once(items.clone()).chain(shop.armors().map(move |armor| {
            let mut items = items.clone();
            items.push(armor);
            items
        }))
    }

    /// Creates an iterator over all possible combinations of the provided
    /// [`Item`]s with rings that are in the [`Shop`].
    fn all_with_rings(
        shop: &'item Shop,
        items: Vec<&'item Item>,
    ) -> impl Iterator<Item = Vec<&'item Item>> {
        let no_rings = iter::once(items.clone());

        let one_ring = shop.rings().map({
            let items = items.clone();
            move |ring| {
                let mut items = items.clone();
                items.push(ring);
                items
            }
        });

        let two_rings = shop
            .rings()
            .collect::<Vec<_>>()
            .into_iter()
            .tuple_combinations()
            .map(move |(a, b)| {
                let mut items = items.clone();
                items.push(a);
                items.push(b);
                items
            });

        no_rings.chain(one_ring).chain(two_rings)
    }

    /// Calculates the cost of all items in this [`Equipment`].
    fn cost(&self) -> u32 {
        self.0.iter().map(|item| item.cost).sum()
    }

    fn damage(&self) -> u8 {
        self.0.iter().map(|item| item.damage).sum()
    }

    fn armor(&self) -> u8 {
        self.0.iter().map(|item| item.armor).sum()
    }
}

/// Valid item kinds.
#[derive(Debug, Clone, Copy, PartialEq)]
enum ItemKind {
    Weapon,
    Armor,
    Ring,
}

/// An item from a shop.
#[derive(Debug)]
struct Item {
    kind: ItemKind,
    cost: u32,
    damage: u8,
    armor: u8,
}

/// A collection of items that can be bough by a player.
#[derive(Debug)]
struct Shop {
    items: Vec<Item>,
}

impl Shop {
    /// Creates an iterator over weapon items within the shop.
    fn weapons(&self) -> impl Iterator<Item = &Item> {
        self.items_of_kind(&ItemKind::Weapon)
    }

    /// Creates an iterator over armor items within the shop.
    fn armors(&self) -> impl Iterator<Item = &Item> {
        self.items_of_kind(&ItemKind::Armor)
    }

    /// Creates an iterator over ring items within the shop.
    fn rings(&self) -> impl Iterator<Item = &Item> {
        self.items_of_kind(&ItemKind::Ring)
    }

    /// Creates an iterator over items with the specified [`ItemKind`] within
    /// the shop.
    fn items_of_kind<'a, 'kind>(
        &'a self,
        kind: &'kind ItemKind,
    ) -> impl Iterator<Item = &'a Item> + use<'a, 'kind> {
        self.items.iter().filter(|&item| item.kind == *kind)
    }
}

impl Default for Shop {
    /// Creates a new default shop using the `shop.txt` file stored within the
    /// module's directory.
    fn default() -> Self {
        let input = include_str!("shop.txt");
        let items = parser::shop(input).unwrap();
        Self { items }
    }
}

/// An enemy's properties.
#[derive(Debug, Clone)]
struct Character {
    health: u8,
    damage: u8,
    armor: u8,
}
