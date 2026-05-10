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

    /// Serializes the maze into a human redable text format.
    ///
    /// The output can be parsed back via [`Self::from_text`].
    #[must_use]
    pub fn to_text(&self) -> String {
        self.format(super::formatters::HexText).into_inner()
    }

    /// Deserializes a maze from [`Self::to_text`] output.
    ///
    /// # Errors
    /// Returns [`MazeSaveError`] when text is malformed or inconsistent.
    pub fn from_text(input: &str) -> Result<Self, MazeSaveError> {
        let mut lines = input.lines();
        let Some(header) = lines.next() else {
            return Err(MazeSaveError::reason("Missing header line"));
        };
        if header.trim() != "KNOSSOS_HEX_V1" {
            return Err(MazeSaveError::reason("Invalid hex maze header"));
        }

        let Some(width_line) = lines.next() else {
            return Err(MazeSaveError::reason("Missing width line"));
        };
        let Some(height_line) = lines.next() else {
            return Err(MazeSaveError::reason("Missing height line"));
        };

        let width = parse_dimension(width_line, "width")?;
        let height = parse_dimension(height_line, "height")?;

        let mut maze = Self::new(width, height);
        let grid = maze.get_grid_mut();

        for y in 0..height {
            let Some(line) = lines.next() else {
                return Err(MazeSaveError::reason("Missing cell row"));
            };

            let cols: Vec<&str> = line.split(',').collect();
            if cols.len() != width {
                return Err(MazeSaveError::reason(format!(
                    "Invalid row width at y={y}: expected {width}, got {}",
                    cols.len()
                )));
            }

            for (x, token) in cols.iter().enumerate() {
                let bits = u8::from_str_radix(token.trim(), 16).map_err(|err| {
                    MazeSaveError::reason(format!(
                        "Invalid cell token at x={x}, y={y}: `{token}` ({err})"
                    ))
                })?;
                grid.cells[y * width + x] = Cell::from_bits_retain(bits);
            }
        }

        Ok(maze)
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

fn parse_dimension(line: &str, key: &str) -> Result<usize, MazeSaveError> {
    let Some((k, value)) = line.split_once('=') else {
        return Err(MazeSaveError::reason(format!(
            "Malformed dimension line `{line}`"
        )));
    };
    if k.trim() != key {
        return Err(MazeSaveError::reason(format!(
            "Expected `{key}=...`, got `{line}`"
        )));
    }

    value
        .trim()
        .parse::<usize>()
        .map_err(|err| MazeSaveError::reason(format!("Invalid {key} value `{value}`: {err}")))
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

#[cfg(test)]
mod tests {
    use crate::maze::HexMazeBuilder;

    use super::HexMaze;

    #[test]
    fn text_roundtrip() {
        let maze = HexMazeBuilder::new()
            .width(6)
            .height(5)
            .seed(42)
            .build()
            .unwrap();
        let text = maze.to_text();
        let restored = HexMaze::from_text(&text).unwrap();
        let maze_cells: Vec<((usize, usize), u8)> =
            maze.iter().map(|(coord, cell)| (coord, cell.to_bits())).collect();
        let restored_cells: Vec<((usize, usize), u8)> = restored
            .iter()
            .map(|(coord, cell)| (coord, cell.to_bits()))
            .collect();

        assert_eq!(maze_cells, restored_cells);
    }

    #[test]
    fn invalid_header() {
        let input = "NOPE\nwidth=1\nheight=1\n00\n";
        let err = HexMaze::from_text(input).unwrap_err();
        assert_eq!(
            err.to_string(),
            "Cannot save maze to file. Reason: Invalid hex maze header"
        );
    }
}
