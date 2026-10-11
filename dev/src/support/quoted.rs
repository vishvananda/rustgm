//! Deterministic byte quoting shared by text views, never a semantic format.
use std::io::{self, Write};

pub(crate) fn bytes(output: &mut impl Write, bytes: &[u8], nul: bool) -> io::Result<()> {
    output.write_all(b"\"")?;
    for &byte in bytes {
        match byte {
            b'"' => output.write_all(b"\\\"")?,
            b'\\' => output.write_all(b"\\\\")?,
            b'\n' => output.write_all(b"\\n")?,
            b'\r' => output.write_all(b"\\r")?,
            b'\t' => output.write_all(b"\\t")?,
            32..=126 => output.write_all(&[byte])?,
            _ => write!(output, "\\x{byte:02x}")?,
        }
    }
    if nul {
        output.write_all(b"\\x00")?;
    }
    output.write_all(b"\"")
}
