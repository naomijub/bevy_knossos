#![allow(missing_docs)]

use super::cell::Cell;
use crate::utils::types::Coords;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Topology {
    Orthogonal,
    HexOddR,
}

impl Topology {
    #[must_use]
    pub const fn directions(self) -> &'static [Cell] {
        match self {
            Self::Orthogonal => &[Cell::NORTH, Cell::SOUTH, Cell::WEST, Cell::EAST],
            Self::HexOddR => &[
                Cell::EAST,
                Cell::WEST,
                Cell::NORTH_EAST,
                Cell::NORTH_WEST,
                Cell::SOUTH_EAST,
                Cell::SOUTH_WEST,
            ],
        }
    }

    #[must_use]
    pub const fn sides(self) -> u8 {
        match self {
            Self::Orthogonal => 4,
            Self::HexOddR => 6,
        }
    }

    #[must_use]
    pub const fn opposite(self, direction: Cell) -> Option<Cell> {
        match self {
            Self::Orthogonal => match direction {
                Cell::NORTH => Some(Cell::SOUTH),
                Cell::SOUTH => Some(Cell::NORTH),
                Cell::EAST => Some(Cell::WEST),
                Cell::WEST => Some(Cell::EAST),
                _ => None,
            },
            Self::HexOddR => match direction {
                Cell::EAST => Some(Cell::WEST),
                Cell::WEST => Some(Cell::EAST),
                Cell::NORTH_EAST => Some(Cell::SOUTH_WEST),
                Cell::NORTH_WEST => Some(Cell::SOUTH_EAST),
                Cell::SOUTH_EAST => Some(Cell::NORTH_WEST),
                Cell::SOUTH_WEST => Some(Cell::NORTH_EAST),
                _ => None,
            },
        }
    }

    #[must_use]
    pub const fn next_coords(
        self,
        (x, y): Coords,
        direction: Cell,
        width: usize,
        height: usize,
    ) -> Option<Coords> {
        match self {
            Self::Orthogonal => match direction {
                Cell::NORTH if y > 0 => Some((x, y - 1)),
                Cell::SOUTH if y + 1 < height => Some((x, y + 1)),
                Cell::WEST if x > 0 => Some((x - 1, y)),
                Cell::EAST if x + 1 < width => Some((x + 1, y)),
                _ => None,
            },
            Self::HexOddR => {
                let is_odd_row = y % 2 == 1;
                match direction {
                    Cell::EAST if x + 1 < width => Some((x + 1, y)),
                    Cell::WEST if x > 0 => Some((x - 1, y)),
                    Cell::NORTH_EAST if y > 0 => {
                        if is_odd_row {
                            if x + 1 < width {
                                Some((x + 1, y - 1))
                            } else {
                                None
                            }
                        } else {
                            Some((x, y - 1))
                        }
                    }
                    Cell::NORTH_WEST if y > 0 => {
                        if is_odd_row {
                            Some((x, y - 1))
                        } else if x > 0 {
                            Some((x - 1, y - 1))
                        } else {
                            None
                        }
                    }
                    Cell::SOUTH_EAST if y + 1 < height => {
                        if is_odd_row {
                            if x + 1 < width {
                                Some((x + 1, y + 1))
                            } else {
                                None
                            }
                        } else {
                            Some((x, y + 1))
                        }
                    }
                    Cell::SOUTH_WEST if y + 1 < height => {
                        if is_odd_row {
                            Some((x, y + 1))
                        } else if x > 0 {
                            Some((x - 1, y + 1))
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }
        }
    }
}
