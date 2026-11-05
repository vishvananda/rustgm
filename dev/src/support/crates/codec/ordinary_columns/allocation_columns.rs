//! Unit order and optional aliases are data, not serialized region identities.
use super::{BodyRef, Input, Writer, crates, id, index_write, s, write_copy};

pub(super) fn unit_write(out: &mut Writer, row: &s::BodyUnit) -> Result<(), &'static str> {
    id(out, row.body.0)?;
    id(out, row.universals)?;
    write_copy(out, &row.allocation_order, index_write)?;
    out.u8(u8::from(row.aliases.is_some()))?;
    if let Some(aliases) = &row.aliases {
        write_copy(out, aliases, index_write)?;
    }
    Ok(())
}
pub(super) fn unit_read(read: &mut Input<'_, '_>) -> Result<s::BodyUnit, &'static str> {
    Ok(s::BodyUnit {
        body: BodyRef(read.id()?),
        universals: read.id()?,
        allocation_order: read.list(crates::MAX_COMPONENTS, 4, Input::id)?,
        aliases: if read.boolean()? {
            Some(read.list(crates::MAX_COMPONENTS, 4, Input::id)?)
        } else {
            None
        },
    })
}
pub(super) fn owned_write(out: &mut Writer, row: &s::OwnedRegion) -> Result<(), &'static str> {
    id(out, row.site)?;
    match row.allocation {
        s::RegionAllocation::Inference { needs_provenance } => {
            out.u8(0)?;
            out.u8(u8::from(needs_provenance))
        }
        s::RegionAllocation::Normalization(input) => {
            out.u8(1)?;
            id(out, input)
        }
    }
}
pub(super) fn owned_read(read: &mut Input<'_, '_>) -> Result<s::OwnedRegion, &'static str> {
    let site = read.id()?;
    let allocation = match read.reader.u8()? {
        0 => s::RegionAllocation::Inference {
            needs_provenance: read.boolean()?,
        },
        1 => s::RegionAllocation::Normalization(read.id()?),
        _ => return Err("unknown source owned allocation kind"),
    };
    Ok(s::OwnedRegion { site, allocation })
}
pub(super) fn projection_write(out: &mut Writer, row: &s::Projection) -> Result<(), &'static str> {
    let s::ProjectionKind::Dereference { region, mutable } = row.kind else {
        return Err("source field projection requires its explicit version-ten profile");
    };
    super::operation_columns::place_write(out, row.base)?;
    id(out, row.ty.0)?;
    id(out, region.0)?;
    out.u8(u8::from(mutable))
}
pub(super) fn nominal_projection_write(
    out: &mut Writer,
    row: &s::Projection,
) -> Result<(), &'static str> {
    super::operation_columns::place_write(out, row.base)?;
    id(out, row.ty.0)?;
    match row.kind {
        s::ProjectionKind::Dereference { region, mutable } => {
            out.u8(0)?;
            id(out, region.0)?;
            out.u8(u8::from(mutable))
        }
        s::ProjectionKind::Field { owner, ordinal } => {
            out.u8(1)?;
            id(out, owner.0)?;
            id(out, ordinal)
        }
    }
}
pub(super) fn projection_read(read: &mut Input<'_, '_>) -> Result<s::Projection, &'static str> {
    let base = super::operation_columns::place_read(read)?;
    let ty = super::g::TypeRef(read.id()?);
    let kind = if read.profile.nominals() {
        match read.reader.u8()? {
            0 => s::ProjectionKind::Dereference {
                region: s::RegionRef(read.id()?),
                mutable: read.boolean()?,
            },
            1 => s::ProjectionKind::Field {
                owner: super::DeclarationRef(read.id()?),
                ordinal: read.id()?,
            },
            _ => return Err("unknown source nominal projection kind"),
        }
    } else {
        s::ProjectionKind::Dereference {
            region: s::RegionRef(read.id()?),
            mutable: read.boolean()?,
        }
    };
    Ok(s::Projection { base, ty, kind })
}
