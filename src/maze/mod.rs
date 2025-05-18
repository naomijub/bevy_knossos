//! Maze representations, builders, formatters, and supported algorithms for generating mazes
//!
//! Acts as a prelude module with all the imports that are necessary for generating and saving
//! mazes.

mod builder;
mod errors;
mod grid;
#[allow(clippy::module_inception)]
mod maze;
mod validate;

pub mod algorithms;
pub mod formatters;

pub use algorithms::*;
pub use builder::OrthogonalMazeBuilder;
pub use errors::MazeSaveError;
pub use formatters::{AsciiBroad, AsciiNarrow, GameMap, Image};
pub use grid::cell::Cell;
pub use maze::OrthogonalMaze;
