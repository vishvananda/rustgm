//! Closed const defaults use a sparse declaration trailer.
use super::{Input, Writer, id, s, write};
use crate::support::crates;

pub(super) fn encode(out: &mut Writer, defaults: &[s::ConstDefault]) -> Result<(), &'static str> {
    write(out, defaults, |out, row| {
        id(out, row.slot.0)?;
        match &row.value {
            s::ConstDefaultValue::Scalar(bits) => {
                out.u8(0)?;
                out.bytes(&bits.to_le_bytes())
            }
            s::ConstDefaultValue::Record(fields) => {
                out.u8(1)?;
                write(out, fields, |out, bits| out.bytes(&bits.to_le_bytes()))
            }
        }
    })
}

pub(super) fn decode(read: &mut Input<'_, '_>) -> Result<Vec<s::ConstDefault>, &'static str> {
    let defaults = read.list(crates::MAX_COMPONENTS, 9, |read| {
        let slot = s::SlotRef(read.id()?);
        let value = match read.reader.u8()? {
            0 => s::ConstDefaultValue::Scalar(read.reader.u128()?),
            1 => s::ConstDefaultValue::Record(
                read.list(crates::MAX_COMPONENTS, 16, |read| read.reader.u128())?,
            ),
            _ => return Err("unknown source const default value kind"),
        };
        Ok(s::ConstDefault { slot, value })
    })?;
    if defaults.is_empty() {
        return Err("source const default trailer is empty");
    }
    Ok(defaults)
}
