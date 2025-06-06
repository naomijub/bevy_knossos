use bevy::ecs::resource::Resource;

use crate::utils::types::Coords;

use super::{
    errors::MazeSaveError,
    formatters::{Formatter, Saveable},
    grid::{Grid, cell::Cell},
    validate::validate,
};
use std::fmt;

/// An orthogonal maze
///
/// Represents a standard orthogonal maze where each cell is a square containing zero or maximum
/// three walls
#[derive(Debug, Clone, PartialEq, Eq, Resource)]
pub struct OrthogonalMaze {
    grid: Grid,
}

impl OrthogonalMaze {
    /// Returns a new instance of an orthogonal maze with a given width and height
    #[must_use]
    pub fn new(width: usize, height: usize) -> OrthogonalMaze {
        OrthogonalMaze {
            grid: Grid::new(width, height),
        }
    }

    /// Returns a mutable ref to a grid
    pub const fn get_grid_mut(&mut self) -> &mut Grid {
        &mut self.grid
    }

    /// Returns `true` if a maze is valid. Otherwise, returns `false`
    #[must_use]
    pub fn is_valid(&self) -> bool {
        validate(&self.grid)
    }

    /// Saves the maze data to a file at the specified path using the provided formatter.
    ///
    /// This method converts the internal grid representation of the maze into a format
    /// defined by the `Formatter` and `Saveable` trait implementations, and then saves it
    /// to the specified file path.
    ///
    /// # Example
    /// ```rust
    /// use bevy_knossos::maze::*;
    ///
    /// let maze = OrthogonalMaze::new(5, 5);
    /// maze.save("output/maze.txt", AsciiNarrow);
    /// ```
    ///
    /// # Errors
    /// This function can return a `MazeSaveError` if the file could not be written.
    pub fn save<F, T>(&self, path: &str, formatter: F) -> Result<String, MazeSaveError>
    where
        F: Formatter<T>,
        T: Saveable,
    {
        let data = formatter.format(&self.grid);
        Saveable::save(&data, path)
    }

    /// Returns an iterator over the maze where `index == y * Maze::width + x`.
    ///
    /// The iterator yields all items, `(Coords, Cell)`, from start to end.
    #[must_use]
    #[expect(clippy::iter_without_into_iter)] // Implemented elsewhere
    pub const fn iter(&self) -> OrthogonalMazeIterator {
        OrthogonalMazeIterator {
            maze: self,
            index: 0,
        }
    }

    /// Returns all cells that have 3 walls, means maze ends.
    #[must_use]
    pub fn ends(&self) -> Vec<((usize, usize), &Cell)> {
        self.iter()
            .filter(|maze_cell| maze_cell.1.walls_count() == 3)
            .collect()
    }

    /// Returns a formatted maze using the provided formatter.
    ///
    /// This method generates a formatted representation of the maze grid by applying
    /// the specified formatter, which converts the internal maze structure into the
    /// desired format `T`. This formatted result can then be further processed or saved
    /// elsewhere.
    ///
    /// # Example
    /// ```rust
    /// use bevy_knossos::maze::*;
    ///
    /// let maze = OrthogonalMaze::new(5, 5);
    /// let formatted_maze = maze.format(GameMap::new());
    /// ```
    ///
    /// # Note
    /// This function can be useful when the formatted maze needs to be used for further processing,
    /// logging, or display.
    pub fn format<F, T>(&self, formatter: F) -> T
    where
        F: Formatter<T>,
        T: Saveable,
    {
        formatter.format(&self.grid)
    }
}

impl std::ops::Index<Coords> for OrthogonalMaze {
    type Output = Cell;

    fn index(&self, index: Coords) -> &Self::Output {
        &self.grid[index]
    }
}

impl fmt::Display for OrthogonalMaze {
    /// Writes a formatted maze into a buffer
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.grid)?;
        Ok(())
    }
}

pub struct OrthogonalMazeIterator<'a> {
    maze: &'a OrthogonalMaze,
    index: usize,
}

impl<'a> Iterator for OrthogonalMazeIterator<'a> {
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

pub struct OrthogonalMazeIntoIterator {
    maze: OrthogonalMaze,
    index: usize,
    width: usize,
}

impl IntoIterator for OrthogonalMaze {
    type Item = (Coords, Cell);
    type IntoIter = OrthogonalMazeIntoIterator;

    fn into_iter(self) -> OrthogonalMazeIntoIterator {
        OrthogonalMazeIntoIterator {
            width: self.grid.width(),
            maze: self,
            index: 0,
        }
    }
}

impl Iterator for OrthogonalMazeIntoIterator {
    type Item = (Coords, Cell);

    fn next(&mut self) -> Option<Self::Item> {
        let width = self.width;
        if self.maze.grid.cells.is_empty() {
            return None;
        }
        let cell = self.maze.get_grid_mut().cells.remove(0);
        let coords = (self.index % width, self.index / width);
        self.index += 1;

        Some((coords, cell))
    }
}

#[cfg(test)]
mod tests {
    use crate::maze::grid::cell::Cell;

    use super::*;

