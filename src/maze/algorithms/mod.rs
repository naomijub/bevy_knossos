//! Algorithms for generating mazes

mod aldous_broder;
mod binary_tree;
mod eller;
mod growing_tree;
mod hunt_and_kill;
mod kruskal;
mod prim;
mod recursive_backtracking;
mod recursive_division;
mod sidewinder;

pub use aldous_broder::AldousBroder;
pub use binary_tree::{Bias, BinaryTree};
pub use eller::Eller;
pub use growing_tree::{GrowingTree, Method};
pub use hunt_and_kill::HuntAndKill;
pub use kruskal::Kruskal;
pub use prim::Prim;
pub use recursive_backtracking::RecursiveBacktracking;
pub use recursive_division::RecursiveDivision;
pub use sidewinder::Sidewinder;

use crate::{maze::grid::Grid, utils::types::Coords};
use rand::rngs::StdRng;

pub(super) const BOOL_TRUE_PROBABILITY: f64 = 0.5;

/// A trait for generating a maze using a selected algorithm
pub trait Algorithm {
    /// Runs algorithm through the given Grid object, thus mutating the grid and generating a new
    /// maze.
    fn generate(&mut self, grid: &mut Grid, start_coords: Option<Coords>, rng: &mut StdRng);

    /// Verifies if algorithm supports start coords
    fn has_start_coords(&self) -> bool;

    // Cannot be a const because of dyn-trait compatibility
    /// Algorithm name
    fn name(&self) -> &'static str;
}
