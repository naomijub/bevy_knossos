use std::fmt;
use std::str::FromStr;

use bevy_knossos::Color;
use bevy_knossos::maze::{self, MazeSaveError, formatters};
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Algorithm {
    AldousBroder,
    BinaryTree,
    Eller,
    GrowingTree,
    HuntAndKill,
    Kruskal,
    Prim,
    RecursiveBacktracking,
    RecursiveDivision,
    Sidewinder,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum AsciiOutputType {
    Narrow,
    Broad,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Coords(usize, usize);

impl fmt::Display for Coords {
    /// Writes a formatted maze into a buffer
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({},{})", self.0, self.1)?;
        Ok(())
    }
}

impl FromStr for Coords {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        let s = s.trim_end_matches(')').trim_start_matches('(');
        let Some((x, y)) = s.split_once(',') else {
            return Err("Start coord should follow the pattern `(0, 0)`".to_string());
        };

        Ok(Coords(
            x.parse::<usize>().map_err(|err| err.to_string())?,
            y.parse::<usize>().map_err(|err| err.to_string())?,
        ))
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Generates a maze
    Generate {
        #[command(subcommand)]
        output: OutputCommands,

        /// Maze generation algorithm
        #[arg(short = 'A', long, value_enum, default_value_t = Algorithm::RecursiveBacktracking)]
        algorithm: Algorithm,

        /// Grid height in a number of cells
        #[arg(short = 'H', long, default_value_t = 10)]
        height: usize,

        /// Seed value for deterministic generation (must be a valid u64)
        #[arg(short = 'S', long)]
        seed: Option<u64>,

        #[arg(short = 'W', long, default_value_t = 10)]
        /// Grid width in a number of cells
        width: usize,

        #[arg(short = 'C', long, default_value = None)]
        /// Start coordinate for maze algorithm
        start_coords: Option<Coords>,

        /// Bias to use for the "Binary Tree" algorithm
        #[arg(
            long,
            default_value_t = maze::Bias::NorthEast,
            require_equals = true,
            num_args = 0..=1,
            default_missing_value = "north-east",
            value_enum,
        )]
        bias: maze::Bias,

        /// Growing method to use for the "Growing Tree" algorithm
        #[arg(
            long,
            default_value_t = maze::Method::Newest,
            require_equals = true,
            num_args = 0..=1,
            default_missing_value = "newest",
            value_enum,
        )]
        growing_method: maze::Method,
    },
}

#[derive(Debug, Subcommand)]
enum OutputCommands {
    /// Save to a text file with an ASCII representation of a maze
    Ascii {
        /// Output path
        #[arg(short = 'O', long)]
        output_path: String,

        /// Output type
        #[arg(
            short = 'T',
            long,
            value_enum,
            default_value_t = AsciiOutputType::Narrow,
            require_equals = true,
            num_args = 0..=1,
            default_missing_value = "narrow",
        )]
        output_type: AsciiOutputType,
    },
    /// Save to a text file as an ASCII game map for pseudo 3D games that use ray casting
    /// for modeling and rendering the map
    GameMap {
        /// Output path
        #[arg(short = 'O', long)]
        output_path: String,

        /// Distance between any two walls
        #[arg(long, default_value_t = 3)]
        span: usize,

        /// ASCII character for a passage
        #[arg(long, default_value_t = '.')]
        passage: char,

        /// ASCII character for a wall
        #[arg(long, default_value_t = '#')]
        wall: char,

        /// With start "S" and goal "G" points randomly spawned on the borders
        #[arg(long, default_value_t = false)]
        with_start_goal: bool,
    },
    /// Save to PNG or JPG file
    Image {
        /// Output path
        #[arg(short = 'O', long)]
        output_path: String,

        /// Wall size in pixels
        #[arg(long = "wall-size", default_value_t = 40)]
        wall_size: usize,

        /// Passage size in pixels
        #[arg(long = "passage-size", default_value_t = 40)]
        passage_size: usize,

        /// Size of the margin area that implies an empty space between an image borders and grid
        #[arg(long, default_value_t = 50)]
        margin: usize,

        /// Color of passages
        #[arg(long = "passage-color", default_value = "#ffffff", value_parser = hex_to_rgb)]
        passage_color: Color,

        /// Color of walls
        #[arg(long = "wall-color", default_value = "#000000", value_parser = hex_to_rgb)]
        wall_color: Color,
    },
}

