#![allow(missing_docs)]

use rand::SeedableRng;
use rand::rngs::StdRng;

use crate::maze::HexMaze;
use crate::maze::algorithms::{Algorithm, RecursiveBacktracking};
use crate::utils::types::Coords;

use super::{builder::validate_dimensions_and_start, errors::BuildError};

/// A hex maze builder for constructing a maze step by step.
pub struct HexMazeBuilder {
    width: usize,
    height: usize,
    algorithm: Box<dyn Algorithm>,
    start_coords: Option<Coords>,
    seed: Option<u64>,
}

impl HexMazeBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self {
            width: 10,
            height: 10,
            algorithm: Box::new(RecursiveBacktracking),
            start_coords: None,
            seed: None,
        }
    }

    #[must_use]
    pub const fn seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    #[must_use]
    pub const fn width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    #[must_use]
    pub const fn height(mut self, height: usize) -> Self {
        self.height = height;
        self
    }

    #[must_use]
    pub fn algorithm(mut self, algorithm: Box<dyn Algorithm>) -> Self {
        self.algorithm = algorithm;
        self
    }

    #[must_use]
    pub fn start_coords(mut self, coord: impl Into<Coords>) -> Self {
        self.start_coords = Some(coord.into());
        self
    }

    /// Builds Hexagonal Maze
    ///
    /// # Errors
    /// - Fails for invalid dimensions, out-of-bounds start coordinates, or an algorithm that
    ///   does not support starting coordinates.
    pub fn build(mut self) -> Result<HexMaze, BuildError> {
        validate_dimensions_and_start(self.width, self.height, self.start_coords)?;
        if !self
            .algorithm
            .supports_topology(crate::maze::grid::topology::Topology::HexOddR)
        {
            return Err(BuildError::reason(format!(
                "algorithm `{}` does not support hexagonal topology",
                self.algorithm.name()
            )));
        }
        let mut maze = HexMaze::new(self.width, self.height);
        let mut rng = self.seed.map_or_else(
            || {
                let mut rng = rand::rng();
                StdRng::from_rng(&mut rng)
            },
            StdRng::seed_from_u64,
        );
        if self.start_coords.is_some() && !self.algorithm.has_start_coords() {
            Err(BuildError::reason(format!(
                "Algorithm `{}` doesn't support `start_coords`",
                self.algorithm.name()
            )))
        } else {
            self.algorithm
                .generate(maze.get_grid_mut(), self.start_coords, &mut rng);
            Ok(maze)
        }
    }
}

impl Default for HexMazeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn rejects_zero_dimensions() {
        for (width, height) in [(0, 1), (1, 0)] {
            assert!(
                HexMazeBuilder::new()
                    .width(width)
                    .height(height)
                    .build()
                    .is_err()
            );
        }
    }

    #[test]
    fn rejects_out_of_bounds_start_coordinates() {
        for start_coords in [(2, 0), (0, 3)] {
            assert!(
                HexMazeBuilder::new()
                    .width(2)
                    .height(3)
                    .start_coords(start_coords)
                    .build()
                    .is_err()
            );
        }
    }

    #[test]
    fn accepts_boundary_start_coordinates() {
        assert!(
            HexMazeBuilder::new()
                .width(2)
                .height(3)
                .start_coords((1, 2))
                .build()
                .unwrap()
                .is_valid()
        );
    }
}
