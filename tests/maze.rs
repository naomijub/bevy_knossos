use assert_fs::fixture::TempDir;
use bevy_knossos::maze::*;

macro_rules! maze {
    ($algo:expr_2021) => {
        OrthogonalMazeBuilder::new()
            .algorithm(Box::new($algo))
            .build()
    };

    () => {
        OrthogonalMazeBuilder::new().build()
    };
}

#[test]
fn build_valid_maze_with_default_params() {
    let maze = OrthogonalMazeBuilder::new().build().unwrap();
    assert!(maze.is_valid());
}

#[test]
fn build_valid_maze_with_custom_params() {
    let maze = OrthogonalMazeBuilder::new()
        .height(10)
        .width(20)
        .algorithm(Box::new(Kruskal))
        .build()
        .unwrap();

    assert!(maze.is_valid());
}

#[test]
fn build_valid_maze_with_seed_value() {
    let maze = OrthogonalMazeBuilder::new().seed(40).build().unwrap();
    assert!(maze.is_valid());
}

#[test]
fn build_identical_mazes_with_same_seed() {
    let old = OrthogonalMazeBuilder::new().seed(40).build().unwrap();
    let new = OrthogonalMazeBuilder::new().seed(40).build().unwrap();
    assert_eq!(old.to_string(), new.to_string());
}

#[test]
fn build_valid_maze_with_custom_params_and_start_position() {
    let maze = OrthogonalMazeBuilder::new()
        .height(10)
        .width(20)
        .start_coords((3, 6))
        .algorithm(Box::new(Prim::new()))
        .build()
        .unwrap();

    assert!(maze.is_valid());
}

#[test]
fn build_valid_maze_with_aldou_broder_algorithm() {
    assert!(maze!(AldousBroder).unwrap().is_valid());
}

#[test]
fn build_valid_maze_with_binary_tree_algorithm() {
    assert!(maze!(BinaryTree::new(Bias::NorthWest)).unwrap().is_valid());
    assert!(maze!(BinaryTree::new(Bias::SouthWest)).unwrap().is_valid());
    assert!(maze!(BinaryTree::new(Bias::NorthEast)).unwrap().is_valid());
    assert!(maze!(BinaryTree::new(Bias::SouthEast)).unwrap().is_valid());
}

#[test]
fn build_valid_maze_with_eller_algorithm() {
    assert!(maze!(Eller).unwrap().is_valid());
}

#[test]
fn build_valid_maze_with_growing_tree_algorithm() {
    assert!(maze!(GrowingTree::new(Method::Newest)).unwrap().is_valid());
    assert!(maze!(GrowingTree::new(Method::Oldest)).unwrap().is_valid());
    assert!(maze!(GrowingTree::new(Method::Middle)).unwrap().is_valid());
    assert!(maze!(GrowingTree::new(Method::Random)).unwrap().is_valid());
    assert!(
        maze!(GrowingTree::new(Method::Newest25Random75))
            .unwrap()
            .is_valid()
    );
    assert!(
        maze!(GrowingTree::new(Method::Newest50Random50))
            .unwrap()
            .is_valid()
    );
    assert!(
        maze!(GrowingTree::new(Method::Newest75Random25))
            .unwrap()
            .is_valid()
    );
}

#[test]
fn build_valid_maze_with_hunt_and_kill_algorithm() {
    assert!(maze!(HuntAndKill::new()).unwrap().is_valid());
}

#[test]
fn build_valid_maze_with_kruskal_algorithm() {
    assert!(maze!(Kruskal).unwrap().is_valid());
}

#[test]
fn build_valid_maze_with_prim_algorithm() {
    assert!(maze!(Prim::new()).unwrap().is_valid());
}

#[test]
fn build_valid_maze_with_recursive_backtracking_algorithm() {
    assert!(maze!(RecursiveBacktracking).unwrap().is_valid());
}

#[test]
fn build_valid_maze_with_recursive_division_algorithm() {
    assert!(maze!(RecursiveDivision).unwrap().is_valid());
}

#[test]
fn build_valid_maze_with_sidewinder_algorithm() {
    assert!(maze!(Sidewinder).unwrap().is_valid());
}

fn assert_maze_consistency(maze: &OrthogonalMaze) {
    let nodes = maze.iter().count();
    let width = maze.iter().map(|((x, _), _)| x).max().map_or(0, |x| x + 1);
    let height = maze.iter().map(|((_, y), _)| y).max().map_or(0, |y| y + 1);

    assert!(maze.is_valid());
    assert_eq!(nodes, width * height);

    for ((x, y), cell) in maze.iter() {
        if x + 1 < width {
            assert_eq!(
                cell.contains(Cell::EAST),
                maze[(x + 1, y)].contains(Cell::WEST)
            );
        }
        if y + 1 < height {
            assert_eq!(
                cell.contains(Cell::SOUTH),
                maze[(x, y + 1)].contains(Cell::NORTH)
            );
        }
    }
}

