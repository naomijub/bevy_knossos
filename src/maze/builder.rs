use rand::SeedableRng;
use rand::rngs::StdRng;

use crate::maze::OrthogonalMaze;
use crate::maze::algorithms::{Algorithm, RecursiveBacktracking};
use crate::utils::types::Coords;

use super::errors::BuildError;

/// An orthogonal maze builder for constructing a maze step by step
pub struct OrthogonalMazeBuilder {
    width: usize,
    height: usize,
    algorithm: Box<dyn Algorithm>,
    start_coords: Option<Coords>,
    seed: Option<u64>,
}

pub(super) fn validate_dimensions_and_start(
    width: usize,
    height: usize,
    start_coords: Option<Coords>,
) -> Result<(), BuildError> {
    if width == 0 || height == 0 {
        return Err(BuildError::reason(
            "maze dimensions must be greater than zero",
        ));
    }

    if let Some((x, y)) = start_coords
        && (x >= width || y >= height)
    {
        return Err(BuildError::reason(format!(
            "start coordinates ({x}, {y}) are outside maze dimensions {width}x{height}"
        )));
    }

    Ok(())
}

impl OrthogonalMazeBuilder {
    /// Returns a new instance of a builder with the default width, height and algorithm
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

    /// Sets a seed value for deterministic generation and returns itself
    #[must_use]
    pub const fn seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Sets a maze width and returns itself
    #[must_use]
    pub const fn width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    /// Sets a maze height and returns itself
    #[must_use]
    pub const fn height(mut self, height: usize) -> Self {
        self.height = height;
        self
    }

    /// Sets an algorithm for generating a maze and returns itself
    #[must_use]
    pub fn algorithm(mut self, algorithm: Box<dyn Algorithm>) -> Self {
        self.algorithm = algorithm;
        self
    }

    /// Sets start coords for arguments that allow start coords
    #[must_use]
    pub fn start_coords(mut self, coord: impl Into<Coords>) -> Self {
        self.start_coords = Some(coord.into());
        self
    }

    /// Builds a maze and returns a resulting object of the generated orthogonal maze
    ///
    /// # Errors
    /// Returns a [`BuildError`] for invalid dimensions, out-of-bounds start coordinates,
    /// or an algorithm that does not support start coordinates.
    pub fn build(mut self) -> Result<OrthogonalMaze, BuildError> {
        validate_dimensions_and_start(self.width, self.height, self.start_coords)?;
        let mut maze = OrthogonalMaze::new(self.width, self.height);
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

impl Default for OrthogonalMazeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use crate::maze::RecursiveDivision;

    use super::*;

    #[test]
    fn build() {
        let maze = OrthogonalMazeBuilder::default().build().unwrap();
        assert!(maze.is_valid());
    }

    #[test]
    fn no_start_coord_support() {
        let maze_err = OrthogonalMazeBuilder::default()
            .start_coords((3, 3))
            .algorithm(Box::new(RecursiveDivision {}))
            .build()
            .unwrap_err();
        assert_eq!(
            maze_err.to_string(),
            "Cannot build maze. Reason: Algorithm `RecursiveDivision` doesn't support `start_coords`"
        );
    }

    #[test]
    fn rejects_zero_dimensions() {
        for (width, height) in [(0, 1), (1, 0)] {
            assert!(
                OrthogonalMazeBuilder::new()
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
                OrthogonalMazeBuilder::new()
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
            OrthogonalMazeBuilder::new()
                .width(2)
                .height(3)
                .start_coords((1, 2))
                .build()
                .unwrap()
                .is_valid()
        );
    }
}
