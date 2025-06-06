//! Formatters for converting a generated maze into other data types

mod ascii;
mod game_map;
mod image;

use crate::maze::grid::Grid;
use ::image::RgbImage;
use std::{fs::File, io::Write};

pub use self::image::Image;
use super::errors::MazeSaveError;
pub use ascii::{AsciiBroad, AsciiNarrow};
pub use game_map::GameMap;

/// A trait for maze formatters
pub trait Formatter<T>
where
    T: Saveable,
{
    /// Returns a given grid converted into a given type that implements [Saveable]
    fn format(&self, grid: &Grid) -> T;
}

/// A trait for data wrappers that must be returned after formatting the grid
pub trait Saveable {
    /// Saves a given object into a file
    ///
    /// In case of success, returns the string with a success message.
    /// Otherwise, returns a [`MazeSaveError`] with a custom reason message.
    ///
    /// # Errors
    /// Returns a [`MazeSaveError`] if the file could not be written
    fn save(&self, path: &str) -> Result<String, MazeSaveError>;
}

/// A custom wrapper over [`RgbImage`] for converting a maze to an image
pub struct ImageWrapper(RgbImage);

impl ImageWrapper {
    /// Consumes `self` and returns the inner [`RgbImage`].
    #[must_use]
    pub fn into_inner(self) -> RgbImage {
        self.0
    }
}

/// An implementation of [Saveable] for saving a maze image into a file
impl Saveable for ImageWrapper {
    /// Saves an image to a file to a given path
    fn save(&self, path: &str) -> Result<String, MazeSaveError> {
        if let Err(reason) = self.0.save(path) {
            return Err(MazeSaveError {
                reason: reason.to_string(),
            });
        }

        Ok(format!("Maze was successfully saved as an image: {path}"))
    }
}

/// A custom wrapper over [`std::string::String`] for converting a maze into
/// string characters
pub struct StringWrapper(pub String);

impl StringWrapper {
    /// Consumes `self` and returns the inner `String`.
    #[must_use]
    pub fn into_inner(self) -> String {
        self.0
    }
}

/// An implementation of [Saveable] for saving a maze string into a text file
impl Saveable for StringWrapper {
    /// Saves a maze string to a file to a given path
    fn save(&self, path: &str) -> Result<String, MazeSaveError> {
        let path = match std::env::current_dir() {
            Err(why) => {
                return Err(MazeSaveError {
                    reason: format!("Couldn't find path to current dir: {why}"),
                });
            }
            Ok(dir) => dir.join(path),
        };

        let mut file = match File::create(&path) {
            Err(why) => {
                return Err(MazeSaveError {
                    reason: format!("Couldn't create {}: {}", path.display(), why),
                });
            }
            Ok(file) => file,
        };

        match file.write_all(self.0.as_bytes()) {
            Err(why) => Err(MazeSaveError {
                reason: format!("Couldn't write to {}: {}", path.display(), why),
            }),
            Ok(()) => Ok(format!(
                "Maze was successfully written to a file: {}",
                path.display()
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::image::Rgb;

    #[test]
    fn into_inner_returns_inner_string() {
        let wrapper = StringWrapper(String::from("Hello, Rust!"));
        let inner = wrapper.into_inner();
        assert_eq!(inner, "Hello, Rust!");
    }

    #[test]
    fn into_inner_returns_inner_image() {
        let img = RgbImage::from_pixel(2, 2, Rgb([255, 0, 0])); // 2x2 red image
        let wrapper = ImageWrapper(img);
        let inner = wrapper.into_inner();

        assert_eq!(inner.dimensions(), (2, 2));
        assert_eq!(inner.get_pixel(0, 0), &Rgb([255, 0, 0]));
    }
}
