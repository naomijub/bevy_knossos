use bevy::{prelude::*, platform_support::collections::HashMap};
use bevy_ecs_tilemap::prelude::*;
// use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_knossos::{
    maze::{self, Cell},
    pathfind::MazePath,
    CellSize, CoordsComponent, Goal, KnossosPlugin, Start,
};

const MAZE_SIZE: u32 = 16;
const CELL_SIZE: f32 = 64.;
fn main() {
    let maze = maze::OrthogonalMazeBuilder::new()
        .algorithm(Box::new(maze::RecursiveBacktracking))
        .width(MAZE_SIZE as usize)
        .height(MAZE_SIZE as usize)
        .build()
        .unwrap();

    App::new()
        .insert_resource(maze)
        .add_plugins((DefaultPlugins, TilemapPlugin))
        .add_plugins(KnossosPlugin,)
        // .add_plugins((KnossosPlugin, WorldInspectorPlugin::new()))
        .add_systems(Startup, setup)
        .add_systems(Update, draw_path)
        .run();
}

fn setup(mut commands: Commands, maze: Res<maze::OrthogonalMaze>, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: bevy::render::camera::ScalingMode::AutoMin {
                min_width: CELL_SIZE * MAZE_SIZE as f32,
                min_height: CELL_SIZE * MAZE_SIZE as f32,
            },
            ..OrthographicProjection::default_2d()
        }),
        Name::new("Camera"),
    ));

    let texture_handle: Handle<Image> = asset_server.load("kenney_topdown_shooter.png");

    let map_size = TilemapSize {
        x: MAZE_SIZE,
        y: MAZE_SIZE,
    };

    // Create a tilemap entity a little early.
    // We want this entity early because we need to tell each tile which tilemap entity
    // it is associated with. This is done with the TilemapId component on each tile.
    // Eventually, we will insert the `TilemapBundle` bundle on the entity, which
    // will contain various necessary components, such as `TileStorage`.
    let tilemap_entity = commands.spawn_empty().id();

    // To begin creating the map we will need a `TileStorage` component.
    // This component is a grid of tile entities and is used to help keep track of individual
    // tiles in the world. If you have multiple layers of tiles you would have a tilemap entity
    // per layer, each with their own `TileStorage` component.
    let mut tile_storage = TileStorage::empty(map_size);

    let maze_cache: HashMap<(usize, usize), &maze::Cell> = maze.iter().collect();

    for ((x, y), cell) in maze.iter() {
        let index = cell_to_index(cell.to_bits(), (x, y), &maze_cache);

        let tile_pos = TilePos {
            x: x as u32,
            y: map_size.y - (y as u32) - 1,
        };

        let coords = CoordsComponent::new(tile_pos.x as usize, tile_pos.y as usize);
        let tile_entity = commands
            .spawn((
                TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    texture_index: index,
                    ..default()
                },
                *cell,
            ))
            .insert_if((Start, Name::new("Start")), || coords.xy() == (0, 0))
            .insert_if((Goal, Name::new("Goal")), || {
                coords.xy() == (MAZE_SIZE as usize - 2, MAZE_SIZE as usize - 1)
            })
            .insert_if(Name::new(coords.to_string()), || {
                coords.xy() != (MAZE_SIZE as usize - 2, MAZE_SIZE as usize - 1)
                    && coords.xy() != (0, 0)
            })
            .insert(coords)
            .id();
        tile_storage.set(&tile_pos, tile_entity);
    }

    let tile_size = TilemapTileSize {
        x: CELL_SIZE,
        y: CELL_SIZE,
    };
    let grid_size = tile_size.into();
    let map_type = TilemapType::default();
    commands.insert_resource(CellSize(CELL_SIZE));

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        map_type,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(texture_handle),
        tile_size,
        anchor: TilemapAnchor::Center,
        ..default()
    });
}

// given a corner direction like 1,1,
// move on each axis and check the direction of the opposite
// axis. (ex: move on x, then check the y direction) to see
// if there's a wall, which is then used to determine if there
// should be a wall piece in the corner of the tile
// ex: a left wall with a top-right corner block.
fn check_corner(
    pos: (usize, usize),
    corner: IVec2,
    cache: &HashMap<(usize, usize), &maze::Cell>,
) -> bool {
    pos.0
        .checked_add_signed(corner.x as isize)
        .and_then(|new_x| cache.get(&(new_x, pos.1)))
        .map(|cell| {
            !cell.contains(match corner.y {
                1 => Cell::SOUTH,
                -1 => Cell::NORTH,
                _ => unreachable!(""),
            })
        })
        .is_some_and(|exists| exists)
        || pos
            .1
            .checked_add_signed(corner.y as isize)
            .and_then(|v| cache.get(&(pos.0, v)))
            .map(|cell| {
                !cell.contains(match corner.x {
                    1 => Cell::EAST,
                    -1 => Cell::WEST,
                    _ => unreachable!(""),
                })
            })
            .is_some_and(|v| v)
}

