use assert_cmd::cargo::cargo_bin_cmd;
use assert_fs::fixture::TempDir;

#[test]
fn image_save_success() {
    let output_dir = TempDir::new().unwrap();
    let file_path = format!("{}/maze.png", output_dir.path().display());
    let expected = format!("Maze was successfully saved as an image: {}\n", &file_path);

    let mut cmd = cargo_bin_cmd!();
    cmd.args(["generate", "image", "--output-path", &file_path])
        .assert()
        .success()
        .stdout(expected);
}

#[test]
fn ascii_save_success() {
    let output_dir = TempDir::new().unwrap();
    let file_path = format!("{}/maze.txt", output_dir.path().display());
    let expected = format!("Maze was successfully written to a file: {file_path}\n");

    let mut cmd = cargo_bin_cmd!();
    cmd.args(["generate", "ascii", "--output-path", &file_path])
        .assert()
        .success()
        .stdout(expected);
}

#[test]
fn game_map_save_success() {
    let output_dir = TempDir::new().unwrap();
    let file_path = format!("{}/maze.txt", output_dir.path().display());
    let expected = format!("Maze was successfully written to a file: {file_path}\n");

    let mut cmd = cargo_bin_cmd!();
    cmd.args(["generate", "game-map", "--output-path", &file_path])
        .assert()
        .success()
        .stdout(expected);
}
