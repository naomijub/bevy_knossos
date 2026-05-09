use std::fmt;

use bevy::{ecs::component::Component, reflect::Reflect};
use bitflags::bitflags;

bitflags! {
    ///Maze Cell defining open passages
    #[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Component, Reflect)]
    #[reflect(opaque)]
    pub struct Cell: u8 {
        /// Has passage to NORTH
        const NORTH = 0b0001;
        /// Has passage to SOUTH
        const SOUTH = 0b0010;
        /// Has passage to EAST
        const EAST =  0b0100;
        /// Has passage to WEST
        const WEST =  0b1000;
    ///Has passage to NORTH_EAST (hex)
        const NORTH_EAST = 0b0001_0000;
    ///Has passage to NORTH_WEST (hex)
        const NORTH_WEST = 0b0010_0000;
    ///Has passage to SOUTH_EAST (hex)
        const SOUTH_EAST = 0b0100_0000;
    ///Has passage to SOUTH_WEST (hex)
        const SOUTH_WEST = 0b1000_0000;
    }
}

impl Cell {
    ///Returns bits &str representation.
    /// > use `to_bits_string` for string value
    #[must_use]
    pub fn to_bits_str(&self) -> &'static str {
        let bits = format!("{:0>4b}", self.bits());
        bits.leak()
    }

    ///Returns bits string representation.
    #[must_use]
    pub fn to_bits_string(&self) -> String {
        format!("{:0>4b}", self.bits())
    }

    ///Returns bits u8 representation.
    ///> analogous to `bits`.
    #[must_use]
    pub const fn to_bits(&self) -> u8 {
        self.bits()
    }

    ///Amount of walls present: [0..=4].
    #[must_use]
    pub const fn walls_count_sq(&self) -> u8 {
        self.walls_count_for(4)
    }

    ///Amount of walls present: [0..=6].
    #[must_use]
    pub const fn walls_count_hex(&self) -> u8 {
        self.walls_count_for(6)
    }

    /// Amount of walls present for an arbitrary-sided cell.
    #[must_use]
    pub const fn walls_count_for(&self, sides: u8) -> u8 {
        sides.saturating_sub(self.bits().count_ones() as u8)
    }

    ///Checks if [`Cell`] has 3 walls (*Dead End*)
    #[must_use]
    pub const fn is_end_sq(&self) -> bool {
        self.walls_count_sq() == 3
    }

    ///Checks if [`Cell`] has 5 walls (*Dead End*)
    #[must_use]
    pub const fn is_end_hex(&self) -> bool {
        self.walls_count_sq() == 3
    }
}

impl fmt::Display for Cell {
    ///Writes a formatted maze into a buffer
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let names = self
            .iter_names()
            .map(|(name, _cell)| &name[0..1])
            .collect::<Vec<_>>();
        let names = if names.is_empty() {
            "BLOCKED".to_string()
        } else {
            names.join("")
        };
        write!(f, "{names}")?;
        Ok(())
    }
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct CellStatus {
    visited: bool,
    marked: bool,
}

impl CellStatus {
    pub const fn visited(self) -> bool {
        self.visited
    }

    pub const fn marked(self) -> bool {
        self.marked
    }

    pub const fn visit(&mut self) {
        self.visited = true;
    }

    pub const fn mark(&mut self) {
        self.marked = true;
    }
}
#[cfg(test)]
mod tests {
    use super::Cell;

    #[test]
    fn empty_is_0000_str() {
        let zero = Cell::empty();

        assert_eq!(zero.to_bits_str(), "0000");
    }

    #[test]
    fn all_is_1111_str() {
        let zero = Cell::all();

        assert_eq!(zero.to_bits_str(), "11111111");
    }

    #[test]
    fn north_is_0001_str() {
        let zero = Cell::NORTH;

        assert_eq!(zero.to_bits_str(), "0001");
    }

    #[test]
    fn south_is_0010_str() {
        let zero = Cell::SOUTH;

        assert_eq!(zero.to_bits_str(), "0010");
    }

    #[test]
    fn east_is_0100_str() {
        let zero = Cell::EAST;

        assert_eq!(zero.to_bits_str(), "0100");
    }

