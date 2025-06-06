use super::grid::{Grid, cell::Cell};
use crate::utils::types::Coords;
use rand::prelude::*;

/// A utility to validate if a given grid is valid, i.e. all the cells are reachable.
///
/// The recursive backtracker is one of the simplest and most efficient algorithms
/// for this kind of work. If an algorithm does not visit all the cells, we make a
/// conclusion that it's not valid.
pub fn validate(grid: &Grid) -> bool {
    let mut visited: Vec<Coords> = Vec::new();
    visited.push((0, 0));
    visit((0, 0), grid, &mut visited);
    visited.len() == grid.width() * grid.height()
}

fn visit(coords: Coords, grid: &Grid, visited: &mut Vec<Coords>) {
    let mut dirs = [Cell::NORTH, Cell::SOUTH, Cell::WEST, Cell::EAST];
    dirs.shuffle(&mut rand::rng());

    for dir in dirs {
        let Ok(next) = grid.get_next_cell_coords(coords, dir) else {
            continue;
        };
        if visited.contains(&next) {
            continue;
        }

        if !grid.is_carved(coords, dir) {
            continue;
        }

        visited.push(next);
        visit(next, grid, visited);
    }
}
