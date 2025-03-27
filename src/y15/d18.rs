use anyhow::{Context, Result};
use itertools::Itertools;

use super::Vec2;

const ROWS: usize = 100;

const COLS: usize = 100;

#[derive(Debug)]
struct Grid {
    cells: [bool; COLS * ROWS],
    always_on: Vec<Vec2<usize>>,
}

impl Grid {
    /// Updates the [`cells`](Self::cells) of the grid according to the rules of Conway's Game of Life.
    fn update(&mut self) {
        let updates: Vec<_> = self
            .cells
            .iter()
            .enumerate()
            .map(|(i, &cell)| {
                let alive_around = self.alive_around(i);
                matches!((cell, alive_around), (true, 2..=3) | (false, 3))
            })
            .collect();

        for (i, state) in updates.into_iter().enumerate() {
            if !self.is_always_on(Self::coords(i)) {
                self.cells[i] = state;
            }
        }
    }

    /// Updates the [`cells`](Self::cells) of the grid `n` times using [`update`](Self::update) function.
    fn update_n(&mut self, n: usize) {
        for _ in 0..n {
            self.update();
        }
    }

    /// Checks if the cell at the given coordinates is always on.
    fn is_always_on(&self, coords: Vec2<usize>) -> bool {
        self.always_on.contains(&coords)
    }

    /// Marks the cell at the given coordinates as always on.
    fn set_always_on(&mut self, coords: Vec2<usize>) {
        self.cells[Self::index(coords)] = true;
        self.always_on.push(coords);
    }

    /// Marks the cells at the given coordinates as always on.
    fn extend_always_on(&mut self, coords: impl IntoIterator<Item = Vec2<usize>>) {
        for coord in coords {
            self.set_always_on(coord);
        }
    }

    /// Returns the number of alive cells in the grid.
    fn alive(&self) -> usize {
        self.cells.iter().filter(|&&cell| cell).count()
    }

    /// Returns the number of alive neighbors around the cell at the given index.
    fn alive_around(&self, i: usize) -> usize {
        let coords = Self::coords(i);
        let bottom_x_bound = if coords.x > 0 { 1 } else { 0 };
        let top_x_bound = if coords.x < COLS - 1 { 1 } else { 0 };
        let bottom_y_bound = if coords.y > 0 { 1 } else { 0 };
        let top_y_bound = if coords.y < ROWS - 1 { 1 } else { 0 };
        (coords - Vec2::new(bottom_x_bound, bottom_y_bound))
            .points_to_inclusive(coords + Vec2::new(top_x_bound, top_y_bound))
            .filter(|v| *v != coords)
            .map(|coords| self.cells[Self::index(coords)])
            .filter(|cell| *cell)
            .count()
    }

    /// Converts the coordinates to an index in the [`cells`][Self::cells] array.
    fn index(coords: Vec2<usize>) -> usize {
        coords.y * COLS + coords.x
    }

    /// Converts an index to coordinates.
    fn coords(i: usize) -> Vec2<usize> {
        let x = i % COLS;
        let y = i / COLS;
        Vec2 { x, y }
    }
}

/// Reads the [`Grid`] from the input file.
async fn read_grid() -> Result<Grid> {
    let data = tokio::fs::read_to_string("inputs/y15_d18.txt").await?;
    let cells = data
        .chars()
        .filter_map(|c| match c {
            '#' => Some(true),
            '.' => Some(false),
            _ => None,
        })
        .collect_array()
        .context("Invalid number of cells within file")?;
    Ok(Grid {
        cells,
        always_on: vec![],
    })
}

/// Solves the first part of the problem.
pub async fn p1() -> Result<()> {
    let mut grid = read_grid().await?;
    grid.update_n(100);
    println!("Answer: {}", grid.alive());
    Ok(())
}

/// Solves the second part of the problem.
pub async fn p2() -> Result<()> {
    let mut grid = read_grid().await?;
    grid.extend_always_on([
        Vec2::new(0, 0),
        Vec2::new(0, ROWS - 1),
        Vec2::new(COLS - 1, 0),
        Vec2::new(COLS - 1, ROWS - 1),
    ]);
    grid.update_n(100);
    println!("Answer: {}", grid.alive());
    Ok(())
}
