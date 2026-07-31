use super::Algorithm;
use crate::maze::grid::{Grid, topology::Topology};
use crate::utils::types::Coords;
use rand::prelude::*;

/// The "Recursive Backtracking" algorithm for generating mazes
///
/// This algorithm quite effectively creates narrow passages with multiple dead-ends which makes it
/// easy to get lost, thus eventually making it hard to solve the maze.
///
/// In most cases, this algorithm is fast. However, due to its recursive nature, it requires stack
/// space proportional to the longest acyclic path, which is, in the worst case, the entire maze. So
/// for exceptionally large mazes this algorithm can be fairly inefficient.
pub struct RecursiveBacktracking;

/// An implementation of the "Recursive Backtracking" algorithm for generating mazes.
///
/// Here is how it works:
///
/// 1. Chooses a starting point in the field.
///
/// 2. Randomly chooses a wall at that point and carves a passage through to the adjacent cell,
///    but only if the adjacent cell has not been visited yet. This becomes the new current cell.
///
/// 3. If all adjacent cells have been visited, backs up to the last cell that has uncarved walls
///    and repeats.
///
/// 4. The algorithm ends when the process has backed all the way up to the starting
///    point.
impl Algorithm for RecursiveBacktracking {
    fn generate(&mut self, grid: &mut Grid, start_coords: Option<Coords>, rng: &mut StdRng) {
        let start_coords = start_coords.unwrap_or((0, 0));
        carve_passages_from(start_coords, grid, rng);
    }

    fn has_start_coords(&self) -> bool {
        true
    }

    fn name(&self) -> &'static str {
        "RecursiveBacktracking"
    }

    fn supports_topology(&self, _topology: Topology) -> bool {
        true
    }
}

fn carve_passages_from(coords: Coords, grid: &mut Grid, rng: &mut impl Rng) {
    // Keep the DFS frames on the heap instead of the call stack. Large mazes
    // can otherwise overflow the stack along a long single corridor.
    let mut initial_dirs = grid.directions().to_vec();
    initial_dirs.shuffle(rng);
    initial_dirs.reverse();
    let mut stack = vec![(coords, initial_dirs)];

    while let Some((current, directions)) = stack.last_mut() {
        let Some(direction) = directions.pop() else {
            stack.pop();
            continue;
        };
        let Ok(next) = grid.get_next_cell_coords(*current, direction) else {
            continue;
        };
        if grid.is_cell_visited(next) {
            continue;
        }

        if let Ok(next) = grid.carve_passage(*current, direction) {
            let mut next_dirs = grid.directions().to_vec();
            next_dirs.shuffle(rng);
            next_dirs.reverse();
            stack.push((next, next_dirs));
        }
    }
}
