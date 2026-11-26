//! Common wire fields for declaration and macro deprecation notices.
use super::{Reader, Writer};
use crate::support::crates::generics::source as s;

pub(super) fn since_write(
    out: &mut Writer,
    since: &s::DeprecationSince,
) -> Result<(), &'static str> {
    match since {
        s::DeprecationSince::Unspecified => out.u8(0),
        s::DeprecationSince::Future => out.u8(1),
        s::DeprecationSince::Version(s::AvailabilitySince::Current) => out.u8(2),
        s::DeprecationSince::Version(s::AvailabilitySince::Version {
            major,
            minor,
            patch,
        }) => {
            out.u8(3)?;
            out.u16(*major)?;
            out.u16(*minor)?;
            out.u16(*patch)
        }
        s::DeprecationSince::Nonstandard(value) => {
            out.u8(4)?;
            out.string(value)
        }
    }
}

pub(super) fn since_read(
    read: &mut Reader<'_>,
    reserve: &mut impl FnMut(usize) -> Result<(), &'static str>,
) -> Result<s::DeprecationSince, &'static str> {
    Ok(match read.u8()? {
        0 => s::DeprecationSince::Unspecified,
        1 => s::DeprecationSince::Future,
        2 => s::DeprecationSince::Version(s::AvailabilitySince::Current),
        3 => s::DeprecationSince::Version(s::AvailabilitySince::Version {
            major: read.u16()?,
            minor: read.u16()?,
            patch: read.u16()?,
        }),
        4 => s::DeprecationSince::Nonstandard(text(read, reserve)?),
        _ => return Err("unknown source deprecation since kind"),
    })
}

pub(super) fn string_option_write(
    out: &mut Writer,
    value: Option<&str>,
) -> Result<(), &'static str> {
    out.u8(u8::from(value.is_some()))?;
    if let Some(value) = value {
        out.string(value)?;
    }
    Ok(())
}

pub(super) fn string_option_read(
    read: &mut Reader<'_>,
    reserve: &mut impl FnMut(usize) -> Result<(), &'static str>,
) -> Result<Option<Box<str>>, &'static str> {
    match read.u8()? {
        0 => Ok(None),
        1 => text(read, reserve).map(Some),
        _ => Err("invalid source boolean"),
    }
}

fn text(
    read: &mut Reader<'_>,
    reserve: &mut impl FnMut(usize) -> Result<(), &'static str>,
) -> Result<Box<str>, &'static str> {
    let length = read.u32()?;
    if length > s::MAX_REASON_BYTES {
        return Err("API availability text exceeds transport limit");
    }
    read.atoms(length)?;
    reserve(length)?;
    let value = std::str::from_utf8(read.bytes(length)?)
        .map_err(|_| "API availability text is not UTF-8")?;
    Ok(Box::from(value))
}
