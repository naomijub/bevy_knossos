use std::fmt::Write;
use std::fs;

use bevy_knossos::maze::{Cell, HexMaze, HexMazeBuilder, HexText, RecursiveBacktracking};

const WIDTH: usize = 10;
const HEIGHT: usize = 8;

fn main() {
    fs::create_dir_all("output").expect("output directory should be creatable");

    let maze = HexMazeBuilder::new()
        .width(WIDTH)
        .height(HEIGHT)
        .seed(2026)
        .algorithm(Box::new(RecursiveBacktracking))
        .build()
        .expect("hex maze should build");

    maze.save("output/hex_maze.txt", HexText)
        .expect("hex maze text should save");

    let text =
        fs::read_to_string("output/hex_maze.txt").expect("hex maze text file should be readable");

    let restored = HexMaze::from_text(&text).expect("hex maze text should deserialize");

    let original_cells: Vec<_> = maze
        .iter()
        .map(|(coord, cell)| (coord, cell.to_bits()))
        .collect();
    let restored_cells: Vec<_> = restored
        .iter()
        .map(|(coord, cell)| (coord, cell.to_bits()))
        .collect();

    assert_eq!(original_cells, restored_cells);

    let pretty = to_human_readable(&maze, WIDTH, HEIGHT);
    fs::write("output/hex_maze_readable.txt", pretty)
        .expect("human-readable hex maze text should save");
    let ascii = to_hex_ascii_art(&maze, WIDTH, HEIGHT);
    fs::write("output/hex_maze_ascii.txt", ascii).expect("hex ascii art should save");

    println!(
        "Hex text roundtrip complete. Saved machine format to output/hex_maze.txt, readable format to output/hex_maze_readable.txt, ascii drawing to output/hex_maze_ascii.txt, and restored {} cells.",
        restored_cells.len()
    );
}

fn to_human_readable(maze: &HexMaze, width: usize, height: usize) -> String {
    let mut output = String::new();
    output.push_str("KNOSSOS_HEX_READABLE_V1\n");
    output.push_str("topology=hex-odd-r\n");
    let _ = writeln!(output, "width={width}");
    let _ = writeln!(output, "height={height}");
    output.push_str("legend: token is HEX_MASK:DIRECTIONS\n");
    output.push_str("directions: E W NE NW SE SW\n\n");

    for y in 0..height {
        let _ = write!(output, "row {y:02}: ");
        for x in 0..width {
            if x > 0 {
                output.push_str("  ");
            }
            let cell = maze[(x, y)];
            let dirs = cell_dirs(cell);
            let _ = write!(output, "{:02X}:{}", cell.to_bits(), dirs);
        }
        output.push('\n');
    }

    output
}

fn cell_dirs(cell: Cell) -> String {
    let mut dirs = Vec::new();
    if cell.contains(Cell::EAST) {
        dirs.push("E");
    }
    if cell.contains(Cell::WEST) {
        dirs.push("W");
    }
    if cell.contains(Cell::NORTH_EAST) {
        dirs.push("NE");
    }
    if cell.contains(Cell::NORTH_WEST) {
        dirs.push("NW");
    }
    if cell.contains(Cell::SOUTH_EAST) {
        dirs.push("SE");
    }
    if cell.contains(Cell::SOUTH_WEST) {
        dirs.push("SW");
    }

    if dirs.is_empty() {
        "BLOCKED".to_string()
    } else {
        dirs.join("|")
    }
}

fn to_hex_ascii_art(maze: &HexMaze, width: usize, height: usize) -> String {
    let canvas_width = width * 4 + 6;
    let canvas_height = height * 2 + 3;
    let mut canvas = vec![vec![' '; canvas_width]; canvas_height];

    for y in 0..height {
        #[expect(clippy::cast_possible_wrap)]
        for x in 0..width {
            let cell = maze[(x, y)];
            let cx = x as i32 * 4 + if y % 2 == 1 { 2 } else { 0 } + 2;
            let cy = y as i32 * 2 + 1;

            put_char(&mut canvas, cx, cy, 'o');

            if cell.contains(Cell::EAST) {
                put_char(&mut canvas, cx + 1, cy, '-');
                put_char(&mut canvas, cx + 2, cy, '-');
            }
            if cell.contains(Cell::WEST) {
                put_char(&mut canvas, cx - 1, cy, '-');
                put_char(&mut canvas, cx - 2, cy, '-');
            }
            if cell.contains(Cell::NORTH_EAST) {
                put_char(&mut canvas, cx + 1, cy - 1, '/');
            }
            if cell.contains(Cell::NORTH_WEST) {
                put_char(&mut canvas, cx - 1, cy - 1, '\\');
            }
            if cell.contains(Cell::SOUTH_EAST) {
                put_char(&mut canvas, cx + 1, cy + 1, '\\');
            }
            if cell.contains(Cell::SOUTH_WEST) {
                put_char(&mut canvas, cx - 1, cy + 1, '/');
            }
        }
    }

    let mut output = String::new();
    output.push_str("KNOSSOS_HEX_ASCII_V1\n");
    output.push_str(
        "legend: o=cell, -=east/west passage, /=ne or sw passage, \\\\=nw or se passage\n\n",
    );

    for row in canvas {
        let line: String = row.into_iter().collect();
        output.push_str(line.trim_end());
        output.push('\n');
    }

    output
}

fn put_char(canvas: &mut [Vec<char>], x: i32, y: i32, ch: char) {
    if x < 0 || y < 0 {
        return;
    }
    let ux = x as usize;
    let uy = y as usize;
    if let Some(row) = canvas.get_mut(uy)
        && let Some(cell) = row.get_mut(ux)
    {
        *cell = ch;
    }
}
