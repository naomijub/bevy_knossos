use crate::maze::{
    formatters::{Formatter, StringWrapper},
    grid::{Grid, topology::Topology},
};
use std::fmt::Write;

/// Formatter that serializes a hex maze grid into a compact text representation.
///
/// The produced text is intended for storage and deserialization via
/// [`crate::maze::HexMaze::from_text`].
pub struct HexText;

impl Formatter<StringWrapper> for HexText {
    fn format(&self, grid: &Grid) -> StringWrapper {
        if grid.topology() != Topology::HexOddR {
            return StringWrapper(
                "KNOSSOS_HEX_V1\nerror=HexText formatter only supports hex topology\n".to_string(),
            );
        }

        let mut output = String::new();
        output.push_str("KNOSSOS_HEX_V1\n");
        let _ = writeln!(output, "width={}", grid.width());
        let _ = writeln!(output, "height={}", grid.height());

        for y in 0..grid.height() {
            for x in 0..grid.width() {
                if x > 0 {
                    output.push(',');
                }
                let bits = grid[(x, y)].to_bits();
                let _ = write!(output, "{bits:02X}");
            }
            output.push('\n');
        }

        StringWrapper(output)
    }
}
