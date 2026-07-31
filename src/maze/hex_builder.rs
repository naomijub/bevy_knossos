#![allow(missing_docs)]

use rand::SeedableRng;
use rand::rngs::StdRng;

use crate::maze::HexMaze;
use crate::maze::algorithms::{Algorithm, RecursiveBacktracking};
use crate::utils::types::Coords;

use super::errors::BuildError;

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
    /// - Fails if selected algorithm doesnt have a set `start_coords` or it doesnt support starting coords.
    pub fn build(mut self) -> Result<HexMaze, BuildError> {
        if self.width == 0 || self.height == 0 {
            return Err(BuildError::reason(
                "hex maze dimensions must be greater than zero",
            ));
        }
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
            Err(BuildError::reason(self.algorithm.name()))
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
