use std::{collections::HashSet, ops::AddAssign};

use anyhow::anyhow;

use super::Vec2;

enum Direction {
    North,
    South,
    East,
    West,
}

impl TryFrom<char> for Direction {
    type Error = anyhow::Error;

    fn try_from(ch: char) -> Result<Self, Self::Error> {
        match ch {
            '^' => Ok(Direction::North),
            '>' => Ok(Direction::East),
            'v' => Ok(Direction::South),
            '<' => Ok(Direction::West),
            _ => Err(anyhow!("Invalid direction: {}", ch)),
        }
    }
}

impl AddAssign<Direction> for Vec2 {
    fn add_assign(&mut self, rhs: Direction) {
        match rhs {
            Direction::North => self.y += 1,
            Direction::South => self.y -= 1,
            Direction::East => self.x += 1,
            Direction::West => self.x -= 1,
        }
    }
}

struct State {
    visited: HashSet<Vec2>,
    current_index: usize,
    current: Vec<Vec2>,
}

impl State {
    fn new(santas: usize) -> Self {
        assert_ne!(santas, 0);

        let mut visited = HashSet::new();
        visited.insert(Vec2::ORIGIN);

        let mut current = Vec::with_capacity(santas);
        for _ in 0..santas {
            current.push(Vec2::ORIGIN);
        }

        Self {
            visited,
            current_index: 0,
            current,
        }
    }

    fn translate(&mut self, dir: Direction) {
        let current = &mut self.current[self.current_index];
        *current += dir;
        self.visited.insert(*current);

        self.current_index = self.current_index.wrapping_add(1) % self.current.len();
    }
}

async fn answer(state: State) -> anyhow::Result<()> {
    let input = tokio::fs::read_to_string("inputs/y15_d03.txt").await?;
    let state =
        input
            .chars()
            .map(|ch| Direction::try_from(ch))
            .try_fold(state, |mut state, dir| {
                state.translate(dir?);
                Ok::<_, anyhow::Error>(state)
            })?;
    println!("Answer: {}", state.visited.len());
    Ok(())
}

pub async fn p1() -> anyhow::Result<()> {
    answer(State::new(1)).await
}

pub async fn p2() -> anyhow::Result<()> {
    answer(State::new(2)).await
}
