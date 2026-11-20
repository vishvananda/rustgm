//! Serialize `LowIR` before publishing its executable metadata.
use crate::crates::ExecutableMetadata;
use rustgm::lowir::{program::Program, writer};
use std::io::Write;

/// Write the program, or the counts of its records.
pub fn emit(
    program: &Program,
    count: bool,
    out: &mut impl Write,
    metadata: Option<ExecutableMetadata<'_>>,
) -> Result<(), String> {
    if count {
        writer::count(program, out)
    } else {
        writer::write(program, out)
    }
    .map_err(|error| error.to_string())?;
    if let Some(metadata) = metadata {
        metadata.publish(out)?;
    }
    Ok(())
}
