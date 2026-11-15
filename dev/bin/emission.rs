//! Where a lowered program leaves this binary: the serialized `LowIR` text.
use rustgm::lowir::{program::Program, writer};
use std::io::Write;

/// Write the program, or the counts of its records.
pub fn emit(program: &Program, count: bool, out: &mut impl Write) -> Result<(), String> {
    if count {
        writer::count(program, out)
    } else {
        writer::write(program, out)
    }
    .map_err(|error| error.to_string())
}