#[test]
fn generated_mazes_are_consistent_across_algorithms_and_seeds() {
    let algorithms: [(&str, fn() -> Box<dyn Algorithm>); 15] = [
        ("AldousBroder", || Box::new(AldousBroder)),
        ("BinaryTree::NorthWest", || {
            Box::new(BinaryTree::new(Bias::NorthWest))
        }),
        ("BinaryTree::SouthWest", || {
            Box::new(BinaryTree::new(Bias::SouthWest))
        }),
        ("BinaryTree::NorthEast", || {
            Box::new(BinaryTree::new(Bias::NorthEast))
        }),
        ("BinaryTree::SouthEast", || {
            Box::new(BinaryTree::new(Bias::SouthEast))
        }),
        ("Eller", || Box::new(Eller)),
        ("GrowingTree::Newest", || {
            Box::new(GrowingTree::new(Method::Newest))
        }),
        ("GrowingTree::Oldest", || {
            Box::new(GrowingTree::new(Method::Oldest))
        }),
        ("GrowingTree::Middle", || {
            Box::new(GrowingTree::new(Method::Middle))
        }),
        ("GrowingTree::Random", || {
            Box::new(GrowingTree::new(Method::Random))
        }),
        ("HuntAndKill", || Box::new(HuntAndKill::new())),
        ("Kruskal", || Box::new(Kruskal)),
        ("Prim", || Box::new(Prim::new())),
        ("RecursiveBacktracking", || Box::new(RecursiveBacktracking)),
        ("Sidewinder", || Box::new(Sidewinder)),
    ];
    let seeds = [0_u64, 1, 7, 19, 42, 99];
    let sizes = [(2, 2), (3, 5), (8, 8), (12, 7)];

    for (_name, algorithm) in algorithms {
        for seed in seeds {
            for (width, height) in sizes {
                let maze = OrthogonalMazeBuilder::new()
                    .width(width)
                    .height(height)
                    .seed(seed)
                    .algorithm(algorithm())
                    .build()
                    .unwrap();
                assert_maze_consistency(&maze);
            }
        }
    }
}

macro_rules! to_absolute_path {
    ($path:expr_2021) => {
        std::env::current_dir().unwrap().join($path).display()
    };
}

macro_rules! assert_save_maze {
    ($path:expr_2021, $formatter:expr_2021, $expected:expr_2021) => {
        let maze = maze!().unwrap();
        let result = maze.save($path, $formatter);
        assert_eq!($expected, result.unwrap());
    };
}

macro_rules! assert_save_maze_error {
    ($path:expr_2021, $formatter:expr_2021, $expected:expr_2021) => {
        let maze = maze!().unwrap();
        let result = maze.save($path, $formatter);
        assert_eq!($expected, result.unwrap_err().reason);
    };
}

#[test]
fn save_maze_as_ascii() {
    let output_dir = TempDir::new().unwrap();
    let file_path = format!("{}/maze.png", output_dir.path().display());
    let expected = format!(
        "Maze was successfully written to a file: {}",
        to_absolute_path!(&file_path)
    );
    assert_save_maze!(&file_path, AsciiNarrow, expected);
}

#[test]
#[cfg(target_os = "linux")]
fn save_maze_as_ascii_returns_error() {
    let expected = format!(
        "Couldn't create {}: Is a directory (os error 21)",
        to_absolute_path!("this is not valid path/")
    );

    assert_save_maze_error!("this is not valid path/", AsciiNarrow, expected);
}

#[test]
#[cfg(target_os = "macos")]
fn save_maze_as_ascii_returns_error() {
    let expected = format!(
        "Couldn't create {}: No such file or directory (os error 2)",
        to_absolute_path!("this is not valid path/")
    );

    assert_save_maze_error!("this is not valid path/", AsciiNarrow, expected);
}

#[test]
fn save_maze_as_game_map() {
    let output_dir = TempDir::new().unwrap();
    let file_path = format!("{}/maze.png", output_dir.path().display());
    let expected = format!(
        "Maze was successfully written to a file: {}",
        to_absolute_path!(&file_path)
    );
    assert_save_maze!(&file_path, GameMap::new(), expected);
}

#[test]
#[cfg(target_os = "linux")]
fn save_maze_as_game_map_returns_error() {
    let expected = format!(
        "Couldn't create {}: Is a directory (os error 21)",
        to_absolute_path!("this is not valid path/")
    );

    assert_save_maze_error!("this is not valid path/", GameMap::new(), expected);
}

#[test]
#[cfg(target_os = "macos")]
fn save_maze_as_game_map_returns_error() {
    let expected = format!(
        "Couldn't create {}: No such file or directory (os error 2)",
        to_absolute_path!("this is not valid path/")
    );

    assert_save_maze_error!("this is not valid path/", GameMap::new(), expected);
}

#[test]
fn save_maze_as_png() {
    let output_dir = TempDir::new().unwrap();
    let file_path = format!("{}/maze.png", output_dir.path().display());
    let expected = format!("Maze was successfully saved as an image: {}", &file_path);
    assert_save_maze!(&file_path, Image::new(), expected);
}

#[test]
fn save_maze_as_png_returns_error() {
    let expected = "The image format could not be determined".to_string();
    assert_save_maze_error!("this is not valid path/", Image::new(), expected);
}

#[test]
fn format_maze() {
    let ascii = OrthogonalMazeBuilder::new()
        .width(5)
        .height(5)
        .seed(10)
        .build()
        .unwrap()
        .format(AsciiBroad)
        .into_inner();

    let mut expected = String::new();
    expected.push_str("+---+---+---+---+---+\n");
    expected.push_str("|       |           |\n");
    expected.push_str("+---+   +---+   +   +\n");
    expected.push_str("|   |       |   |   |\n");
    expected.push_str("+   +---+   +   +---+\n");
    expected.push_str("|   |       |       |\n");
    expected.push_str("+   +   +---+---+   +\n");
    expected.push_str("|   |   |       |   |\n");
    expected.push_str("+   +   +   +   +   +\n");
    expected.push_str("|           |       |\n");
    expected.push_str("+---+---+---+---+---+\n");

    assert_eq!(expected, ascii);
}
