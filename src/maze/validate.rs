use super::grid::Grid;
use crate::utils::types::Coords;
use rand::prelude::*;

/// A utility to validate if a given grid is valid, i.e. all the cells are reachable.
///
/// The recursive backtracker is one of the simplest and most efficient algorithms
/// for this kind of work. If an algorithm does not visit all the cells, we make a
/// conclusion that it's not valid.
pub fn validate(grid: &Grid) -> bool {
    if grid.width() == 0 || grid.height() == 0 {
        return false;
    }
    let mut visited: Vec<Coords> = Vec::new();
    visited.push((0, 0));
    visit((0, 0), grid, &mut visited);
    if visited.len() != grid.width() * grid.height() {
        return false;
    }

    // Every passage must be represented in both cells. This catches malformed
    // imported mazes as well as topology/rendering direction mismatches.
    for y in 0..grid.height() {
        for x in 0..grid.width() {
            for direction in grid.directions() {
                if !grid.is_carved((x, y), *direction) {
                    continue;
                }
                let Ok(next) = grid.get_next_cell_coords((x, y), *direction) else {
                    return false;
                };
                let Some(opposite) = grid.topology().opposite(*direction) else {
                    return false;
                };
                if !grid.is_carved(next, opposite) {
                    return false;
                }
            }
        }
    }
    true
}

fn visit(coords: Coords, grid: &Grid, visited: &mut Vec<Coords>) {
    let mut dirs = grid.directions().to_vec();
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
