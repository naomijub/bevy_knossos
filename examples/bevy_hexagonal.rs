use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use hexx::{EdgeDirection, Hex, HexLayout, HexOrientation, OffsetHexMode};
use rand::{RngExt, SeedableRng, prelude::SliceRandom, rngs::StdRng};

const MAZE_WIDTH: i32 = 7;
const MAZE_HEIGHT: i32 = 8;
// Controls distance between neighboring hex centers.
const HEX_LAYOUT_SCALE: f32 = 0.72;
// Controls the GLB mesh size independently from layout spacing.
const HEX_MODEL_SCALE: f32 = 1.18;
// Calibrates GLB native orientation against logical hex directions.
// Increase/decrease by 1 to rotate all tiles by +/- 60 degrees.
const MODEL_ROTATION_OFFSET_STEPS: i32 = 1;
// Fine-grained base rotation calibration for how the GLB was authored.
// Useful when the mesh's intrinsic "forward" is offset by 30 degrees.
const MODEL_ROTATION_BASE_RADIANS: f32 = -std::f32::consts::FRAC_PI_6;

const DIRS: [EdgeDirection; 6] = [
    EdgeDirection::FLAT_NORTH_EAST,
    EdgeDirection::FLAT_NORTH,
    EdgeDirection::FLAT_NORTH_WEST,
    EdgeDirection::FLAT_SOUTH_WEST,
    EdgeDirection::FLAT_SOUTH,
    EdgeDirection::FLAT_SOUTH_EAST,
];

#[derive(Resource)]
struct HexMaze {
    cells: HashMap<Hex, u8>,
}

#[derive(Clone, Copy)]
struct TileChoice {
    name: &'static str,
    rot_steps: u8,
}

fn main() {
    App::new()
        .insert_resource(HexMaze::generate(MAZE_WIDTH, MAZE_HEIGHT, 7))
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (setup_camera_and_light, spawn_hex_maze))
        .run();
}

impl HexMaze {
    fn generate(width: i32, height: i32, seed: u64) -> Self {
        let mut cells = HashMap::<Hex, u8>::new();
        let mut all = HashSet::<Hex>::new();

        for row in 0..height {
            for col in 0..width {
                let hex = Hex::from_offset_coordinates(
                    [col, row],
                    OffsetHexMode::Odd,
                    HexOrientation::Flat,
                );
                cells.insert(hex, 0);
                all.insert(hex);
            }
        }

        let mut rng = StdRng::seed_from_u64(seed);
        let start_col = rng.random_range(0..width);
        let start_row = rng.random_range(0..height);
        let start = Hex::from_offset_coordinates(
            [start_col, start_row],
            OffsetHexMode::Odd,
            HexOrientation::Flat,
        );

        let mut visited = HashSet::<Hex>::new();
        carve_recursive(start, &all, &mut visited, &mut cells, &mut rng);

        Self { cells }
    }

    fn dead_ends(&self) -> Vec<Hex> {
        self.cells
            .iter()
            .filter_map(|(hex, mask)| mask.is_power_of_two().then_some(*hex))
            .collect()
    }
}

fn carve_recursive(
    current: Hex,
    all: &HashSet<Hex>,
    visited: &mut HashSet<Hex>,
    cells: &mut HashMap<Hex, u8>,
    rng: &mut StdRng,
) {
    visited.insert(current);

    let mut dirs = DIRS;
    dirs.shuffle(rng);

    for dir in dirs {
        let next = current.neighbor(dir);
        if !all.contains(&next) || visited.contains(&next) {
            continue;
        }

        carve(cells, current, dir);
        carve(cells, next, -dir);

        carve_recursive(next, all, visited, cells, rng);
    }
}

fn carve(cells: &mut HashMap<Hex, u8>, hex: Hex, dir: EdgeDirection) {
    if let Some(mask) = cells.get_mut(&hex) {
        let bit = direction_index(dir);
        *mask |= 1 << bit;
    }
}

fn direction_index(dir: EdgeDirection) -> u8 {
    match dir {
        EdgeDirection::FLAT_NORTH_EAST => 0,
        EdgeDirection::FLAT_NORTH => 1,
        EdgeDirection::FLAT_NORTH_WEST => 2,
        EdgeDirection::FLAT_SOUTH_WEST => 3,
        EdgeDirection::FLAT_SOUTH => 4,
        EdgeDirection::FLAT_SOUTH_EAST => 5,
        _ => unreachable!("expected flat edge direction"),
    }
}