fn cell_to_index(
    cell: u8,
    position: (usize, usize),
    cache: &HashMap<(usize, usize), &maze::Cell>,
) -> TileTextureIndex {
    TileTextureIndex(
        // wesn
        match cell {
            0b0000 => 358,
            0b0001 => 286,
            0b0010 => 312,
            0b0011 => 309,
            0b0100 => 313,
            0b0101 => {
                let has_ne_corner = check_corner(position, IVec2::new(1, -1), cache);
                if has_ne_corner { 307 } else { 314 }
            }
            0b0110 => {
                let has_se_corner = check_corner(position, IVec2::new(1, 1), cache);
                if has_se_corner { 280 } else { 287 }
            }
            0b0111 => {
                let has_ne_corner = check_corner(position, IVec2::new(1, -1), cache);
                let has_se_corner = check_corner(position, IVec2::new(1, 1), cache);
                match (has_ne_corner, has_se_corner) {
                    (true, true) => 310,
                    (true, false) => 390,
                    (false, true) => 417,
                    (false, false) => 338,
                }
            }
            0b1000 => 285,
            0b1001 => {
                let has_nw_corner = check_corner(position, IVec2::new(-1, -1), cache);
                if has_nw_corner { 308 } else { 315 }
            }
            0b1010 => {
                let has_sw_corner = check_corner(position, IVec2::new(-1, 1), cache);
                if has_sw_corner { 281 } else { 288 }
            }
            0b1011 => {
                let has_nw_corner = check_corner(position, IVec2::new(-1, -1), cache);
                let has_sw_corner = check_corner(position, IVec2::new(-1, 1), cache);
                match (has_nw_corner, has_sw_corner) {
                    (true, true) => 311,
                    (true, false) => 391,
                    (false, true) => 418,
                    (false, false) => 339,
                }
            }
            0b1100 => 282,
            0b1101 => {
                let has_ne_corner = check_corner(position, IVec2::new(1, -1), cache);
                let has_nw_corner = check_corner(position, IVec2::new(-1, -1), cache);
                match (has_ne_corner, has_nw_corner) {
                    (true, true) => 284,
                    (true, false) => 420,
                    (false, true) => 419,
                    (false, false) => 366,
                }
            }
            0b1110 => {
                let has_se_corner = check_corner(position, IVec2::new(1, 1), cache);
                let has_sw_corner = check_corner(position, IVec2::new(-1, 1), cache);
                match (has_se_corner, has_sw_corner) {
                    (true, true) => 283,
                    (true, false) => 393,
                    (false, true) => 392,
                    (false, false) => 365,
                }
            }
            0b1111 => {
                let has_ne_corner = check_corner(position, IVec2::new(1, -1), cache);
                let has_nw_corner = check_corner(position, IVec2::new(-1, -1), cache);
                let has_se_corner = check_corner(position, IVec2::new(1, 1), cache);
                let has_sw_corner = check_corner(position, IVec2::new(-1, 1), cache);

                match (has_ne_corner, has_nw_corner, has_se_corner, has_sw_corner) {
                    (true, true, true, true) => 341,
                    (true, true, true, false) => 389,
                    (true, true, false, true) => 388,
                    (true, true, false, false) => 337,
                    (true, false, true, true) => 416,
                    (true, false, true, false) => 363,
                    (true, false, false, true) => 394,
                    (true, false, false, false) => 361,
                    (false, true, true, true) => 415,
                    (false, true, true, false) => 421,
                    (false, true, false, true) => 364,
                    (false, true, false, false) => 362,
                    (false, false, true, true) => 336,
                    (false, false, true, false) => 334,
                    (false, false, false, true) => 335,
                    (false, false, false, false) => 340,
                }
            } // check all corners
            _ => unreachable!("cell can only be 4 bits"),
        } - 1,
    )
}

pub(crate) fn draw_path(
    start: Query<&CoordsComponent, (With<Cell>, With<Start>)>,
    goal: Query<&CoordsComponent, (With<Cell>, With<Goal>)>,
    mut cells: Query<(&CoordsComponent, &mut TileTextureIndex), With<Cell>>,
    path: Res<MazePath>,
) {
    let Ok(_start) = start.single().cloned() else {
        return;
    };
    let Ok(_goal) = goal.single().cloned() else {
        return;
    };

    if let (true, Some((path, _cost))) = (path.is_changed() || path.is_added(), &path.path) {
        for (cell, mut index) in cells.iter_mut() {
            if path.contains(cell) {
                index.0 -= 162;
            }
        }
    }
}
