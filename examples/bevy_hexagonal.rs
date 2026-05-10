// TODO: Fix example
// Currently example is not correctly matching the generated paths for the maze
use std::{
    collections::{HashMap, HashSet},
    sync::OnceLock,
};

use bevy::prelude::*;
use hexx::{EdgeDirection, Hex, HexLayout, HexOrientation, OffsetHexMode};
use rand::{RngExt, SeedableRng, prelude::SliceRandom, rngs::StdRng};

const MAZE_WIDTH: i32 = 7;
const MAZE_HEIGHT: i32 = 8;
const HEX_SIDES_U8: u8 = 6;
const HEX_SIDES_I32: i32 = 6;
const HEX_MASK: u8 = 0b11_1111;
// Controls distance between neighboring hex centers.
const HEX_LAYOUT_SCALE: f32 = 0.72;
// Controls the GLB mesh size independently from layout spacing.
const HEX_MODEL_SCALE: f32 = 1.18;
// Calibrates GLB native orientation against logical hex directions.
// Increase/decrease by 1 to rotate all tiles by +/- 60 degrees.
const MODEL_ROTATION_OFFSET_STEPS: i32 = 1;
// Rotation winding between logical mask steps and rendered model orientation.
// Use 1 for same winding, -1 for opposite winding.
const MODEL_ROTATION_WINDING_SIGN: i32 = -1;
// Fine-grained base rotation calibration for how the GLB was authored.
// Useful when the mesh's intrinsic "forward" is offset by 30 degrees.
const MODEL_ROTATION_BASE_RADIANS: f32 = -std::f32::consts::FRAC_PI_6;

const HEX_EDGE_DIRECTIONS: [EdgeDirection; HEX_SIDES_U8 as usize] = [
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

#[derive(Clone, Copy)]
struct TileTemplate {
    name: &'static str,
    mask: u8,
}

impl TileTemplate {
    fn from_dirs(name: &'static str, dirs: &[EdgeDirection]) -> Self {
        Self {
            name,
            mask: mask_from_dirs(dirs),
        }
    }
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
                let hex = offset_to_hex(col, row);
                cells.insert(hex, 0);
                all.insert(hex);
            }
        }

        let mut rng = StdRng::seed_from_u64(seed);
        let start_col = rng.random_range(0..width);
        let start_row = rng.random_range(0..height);
        let start = offset_to_hex(start_col, start_row);

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

    let mut dirs = HEX_EDGE_DIRECTIONS;
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

const fn direction_index(dir: EdgeDirection) -> u8 {
    dir.index()
}

const fn offset_to_hex(col: i32, row: i32) -> Hex {
    Hex::from_offset_coordinates([col, row], OffsetHexMode::Odd, HexOrientation::Flat)
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
        let rot = tile_rotation(tile.rot_steps);

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
    for template in tile_templates() {
        for rot_steps in 0..HEX_SIDES_U8 {
            if rotate_mask(template.mask, rot_steps) == mask {
                let name =
                    dead_end_override(mask, hex, start_hex, end_hex).unwrap_or(template.name);
                let final_rot_steps =
                    (rot_steps + per_tile_rotation_offset_steps(name)) % HEX_SIDES_U8;

                return TileChoice {
                    name,
                    rot_steps: final_rot_steps,
                };
            }
        }
    }

    // Fallback should be unreachable for a valid maze, but keeps the example robust.
    TileChoice {
        name: "river-crossing",
        rot_steps: 0,
    }
}

fn tile_templates() -> &'static [TileTemplate] {
    static TEMPLATES: OnceLock<Vec<TileTemplate>> = OnceLock::new();
    TEMPLATES.get_or_init(build_tile_templates).as_slice()
}

