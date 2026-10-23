//! Deterministic inspection of the owned-value proof, never semantic transport.
use super::{Analysis, Component, Root, Use};
use crate::body::ir::Selection;
use std::io::{self, Write};

/// # Errors
/// Propagates output failures.
pub fn write(checked: &Analysis<'_>, out: &mut impl Write) -> io::Result<()> {
    write_with_origins(checked, crate::body::BodyOrigins::source_only(), out)
}
/// # Errors
/// Rejects missing/foreign imported locations and propagates output failures.
pub fn write_with_origins(
    checked: &Analysis<'_>,
    origins: crate::body::BodyOrigins<'_>,
    out: &mut impl Write,
) -> io::Result<()> {
    writeln!(out, "ownership-crate 1")?;
    for (index, facts) in checked.bodies.iter().enumerate() {
        let body = &checked.program().bodies[index];
        write!(out, "body {index} source=")?;
        origins.body(body.source, out)?;
        writeln!(out)?;
        for (id, path) in facts.paths.entries.iter().enumerate() {
            write!(out, "  path m{id} type={} root=", path.ty.0)?;
            match path.root {
                Root::Local(id) => write!(out, "l{}", id.0)?,
                Root::Temporary(id) => write!(out, "v{}", id.0)?,
            }
            if let Some(parent) = path.parent {
                write!(out, " parent=m{}", parent.0)?;
            }
            match path.component {
                Some(Component::Field(Selection::Tuple(index))) => {
                    write!(out, " tuple-field={index}")?;
                }
                Some(Component::Field(Selection::Nominal(id))) => {
                    write!(out, " nominal-field={}", id.0)?;
                }
                Some(Component::Index(index)) => write!(out, " array-index={index}")?,
                Some(Component::Subarray { start, end }) => {
                    write!(out, " subarray={start}..{end}")?;
                }
                None => {}
            }
            writeln!(out, " subtree={}..{}", path.start, path.end)?;
        }
        for (id, selection) in facts.uses.iter().enumerate() {
            match selection {
                Use::Copy => writeln!(out, "  use v{id} copy")?,
                Use::Move => writeln!(out, "  use v{id} move")?,
                Use::Other => {}
            }
        }
        for (id, reachable) in facts.reachable.iter().enumerate() {
            writeln!(out, "  block b{id} reachable={reachable}")?;
        }
        writeln!(out, "end-body")?;
    }
    write!(out, "eof ")?;
    count(checked, out)
}
/// # Errors
/// Propagates output failures.
pub fn count(checked: &Analysis<'_>, out: &mut impl Write) -> io::Result<()> {
    let mut paths = 0;
    let mut copies = 0;
    let mut moves = 0;
    for facts in &checked.bodies {
        paths += facts.paths.entries.len();
        for selection in &facts.uses {
            copies += usize::from(*selection == Use::Copy);
            moves += usize::from(*selection == Use::Move);
        }
    }
    writeln!(
        out,
        "bodies={} paths={paths} copies={copies} moves={moves}",
        checked.bodies.len()
    )
}
