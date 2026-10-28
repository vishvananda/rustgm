//! Version-three API policy columns use the same bounded source-row decoder.
use super::{Input, Writer, id, s, write};
use crate::support::crates::{MAX_COMPONENTS, owner_catalog::DeclarationRef};

pub(super) fn encode(out: &mut Writer, rows: &s::AvailabilityRows) -> Result<(), &'static str> {
    write(out, &rows.policies, policy_write)?;
    write(out, &rows.declarations, |out, row| {
        id(out, row.declaration.0)?;
        reference(out, row.regular)?;
        reference(out, row.constant)
    })?;
    write(out, &rows.bindings, |out, row| {
        id(out, row.binding)?;
        reference(out, row.regular)
    })
}
pub(super) fn decode(read: &mut Input<'_, '_>) -> Result<s::AvailabilityRows, &'static str> {
    read.reserve(size_of::<s::AvailabilityRows>())?;
    Ok(s::AvailabilityRows {
        policies: read.list(MAX_COMPONENTS, 13, policy_read)?,
        declarations: read.list(MAX_COMPONENTS, 6, |read| {
            Ok(s::DeclarationAvailability {
                declaration: DeclarationRef(read.id()?),
                regular: read.option()?.map(s::AvailabilityRef),
                constant: read.option()?.map(s::AvailabilityRef),
            })
        })?,
        bindings: read.list(MAX_COMPONENTS, 5, |read| {
            Ok(s::BindingAvailability {
                binding: read.id()?,
                regular: read.option()?.map(s::AvailabilityRef),
            })
        })?,
    })
}
fn reference(out: &mut Writer, value: Option<s::AvailabilityRef>) -> Result<(), &'static str> {
    out.u8(u8::from(value.is_some()))?;
    if let Some(value) = value {
        id(out, value.0)?;
    }
    Ok(())
}
fn policy_write(out: &mut Writer, policy: &s::AvailabilityPolicy) -> Result<(), &'static str> {
    out.u8(match policy.kind {
        s::AvailabilityKind::Regular => 0,
        s::AvailabilityKind::Const => 1,
    })?;
    out.string(&policy.feature)?;
    match &policy.level {
        s::AvailabilityLevel::Stable(since) => {
            out.u8(0)?;
            match since {
                s::AvailabilitySince::Current => out.u8(0)?,
                s::AvailabilitySince::Version {
                    major,
                    minor,
                    patch,
                } => {
                    out.u8(1)?;
                    out.u16(*major)?;
                    out.u16(*minor)?;
                    out.u16(*patch)?;
                }
            }
        }
        s::AvailabilityLevel::Unstable { issue, reason } => {
            out.u8(1)?;
            out.u8(u8::from(issue.is_some()))?;
            if let Some(issue) = issue {
                id(out, issue.get())?;
            }
            out.u8(u8::from(reason.is_some()))?;
            if let Some(reason) = reason {
                out.string(reason)?;
            }
        }
    }
    match policy.origin {
        s::AvailabilityOrigin::Declaration { owner, site } => {
            out.u8(0)?;
            id(out, owner.0)?;
            id(out, site)?;
        }
        s::AvailabilityOrigin::Binding { binding, site } => {
            out.u8(1)?;
            id(out, binding)?;
            id(out, site)?;
        }
    }
    Ok(())
}
fn policy_read(read: &mut Input<'_, '_>) -> Result<s::AvailabilityPolicy, &'static str> {
    let kind = match read.reader.u8()? {
        0 => s::AvailabilityKind::Regular,
        1 => s::AvailabilityKind::Const,
        _ => return Err("unknown API availability policy kind"),
    };
    let feature = text(read)?;
    let level = match read.reader.u8()? {
        0 => s::AvailabilityLevel::Stable(match read.reader.u8()? {
            0 => s::AvailabilitySince::Current,
            1 => s::AvailabilitySince::Version {
                major: read.reader.u16()?,
                minor: read.reader.u16()?,
                patch: read.reader.u16()?,
            },
            _ => return Err("unknown API stability version kind"),
        }),
        1 => {
            let issue = read
                .option()?
                .map(|issue| {
                    std::num::NonZeroU32::new(issue).ok_or("API availability issue must be nonzero")
                })
                .transpose()?;
            let reason = if read.boolean()? {
                Some(text(read)?)
            } else {
                None
            };
            s::AvailabilityLevel::Unstable { issue, reason }
        }
        _ => return Err("unknown API availability level"),
    };
    let origin = match read.reader.u8()? {
        0 => s::AvailabilityOrigin::Declaration {
            owner: DeclarationRef(read.id()?),
            site: read.id()?,
        },
        1 => s::AvailabilityOrigin::Binding {
            binding: read.id()?,
            site: read.id()?,
        },
        _ => return Err("unknown API availability origin kind"),
    };
    Ok(s::AvailabilityPolicy {
        kind,
        feature,
        level,
        origin,
    })
}
fn text(read: &mut Input<'_, '_>) -> Result<String, &'static str> {
    let length = read.reader.u32()?;
    if length > s::MAX_REASON_BYTES {
        return Err("API availability text exceeds transport limit");
    }
    read.reader.atoms(length)?;
    read.reserve(length)?;
    let value = std::str::from_utf8(read.reader.bytes(length)?)
        .map_err(|_| "API availability text is not UTF-8")?;
    Ok(value.to_owned())
}
