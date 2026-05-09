#![allow(missing_docs)]

use bevy::ecs::resource::Resource;

use crate::utils::types::Coords;

use super::{
    errors::MazeSaveError,
    formatters::{Formatter, Saveable},
    grid::{Grid, cell::Cell},
    validate::validate,
};

/// A hex maze represented using an odd-r offset layout.
#[derive(Debug, Clone, PartialEq, Eq, Resource)]
pub struct HexMaze {
    grid: Grid,
}

impl HexMaze {
    #[must_use]
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            grid: Grid::new_hex(width, height),
        }
    }

    pub const fn get_grid_mut(&mut self) -> &mut Grid {
        &mut self.grid
    }

    #[must_use]
    pub fn is_valid(&self) -> bool {
        validate(&self.grid)
    }

    /// Saves maze
    /// 
    /// # Errors
    /// fails to save maze due to incorrect formatting
    pub fn save<F, T>(&self, path: &str, formatter: F) -> Result<String, MazeSaveError>
    where
        F: Formatter<T>,
        T: Saveable,
    {
        let data = formatter.format(&self.grid);
        Saveable::save(&data, path)
    }

    #[must_use]
    pub fn ends(&self) -> Vec<((usize, usize), &Cell)> {
        self.iter()
            .filter(|maze_cell| maze_cell.1.walls_count_for(self.grid.topology().sides()) + 1 == self.grid.topology().sides())
            .collect()
    }

    pub fn format<F, T>(&self, formatter: F) -> T
    where
        F: Formatter<T>,
        T: Saveable,
    {
        formatter.format(&self.grid)
    }

    #[must_use]
    #[expect(clippy::iter_without_into_iter)]
    pub const fn iter(&'_ self) -> HexMazeIterator<'_> {
        HexMazeIterator {
            maze: self,
            index: 0,
        }
    }
}

impl std::ops::Index<Coords> for HexMaze {
    type Output = Cell;

    fn index(&self, index: Coords) -> &Self::Output {
        &self.grid[index]
    }
}

pub struct HexMazeIterator<'a> {
    maze: &'a HexMaze,
    index: usize,
}

impl<'a> Iterator for HexMazeIterator<'a> {
    type Item = (Coords, &'a Cell);
    fn next(&mut self) -> Option<Self::Item> {
        let width = self.maze.grid.width();
        if self.index < self.maze.grid.cells.len() {
            let result = Some((
                (self.index % width, self.index / width),
                &self.maze.grid.cells[self.index],
            ));
            self.index += 1;
            result
        } else {
            None
        }
    }
}