    #[test]
    fn iterators_have_size() {
        let grid = generate_valid_maze();
        let maze = OrthogonalMaze { grid };

        let iter_count = maze.iter().count();

        assert_eq!(iter_count, 16);
        assert_eq!(
            maze.iter().nth(5).unwrap(),
            ((1usize, 1usize), &(Cell::SOUTH | Cell::EAST | Cell::WEST))
        );
    }

    #[test]
    fn into_iterators_have_size() {
        let grid = generate_valid_maze();
        let maze = OrthogonalMaze { grid };

        let iter_count = maze.clone().into_iter().count();

        assert_eq!(iter_count, 16);
        assert_eq!(
            maze.into_iter().nth(5).unwrap(),
            ((1usize, 1usize), (Cell::SOUTH | Cell::EAST | Cell::WEST))
        );
    }

    #[test]
    fn display_orthogonal_maze() {
        let mut expected = String::new();
        expected.push_str(" _______ \n");
        expected.push_str("| |___  |\n");
        expected.push_str("|_   _| |\n");
        expected.push_str("|  _____|\n");
        expected.push_str("|_______|\n");

        let grid = generate_valid_maze();
        let maze = OrthogonalMaze { grid };
        let actual = maze.to_string();

        assert_eq!(actual, expected);
    }

    #[test]
    fn valid_maze() {
        let grid = generate_valid_maze();
        let maze = OrthogonalMaze { grid };
        assert!(maze.is_valid());
    }

    #[test]
    fn invalid_maze() {
        let grid = generate_invalid_maze();
        let maze = OrthogonalMaze { grid };
        assert!(!maze.is_valid());
    }

    #[test]
    fn access_by_index_maze() {
        let grid = generate_valid_maze();
        let maze = OrthogonalMaze { grid };

        let cell = maze[(3, 1)];
        assert_eq!(cell, Cell::from_bits(0b0011).unwrap());
    }

    #[test]
    fn into_iterators_correct_index() {
        let grid = generate_valid_maze();
        let maze = OrthogonalMaze { grid };
        let width = maze.grid.width();

        maze.into_iter()
            .enumerate()
            .for_each(|(idx, (coord, cell))| {
                assert_eq!(idx, coord.1 * width + coord.0);
                assert!(!cell.is_empty());
            });
    }

    #[test]
    fn iterators_correct_index() {
        let grid = generate_valid_maze();
        let maze = OrthogonalMaze { grid };
        let width = maze.grid.width();

        maze.iter().enumerate().for_each(|(idx, (coord, cell))| {
            assert_eq!(idx, coord.1 * width + coord.0);
            assert!(!cell.is_empty());
        });
    }

    #[test]
    fn maze_ends() {
        let grid = generate_valid_maze();
        let maze = OrthogonalMaze { grid };

        let ends = maze.ends();

        assert_eq!(
            ends,
            vec![
                ((0, 0), &Cell::SOUTH),
                ((1, 0), &Cell::EAST),
                ((2, 1), &Cell::WEST),
                ((3, 3), &Cell::WEST)
            ]
        );
    }

    fn generate_valid_maze() -> Grid {
        let mut grid = Grid::new(4, 4);

        grid.carve_passage((0, 0), Cell::SOUTH).unwrap();
        grid.carve_passage((0, 1), Cell::EAST).unwrap();
        grid.carve_passage((0, 2), Cell::EAST).unwrap();
        grid.carve_passage((0, 2), Cell::SOUTH).unwrap();
        grid.carve_passage((0, 3), Cell::EAST).unwrap();

        grid.carve_passage((1, 0), Cell::EAST).unwrap();
        grid.carve_passage((1, 1), Cell::EAST).unwrap();
        grid.carve_passage((1, 1), Cell::SOUTH).unwrap();
        grid.carve_passage((1, 2), Cell::EAST).unwrap();
        grid.carve_passage((1, 3), Cell::EAST).unwrap();

        grid.carve_passage((2, 0), Cell::EAST).unwrap();
        grid.carve_passage((2, 2), Cell::EAST).unwrap();
        grid.carve_passage((2, 3), Cell::EAST).unwrap();

        grid.carve_passage((3, 1), Cell::NORTH).unwrap();
        grid.carve_passage((3, 1), Cell::SOUTH).unwrap();

        grid
    }

    fn generate_invalid_maze() -> Grid {
        let mut grid = Grid::new(4, 4);

        grid.carve_passage((0, 0), Cell::SOUTH).unwrap();
        grid.carve_passage((0, 1), Cell::EAST).unwrap();
        grid.carve_passage((0, 2), Cell::EAST).unwrap();
        grid.carve_passage((0, 2), Cell::SOUTH).unwrap();
        grid.carve_passage((0, 3), Cell::EAST).unwrap();

        grid.carve_passage((1, 1), Cell::EAST).unwrap();
        grid.carve_passage((1, 1), Cell::NORTH).unwrap();
        grid.carve_passage((1, 2), Cell::EAST).unwrap();
        grid.carve_passage((1, 3), Cell::EAST).unwrap();

        grid.carve_passage((2, 0), Cell::EAST).unwrap();
        grid.carve_passage((2, 2), Cell::EAST).unwrap();
        grid.carve_passage((2, 3), Cell::EAST).unwrap();

        grid.carve_passage((3, 1), Cell::NORTH).unwrap();
        grid.carve_passage((3, 1), Cell::SOUTH).unwrap();

        grid
    }
}