fn main() -> Result<(), maze::MazeSaveError> {
    let args = Cli::parse();

    match args.command {
        Commands::Generate {
            output,
            algorithm,
            height,
            width,
            seed,
            bias,
            growing_method,
            start_coords,
        } => {
            let algorithm: Box<dyn maze::Algorithm> = match algorithm {
                Algorithm::AldousBroder => Box::new(maze::AldousBroder),
                Algorithm::BinaryTree => Box::new(maze::BinaryTree::new(bias)),
                Algorithm::Eller => Box::new(maze::Eller),
                Algorithm::GrowingTree => Box::new(maze::GrowingTree::new(growing_method)),
                Algorithm::HuntAndKill => Box::new(maze::HuntAndKill::new()),
                Algorithm::Kruskal => Box::new(maze::Kruskal),
                Algorithm::Prim => Box::new(maze::Prim::new()),
                Algorithm::RecursiveBacktracking => Box::new(maze::RecursiveBacktracking),
                Algorithm::RecursiveDivision => Box::new(maze::RecursiveDivision),
                Algorithm::Sidewinder => Box::new(maze::Sidewinder),
            };

            let maze = start_coords
                .map_or_else(maze::OrthogonalMazeBuilder::new, |coords| {
                    maze::OrthogonalMazeBuilder::new().start_coords((coords.0, coords.1))
                })
                .height(height)
                .width(width)
                .seed(seed)
                .algorithm(algorithm)
                .build()
                .map_err(|err| MazeSaveError::reason(err.to_string()))?;

            let result;

            match output {
                OutputCommands::Ascii {
                    output_path,
                    output_type,
                } => match output_type {
                    AsciiOutputType::Narrow => {
                        result = maze.save(output_path.as_str(), formatters::AsciiNarrow);
                    }
                    AsciiOutputType::Broad => {
                        result = maze.save(output_path.as_str(), formatters::AsciiBroad);
                    }
                },
                OutputCommands::GameMap {
                    output_path,
                    span,
                    passage,
                    wall,
                    with_start_goal,
                } => {
                    result = if with_start_goal {
                        maze.save(
                            output_path.as_str(),
                            maze::GameMap::new()
                                .span(span)
                                .passage(passage)
                                .wall(wall)
                                .with_start_goal(),
                        )
                    } else {
                        maze.save(
                            output_path.as_str(),
                            maze::GameMap::new().span(span).passage(passage).wall(wall),
                        )
                    };
                }
                OutputCommands::Image {
                    output_path,
                    wall_size,
                    passage_size,
                    margin,
                    passage_color,
                    wall_color,
                } => {
                    result = maze.save(
                        output_path.as_str(),
                        maze::Image::new()
                            .wall(wall_size)
                            .passage(passage_size)
                            .margin(margin)
                            .background(passage_color)
                            .foreground(wall_color),
                    );
                }
            }

            match result {
                Ok(msg) => {
                    println!("{msg}");
                    Ok(())
                }
                Err(err) => Err(err),
            }
        }
    }
}

fn hex_to_rgb(s: &str) -> Result<Color, ParseHexError> {
    let s = s.strip_prefix('#').map_or(s, |hex| hex);

    if s.len() != 6 {
        return Err(ParseHexError::Length(s.to_string()));
    }

    Ok(Color::RGB(
        u8::from_str_radix(&s[..2], 16)?,
        u8::from_str_radix(&s[2..4], 16)?,
        u8::from_str_radix(&s[4..6], 16)?,
    ))
}

#[derive(Debug)]
enum ParseHexError {
    IntError(std::num::ParseIntError),
    Length(String),
}

impl std::error::Error for ParseHexError {}

impl std::fmt::Display for ParseHexError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParseHexError::Length(e) => write!(
                f,
                "Expected a 6 character color value in hex, but got: {e:?}"
            ),
            ParseHexError::IntError(e) => e.fmt(f),
        }
    }
}

impl From<std::num::ParseIntError> for ParseHexError {
    fn from(err: std::num::ParseIntError) -> ParseHexError {
        ParseHexError::IntError(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }
}
