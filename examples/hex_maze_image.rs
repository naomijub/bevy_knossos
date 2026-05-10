use std::fs;

use bevy::math::Vec2;
use bevy_knossos::maze::{Cell, HexMazeBuilder, RecursiveBacktracking};
use hexx::{EdgeDirection, Hex, HexLayout, HexOrientation, OffsetHexMode};
use image::{Rgb, RgbImage};

const WIDTH: usize = 24;
const HEIGHT: usize = 16;
const HEX_SCALE: f32 = 18.0;
const MARGIN: i32 = 24;
const PATH_THICKNESS: i32 = 6;
const NODE_RADIUS: i32 = 5;

fn main() {
    fs::create_dir_all("output").expect("output directory should be creatable");

    let maze = HexMazeBuilder::new()
        .width(WIDTH)
        .height(HEIGHT)
        .seed(2026)
        .algorithm(Box::new(RecursiveBacktracking))
        .build()
        .expect("hex maze should build");

    let layout = HexLayout {
        orientation: HexOrientation::Flat,
        origin: Vec2::ZERO,
        scale: Vec2::splat(HEX_SCALE),
    };

    let mut min_x = f32::MAX;
    let mut max_x = f32::MIN;
    let mut min_y = f32::MAX;
    let mut max_y = f32::MIN;

    for (coord, _) in maze.iter() {
        let hex = offset_to_hex(coord);
        let pos = layout.hex_to_world_pos(hex);
        min_x = min_x.min(pos.x);
        max_x = max_x.max(pos.x);
        min_y = min_y.min(pos.y);
        max_y = max_y.max(pos.y);
    }

    let image_width = ((max_x - min_x).ceil() as i32 + MARGIN * 2).max(1) as u32;
    let image_height = ((max_y - min_y).ceil() as i32 + MARGIN * 2).max(1) as u32;
    let mut image = RgbImage::from_pixel(image_width, image_height, Rgb([18, 30, 28]));

    let path_color = Rgb([98, 173, 153]);

    for (coord, cell) in maze.iter() {
        let current_hex = offset_to_hex(coord);
        let current_center = layout.hex_to_world_pos(current_hex);
        let current_px = world_to_pixel(current_center, min_x, min_y);

        draw_filled_circle(
            &mut image,
            current_px.0,
            current_px.1,
            NODE_RADIUS,
            path_color,
        );

        for dir in open_hex_dirs(cell) {
            let neighbor_hex = current_hex.neighbor(dir);
            let neighbor_center = layout.hex_to_world_pos(neighbor_hex);
            let neighbor_px = world_to_pixel(neighbor_center, min_x, min_y);

            draw_thick_line(
                &mut image,
                current_px.0,
                current_px.1,
                neighbor_px.0,
                neighbor_px.1,
                PATH_THICKNESS,
                path_color,
            );
        }
    }

    let path = "output/hex_maze_image_example.png";
    image.save(path).expect("hex maze image should save");

    println!("Saved hex maze image to {path}");
}

#[expect(clippy::cast_possible_wrap)]
const fn offset_to_hex(coord: (usize, usize)) -> Hex {
    Hex::from_offset_coordinates(
        [coord.0 as i32, coord.1 as i32],
        OffsetHexMode::Odd,
        HexOrientation::Flat,
    )
}

fn open_hex_dirs(cell: &Cell) -> impl Iterator<Item = EdgeDirection> {
    let mut dirs = Vec::with_capacity(6);

    if cell.contains(Cell::EAST) {
        dirs.push(EdgeDirection::FLAT_SOUTH_EAST);
    }
    if cell.contains(Cell::SOUTH_EAST) {
        dirs.push(EdgeDirection::FLAT_SOUTH);
    }
    if cell.contains(Cell::SOUTH_WEST) {
        dirs.push(EdgeDirection::FLAT_SOUTH_WEST);
    }
    if cell.contains(Cell::WEST) {
        dirs.push(EdgeDirection::FLAT_NORTH_WEST);
    }
    if cell.contains(Cell::NORTH_WEST) {
        dirs.push(EdgeDirection::FLAT_NORTH);
    }
    if cell.contains(Cell::NORTH_EAST) {
        dirs.push(EdgeDirection::FLAT_NORTH_EAST);
    }

    dirs.into_iter()
}

fn world_to_pixel(pos: Vec2, min_x: f32, min_y: f32) -> (i32, i32) {
    let x = (pos.x - min_x).round() as i32 + MARGIN;
    let y = (pos.y - min_y).round() as i32 + MARGIN;
    (x, y)
}

fn draw_thick_line(
    image: &mut RgbImage,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    thickness: i32,
    color: Rgb<u8>,
) {
    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let steps = dx.max(dy).max(1);

    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let x = ((x1 - x0) as f32).mul_add(t, x0 as f32);
        let y = ((y1 - y0) as f32).mul_add(t, y0 as f32);
        draw_filled_circle(image, x.round() as i32, y.round() as i32, thickness / 2, color);
    }
}

fn draw_filled_circle(image: &mut RgbImage, cx: i32, cy: i32, radius: i32, color: Rgb<u8>) {
    for y in (cy - radius)..=(cy + radius) {
        for x in (cx - radius)..=(cx + radius) {
            let dx = x - cx;
            let dy = y - cy;
            if dx * dx + dy * dy <= radius * radius {
                put_pixel_safe(image, x, y, color);
            }
        }
    }
}

fn put_pixel_safe(image: &mut RgbImage, x: i32, y: i32, color: Rgb<u8>) {
    if x < 0 || y < 0 {
        return;
    }
    let ux = x as u32;
    let uy = y as u32;
    if ux < image.width() && uy < image.height() {
        image.put_pixel(ux, uy, color);
    }
}
