//! Maze representations, builders, formatters, and supported algorithms for generating mazes
//!
//! Acts as a prelude module with all the imports that are necessary for generating and saving
//! mazes.

mod builder;
mod errors;
mod grid;
mod hex_builder;
mod hex_maze;
#[allow(clippy::module_inception)]
mod maze;
mod validate;

pub mod algorithms;
pub mod formatters;

pub use algorithms::*;
pub use builder::OrthogonalMazeBuilder;
pub use errors::MazeSaveError;
pub use formatters::{AsciiBroad, AsciiNarrow, GameMap, Image};
pub use formatters::HexText;
pub use grid::cell::Cell;
pub use grid::topology::Topology;
pub use hex_builder::HexMazeBuilder;
pub use hex_maze::HexMaze;
pub use maze::OrthogonalMaze;