    #[test]
    fn west_is_1000_str() {
        let zero = Cell::WEST;

        assert_eq!(zero.to_bits_str(), "1000");
    }

    #[test]
    fn empty_is_0000_bits() {
        let zero = Cell::empty();

        assert_eq!(zero.to_bits(), 0b0000);
    }

    #[test]
    fn all_is_1111_bits() {
        let zero = Cell::all();

        assert_eq!(zero.to_bits(), 0b1111_1111);
    }

    #[test]
    fn north_is_0001_bits() {
        let zero = Cell::NORTH;

        assert_eq!(zero.to_bits(), 0b0001);
    }

    #[test]
    fn south_is_0010_bits() {
        let zero = Cell::SOUTH;

        assert_eq!(zero.to_bits(), 0b0010);
    }

    #[test]
    fn east_is_0100_bits() {
        let zero = Cell::EAST;

        assert_eq!(zero.to_bits(), 0b0100);
    }

    #[test]
    fn west_is_1000_bits() {
        let zero = Cell::WEST;

        assert_eq!(zero.to_bits(), 0b1000);
    }

    #[test]
    fn empty_is_0000() {
        let zero = Cell::empty();

        assert_eq!(zero.to_bits_string(), "0000");
    }

    #[test]
    fn all_is_1111() {
        let zero = Cell::all();

        assert_eq!(zero.to_bits_string(), "11111111");
    }

    #[test]
    fn north_is_0001() {
        let zero = Cell::NORTH;

        assert_eq!(zero.to_bits_string(), "0001");
    }

    #[test]
    fn south_is_0010() {
        let zero = Cell::SOUTH;

        assert_eq!(zero.to_bits_string(), "0010");
    }

    #[test]
    fn east_is_0100() {
        let zero = Cell::EAST;

        assert_eq!(zero.to_bits_string(), "0100");
    }

    #[test]
    fn west_is_1000() {
        let zero = Cell::WEST;

        assert_eq!(zero.to_bits_string(), "1000");
    }

    #[test]
    fn empty_cell_name() {
        let cell = Cell::empty();

        assert_eq!(cell.to_string(), "BLOCKED");
    }

    #[test]
    fn all_cell_name() {
        let cell = Cell::all();

        assert_eq!(cell.to_string(), "NSEWNNSS");
    }

    #[test]
    fn sw_cell_name() {
        let cell = Cell::SOUTH | Cell::WEST;

        assert_eq!(cell.to_string(), "SW");
    }

    #[test]
    fn get_all_walls_count() {
        assert_eq!(Cell::empty().walls_count_sq(), 4);
        assert_eq!(Cell::SOUTH.walls_count_sq(), 3);
        assert_eq!(Cell::NORTH.walls_count_sq(), 3);
        assert_eq!(Cell::WEST.walls_count_sq(), 3);
        assert_eq!(Cell::EAST.walls_count_sq(), 3);
        assert_eq!((Cell::SOUTH | Cell::NORTH).walls_count_sq(), 2);
        assert_eq!((Cell::EAST | Cell::WEST).walls_count_sq(), 2);
        assert_eq!((Cell::SOUTH | Cell::NORTH | Cell::WEST).walls_count_sq(), 1);
        assert_eq!((Cell::EAST | Cell::NORTH | Cell::WEST).walls_count_sq(), 1);
        assert_eq!(Cell::all().walls_count_sq(), 0);
    }

    #[test]
    fn get_all_is_end() {
        assert!(!Cell::empty().is_end_sq());
        assert!(Cell::SOUTH.is_end_sq());
        assert!(Cell::NORTH.is_end_sq());
        assert!(Cell::WEST.is_end_sq());
        assert!(Cell::EAST.is_end_sq());
        assert!(!(Cell::SOUTH | Cell::NORTH).is_end_sq());
        assert!(!(Cell::EAST | Cell::WEST).is_end_sq());
        assert!(!(Cell::SOUTH | Cell::NORTH | Cell::WEST).is_end_sq());
        assert!(!(Cell::EAST | Cell::NORTH | Cell::WEST).is_end_sq());
        assert!(!Cell::all().is_end_sq());
    }
}
