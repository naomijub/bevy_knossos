pub mod cell;
pub mod topology;
use self::cell::CellStatus;

use super::errors::TransitError;
use crate::utils::types::Coords;
use cell::Cell;
use std::fmt;
use topology::Topology;

type TransitResult<T> = Result<T, TransitError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grid {
    width: usize,
    height: usize,
    topology: Topology,
    pub(crate) cells: Vec<Cell>,
    cell_statuses: Vec<CellStatus>,
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Self {
        Self::with_topology(width, height, Topology::Orthogonal)
    }

    pub fn new_hex(width: usize, height: usize) -> Self {
        Self::with_topology(width, height, Topology::HexOddR)
    }

    pub fn with_topology(width: usize, height: usize, topology: Topology) -> Self {
        Self {
            width,
            height,
            topology,
            cells: vec![Cell::default(); width * height],
            cell_statuses: vec![CellStatus::default(); width * height],
        }
    }

    pub const fn topology(&self) -> Topology {
        self.topology
    }

    pub const fn height(&self) -> usize {
        self.height
    }

    pub const fn width(&self) -> usize {
        self.width
    }

    pub const fn directions(&self) -> &'static [Cell] {
        self.topology.directions()
    }

    pub fn mark_cell(&mut self, coords: Coords) {
        self.get_cell_status_mut(coords).mark();
    }

    pub fn is_cell_visited(&self, coords: Coords) -> bool {
        self.get_cell_status(coords).visited()
    }

    pub fn is_cell_marked(&self, coords: Coords) -> bool {
        self.get_cell_status(coords).marked()
    }

    pub fn get_cell_status(&self, coords: Coords) -> CellStatus {
        let (x, y) = coords;
        self.cell_statuses[y * self.width + x]
    }

    pub fn is_carved(&self, coords: Coords, direction: Cell) -> bool {
        let (x, y) = coords;
        self.cells[y * self.width + x].contains(direction)
    }

    pub fn carve_passage(&mut self, coords: Coords, direction: Cell) -> TransitResult<Coords> {
        let (x, y) = coords;
        let (nx, ny) = self.get_next_cell_coords(coords, direction)?;
        let opposite = self.topology.opposite(direction).ok_or_else(|| TransitError {
            coords,
            reason: format!("Invalid direction for {:?} topology", self.topology),
        })?;

        self.cells[y * self.width + x] |= direction;
        self.cells[ny * self.width + nx] |= opposite;

        self.visit_cell(coords);
        self.visit_cell((nx, ny));

        Ok((nx, ny))
    }

    pub fn get_next_cell_coords(&self, coords: Coords, direction: Cell) -> TransitResult<Coords> {
        self.topology
            .next_coords(coords, direction, self.width, self.height)
            .ok_or_else(|| TransitError {
                coords,
                reason: format!(
                    "Cannot transit from {:?} in direction {:?} for {:?} topology",
                    coords, direction, self.topology
                ),
            })
    }

    pub fn neighbor_coords(&self, coords: Coords) -> Vec<(Cell, Coords)> {
        self.directions()
            .iter()
            .filter_map(|dir| self.get_next_cell_coords(coords, *dir).ok().map(|next| (*dir, next)))
            .collect()
    }

    fn visit_cell(&mut self, coords: Coords) {
        self.get_cell_status_mut(coords).visit();
    }

    fn get_cell_status_mut(&mut self, coords: Coords) -> &mut CellStatus {
        let (x, y) = coords;
        &mut self.cell_statuses[y * self.width + x]
    }
}

impl std::ops::Index<Coords> for Grid {
    type Output = Cell;

    fn index(&self, index: Coords) -> &Self::Output {
        let width = self.width();
        let idx = index.1 * width + index.0;
        self.cells
            .get(idx)
            .unwrap_or_else(|| panic!("Cell at {:?} doesn't exist.", &index))
    }
}

impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.topology != Topology::Orthogonal {
            return write!(
                f,
                "Grid({:?}, {}x{}) display is only implemented for orthogonal topology",
                self.topology, self.width, self.height
            );
        }

        let top_border = "_".repeat(self.width * 2 - 1);

        writeln!(f, " {top_border} ")?;

        for y in 0..self.height {
            write!(f, "|")?; // display left border

            for x in 0..self.width {
                if self.is_carved((x, y), Cell::SOUTH) {
                    write!(f, " ")?;
                } else {
                    write!(f, "_")?;
                }

                if self.is_carved((x, y), Cell::EAST) {
                    if self.is_carved((x, y), Cell::SOUTH)
                        || self.is_carved((x + 1, y), Cell::SOUTH)
                    {
                        write!(f, " ")?;
                    } else {
                        write!(f, "_")?;
                    }
                } else {
                    write!(f, "|")?;
                }
            }

            writeln!(f)?; // goto next line
        }

        Ok(())
    }
}
