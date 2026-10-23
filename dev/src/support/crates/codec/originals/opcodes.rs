//! Versioned operation operands have no proof authority.
use super::{Input, Writer, g, id, range};
pub(super) fn encode(
    out: &mut Writer,
    values: &[g::Value],
    version: u8,
) -> Result<(), &'static str> {
    for value in values {
        id(out, value.ty.0)?;
        match value.kind {
            g::ValueKind::Unit => out.u8(0)?,
            g::ValueKind::Read(local) => {
                out.u8(1)?;
                id(out, local)?;
            }
            g::ValueKind::Constrain(value) => {
                out.u8(2)?;
                id(out, value)?;
            }
            g::ValueKind::Function(application) => {
                if version == 0 {
                    return Err("generic function values require row version one");
                }
                out.u8(3)?;
                id(out, application.0)?;
            }
            g::ValueKind::Call {
                signature,
                callee,
                arguments,
                site,
            } => {
                if version == 0 {
                    return Err("generic calls require row version one");
                }
                out.u8(4)?;
                id(out, signature.0)?;
                id(out, callee)?;
                range(out, arguments)?;
                id(out, site)?;
            }
        }
    }
    Ok(())
}

pub(super) fn decode(read: &mut Input<'_, '_>) -> Result<g::Value, &'static str> {
    let ty = g::TypeRef(read.id()?);
    let kind = match read.reader.u8()? {
        0 => g::ValueKind::Unit,
        1 => g::ValueKind::Read(read.id()?),
        2 => g::ValueKind::Constrain(read.id()?),
        3 if read.version == 1 => g::ValueKind::Function(g::ApplicationRef(read.id()?)),
        4 if read.version == 1 => g::ValueKind::Call {
            signature: g::TypeRef(read.id()?),
            callee: read.id()?,
            arguments: read.range()?,
            site: read.id()?,
        },
        _ => return Err("unknown generic value operation"),
    };
    Ok(g::Value { ty, kind })
}