fn setup_camera_and_light(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 28.0, 24.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            illuminance: 15_000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -1.0, 0.9, 0.0)),
    ));
}

fn spawn_hex_maze(
    mut commands: Commands,
    maze: Res<HexMaze>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let layout = HexLayout {
        orientation: HexOrientation::Flat,
        origin: Vec2::ZERO,
        scale: Vec2::splat(HEX_LAYOUT_SCALE),
    };

    let dead_ends = maze.dead_ends();
    let start_hex = dead_ends.first().copied();
    let end_hex = dead_ends.last().copied();

    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(200.0, 200.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.08, 0.11, 0.11))),
        Transform::from_xyz(0.0, -0.1, 0.0),
    ));

    for (hex, mask) in &maze.cells {
        let tile = choose_tile(*mask, *hex, start_hex, end_hex);
        let scene: Handle<Scene> = asset_server.load(format!("hexagons/{}.glb#Scene0", tile.name));

        let world = layout.hex_to_world_pos(*hex);
        // The GLBs are authored in a local orientation that needs a fixed calibration offset.
        let calibrated_steps =
            (i32::from(tile.rot_steps) + MODEL_ROTATION_OFFSET_STEPS).rem_euclid(6) as f32;
        let rot = Quat::from_rotation_y(
            MODEL_ROTATION_BASE_RADIANS + calibrated_steps * std::f32::consts::FRAC_PI_3,
        );

        commands.spawn((
            SceneRoot(scene),
            // hexx returns 2D world coords in an XY plane. In Bevy's XZ ground plane,
            // mapping y->-z preserves winding/chirality for tile orientation.
            Transform::from_translation(Vec3::new(world.x, 0.0, -world.y))
                .with_rotation(rot)
                .with_scale(Vec3::splat(HEX_MODEL_SCALE)),
            Name::new(format!("Hex {hex:?} mask {mask:06b}")),
        ));
    }
}

fn choose_tile(mask: u8, hex: Hex, start_hex: Option<Hex>, end_hex: Option<Hex>) -> TileChoice {
    let base_tiles: [(&str, u8); 13] = [
        ("river-crossing", mask_from_indices(&[0, 1, 2, 3, 4, 5])),
        ("river-straight", mask_from_indices(&[1, 4])), // north + south
        ("river-corner-sharp", mask_from_indices(&[0, 5])), // north-east + south-east
        ("river-corner", mask_from_indices(&[5, 1])),   // south-east + north
        ("river-intersectionA", mask_from_indices(&[0, 1, 2])), // north-east + north + north-west
        ("river-intersectionB", mask_from_indices(&[5, 1, 2])), // south-east + north + north-west
        ("river-intersectionC", mask_from_indices(&[0, 1, 3])), // north-east + north + south-west
        ("river-intersectionD", mask_from_indices(&[0, 1, 2, 4])), // north-east + north + north-west + south
        ("river-intersectionE", mask_from_indices(&[5, 4, 1, 2])), // south-east + south + north + north-west
        ("river-intersectionF", mask_from_indices(&[0, 4, 2])), // north-east + south + north-west
        ("river-intersectionG", mask_from_indices(&[0, 1, 2, 5, 3])), // north-east + north + north-west + south-east + south-west
        ("river-intersectionH", mask_from_indices(&[0, 1, 2, 3])), // north-east + north + north-west + south-west
        ("river-end", mask_from_indices(&[0])),
    ];

    for (name, base) in base_tiles {
        for rot_steps in 0..6 {
            if rotate_mask(base, rot_steps) == mask {
                let is_dead_end = mask.is_power_of_two();
                if is_dead_end && Some(hex) == start_hex {
                    return TileChoice {
                        name: "river-start",
                        rot_steps,
                    };
                }
                if is_dead_end && Some(hex) == end_hex {
                    return TileChoice {
                        name: "river-end",
                        rot_steps,
                    };
                }

                return TileChoice { name, rot_steps };
            }
        }
    }

    // Fallback should be unreachable for a valid maze, but keeps the example robust.
    TileChoice {
        name: "river-crossing",
        rot_steps: 0,
    }
}

const fn rotate_mask(mask: u8, steps: u8) -> u8 {
    let s = steps % 6;
    ((mask << s) | (mask >> (6 - s))) & 0b11_1111
}

const fn mask_from_indices(indices: &[u8]) -> u8 {
    let mut mask = 0u8;
    let mut i = 0usize;
    while i < indices.len() {
        mask |= 1 << indices[i];
        i += 1;
    }
    mask
}