fn build_tile_templates() -> Vec<TileTemplate> {
    vec![
        TileTemplate::from_dirs(
            "river-crossing",
            &[
                EdgeDirection::FLAT_NORTH_EAST,
                EdgeDirection::FLAT_NORTH,
                EdgeDirection::FLAT_NORTH_WEST,
                EdgeDirection::FLAT_SOUTH_WEST,
                EdgeDirection::FLAT_SOUTH,
                EdgeDirection::FLAT_SOUTH_EAST,
            ],
        ),
        TileTemplate::from_dirs(
            "river-straight",
            &[EdgeDirection::FLAT_NORTH, EdgeDirection::FLAT_SOUTH],
        ),
        TileTemplate::from_dirs(
            "river-corner-sharp",
            &[
                EdgeDirection::FLAT_NORTH_EAST,
                EdgeDirection::FLAT_SOUTH_EAST,
            ],
        ),
        TileTemplate::from_dirs(
            "river-corner",
            &[EdgeDirection::FLAT_SOUTH_EAST, EdgeDirection::FLAT_NORTH],
        ),
        TileTemplate::from_dirs(
            "river-intersectionA",
            &[
                EdgeDirection::FLAT_NORTH_EAST,
                EdgeDirection::FLAT_NORTH,
                EdgeDirection::FLAT_NORTH_WEST,
            ],
        ),
        TileTemplate::from_dirs(
            "river-intersectionB",
            &[
                EdgeDirection::FLAT_SOUTH_EAST,
                EdgeDirection::FLAT_NORTH,
                EdgeDirection::FLAT_NORTH_WEST,
            ],
        ),
        TileTemplate::from_dirs(
            "river-intersectionC",
            &[
                EdgeDirection::FLAT_NORTH_EAST,
                EdgeDirection::FLAT_NORTH,
                EdgeDirection::FLAT_SOUTH_WEST,
            ],
        ),
        TileTemplate::from_dirs(
            "river-intersectionD",
            &[
                EdgeDirection::FLAT_NORTH_EAST,
                EdgeDirection::FLAT_NORTH,
                EdgeDirection::FLAT_NORTH_WEST,
                EdgeDirection::FLAT_SOUTH,
            ],
        ),
        TileTemplate::from_dirs(
            "river-intersectionE",
            &[
                EdgeDirection::FLAT_SOUTH_EAST,
                EdgeDirection::FLAT_SOUTH,
                EdgeDirection::FLAT_NORTH,
                EdgeDirection::FLAT_NORTH_WEST,
            ],
        ),
        TileTemplate::from_dirs(
            "river-intersectionF",
            &[
                EdgeDirection::FLAT_NORTH_EAST,
                EdgeDirection::FLAT_SOUTH,
                EdgeDirection::FLAT_NORTH_WEST,
            ],
        ),
        TileTemplate::from_dirs(
            "river-intersectionG",
            &[
                EdgeDirection::FLAT_NORTH_EAST,
                EdgeDirection::FLAT_NORTH,
                EdgeDirection::FLAT_NORTH_WEST,
                EdgeDirection::FLAT_SOUTH_EAST,
                EdgeDirection::FLAT_SOUTH_WEST,
            ],
        ),
        TileTemplate::from_dirs(
            "river-intersectionH",
            &[
                EdgeDirection::FLAT_NORTH_EAST,
                EdgeDirection::FLAT_NORTH,
                EdgeDirection::FLAT_NORTH_WEST,
                EdgeDirection::FLAT_SOUTH_WEST,
            ],
        ),
        TileTemplate::from_dirs("river-end", &[EdgeDirection::FLAT_NORTH_EAST]),
    ]
}

fn dead_end_override(
    mask: u8,
    hex: Hex,
    start_hex: Option<Hex>,
    end_hex: Option<Hex>,
) -> Option<&'static str> {
    if !mask.is_power_of_two() {
        return None;
    }

    if Some(hex) == start_hex {
        Some("river-start")
    } else if Some(hex) == end_hex {
        Some("river-end")
    } else {
        None
    }
}

fn tile_rotation(rot_steps: u8) -> Quat {
    // The GLBs are authored in a local orientation that needs a fixed calibration offset.
    let calibrated_steps = (MODEL_ROTATION_WINDING_SIGN * i32::from(rot_steps)
        + MODEL_ROTATION_OFFSET_STEPS)
        .rem_euclid(HEX_SIDES_I32) as f32;
    Quat::from_rotation_y(
        calibrated_steps.mul_add(std::f32::consts::FRAC_PI_3, MODEL_ROTATION_BASE_RADIANS),
    )
}

const fn rotate_mask(mask: u8, steps: u8) -> u8 {
    let s = steps % HEX_SIDES_U8;
    ((mask << s) | (mask >> (HEX_SIDES_U8 - s))) & HEX_MASK
}

fn mask_from_dirs(dirs: &[EdgeDirection]) -> u8 {
    let mut mask = 0u8;
    for dir in dirs {
        mask |= 1 << dir.index();
    }
    mask
}

// Per-model rotation calibration (in 60-degree steps) so rendered river exits
// align with logical cell directions.
fn per_tile_rotation_offset_steps(name: &str) -> u8 {
    match name {
        // Corner families are authored one step offset from logical direction masks.
        // These intersection variants share the same local orientation convention as corners.
        "river-corner"
        | "river-corner-sharp"
        | "river-intersectionB"
        | "river-intersectionC"
        | "river-intersectionF"
        | "river-intersectionH"
        | "river-end"
        | "river-start" => 1,
        _ => 0,
    }
}
