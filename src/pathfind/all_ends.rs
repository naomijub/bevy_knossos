use bevy::prelude::*;
use pathfinding::prelude::astar;
use crate::{
    maze::Cell,
    pathfind::{MazePath, Cost},
    Coords, CoordsComponent, Start,
};

use std::collections::HashMap;

/// Auxiliary struct that holds knowledge for path finding on each [`Cell`]
#[derive(Debug, Clone, PartialEq, Resource, Default, Reflect)]
pub struct MazeEndsPaths {
    /// Map containing all ends and their paths from the [`Start`] component [`Coords`]
    pub paths: HashMap<(Coords, Coords), (Vec<CoordsComponent>, u32)>,
}

impl MazeEndsPaths {
    /// Checks if a `path_coord` ([`Coords`]) is contained in the path from [`Start`] to [`MazeEnd`]
    pub fn contains_path_to_end(&self, goal: Coords, path_coord: Coords) -> bool {
        self.paths
            .iter()
            .find(|((_start, end), (_path, _cost))| end == &goal)
            .is_some_and(|((_start, _end), (path, _cost))| path.contains(&(path_coord.into())))
    }
}

/// Component that signals that the cell is a Maze End.
#[derive(Debug, Clone, PartialEq, Eq, Component, Default, Reflect)]
pub struct MazeEnd;

/// Creates resource [`MazeEndsPaths`] that defines paths for all ends in the maze.
/// This function should be called on demand and is not scheduled to run.
///
/// # Warning
/// This operation is quite slow for large mazes, as it needs to pathfind over all ends.
/// issue
#[cfg(not(tarpaulin_include))]
#[cfg(not(feature = "single_end"))]
pub fn find_maze_ends_paths(
    mut commands: Commands,
    start: Query<&CoordsComponent, (With<Cell>, With<Start>)>,
    cells: Query<(Entity, &CoordsComponent, &Cell, Option<&Cost>)>,
) {
    let Ok(start) = start.get_single().cloned() else {
        return;
    };

    let mut ends: Vec<((usize, usize), &Cell)> = Vec::default();

    for (entity, coords, cell, ..) in &cells {
        if cell.is_end() {
            commands.entity(entity).insert(MazeEnd);
            ends.push((coords.coord, cell));
        }
    }

    let cells: HashMap<&CoordsComponent, (&Cell, Option<&Cost>)> = cells
        .iter()
        .map(|(_entity, k, v1, v2)| (k, (v1, v2)))
        .collect();

    let paths = ends
        .into_iter()
        .filter_map(|(goal, _cell)| {
            let goal_comp: CoordsComponent = goal.into();

            Some((
                (start.clone().into(), goal),
                astar(
                    &start,
                    |p| MazePath::successors(p, &cells),
                    |p| MazePath::distance(p, &goal_comp),
                    |p| p == &goal_comp,
                )?,
            ))
        })
        .collect();

    commands.insert_resource(MazeEndsPaths { paths });
}
