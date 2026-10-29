//! Header rows preserve required-body absence and full canonical input boundaries.
use super::{
    BodyRef, DeclarationRef, Input, Writer, address_read, address_write, g, id, option, range, s,
};
use crate::support::bindings::LangItem;

pub(super) fn trait_write(out: &mut Writer, row: &s::TraitHeader) -> Result<(), &'static str> {
    for value in [
        row.declaration.0,
        row.scope.0,
        row.self_definition.0,
        row.self_type.0,
    ] {
        id(out, value)?;
    }
    out.u16(row.flags.bits())?;
    id(out, row.region_captures)?;
    range(out, row.region_bounds)?;
    range(out, row.inputs)?;
    for value in [
        row.canonical_inputs,
        row.declared_inputs,
        row.written_parameters,
    ] {
        id(out, value)?;
    }
    range(out, row.written_supertraits)?;
    range(out, row.members)
}
pub(super) fn trait_read(read: &mut Input<'_, '_>) -> Result<s::TraitHeader, &'static str> {
    Ok(s::TraitHeader {
        declaration: DeclarationRef(read.id()?),
        scope: DeclarationRef(read.id()?),
        self_definition: DeclarationRef(read.id()?),
        self_type: g::TypeRef(read.id()?),
        flags: s::TraitFlags::from_bits(read.reader.u16()?)?,
        region_captures: read.id()?,
        region_bounds: read.range()?,
        inputs: read.range()?,
        canonical_inputs: read.id()?,
        declared_inputs: read.id()?,
        written_parameters: read.id()?,
        written_supertraits: read.range()?,
        members: read.range()?,
    })
}
pub(super) fn implementation_write(
    out: &mut Writer,
    row: &s::ImplHeader,
) -> Result<(), &'static str> {
    for value in [row.self_definition.0, row.scope.0, row.self_type.0] {
        id(out, value)?;
    }
    option(out, row.trait_application.map(|r| r.0))?;
    out.bytes(&[
        u8::from(row.unsafe_),
        u8::from(row.constant),
        u8::from(row.negative),
    ])?;
    id(out, row.region_captures)?;
    range(out, row.region_bounds)?;
    range(out, row.inputs)?;
    id(out, row.declared_inputs)?;
    range(out, row.members)
}
pub(super) fn implementation_read(read: &mut Input<'_, '_>) -> Result<s::ImplHeader, &'static str> {
    Ok(s::ImplHeader {
        self_definition: DeclarationRef(read.id()?),
        scope: DeclarationRef(read.id()?),
        self_type: g::TypeRef(read.id()?),
        trait_application: read.option()?.map(s::TraitApplicationRef),
        unsafe_: read.boolean()?,
        constant: read.boolean()?,
        negative: read.boolean()?,
        region_captures: read.id()?,
        region_bounds: read.range()?,
        inputs: read.range()?,
        declared_inputs: read.id()?,
        members: read.range()?,
    })
}
pub(super) fn function_write(
    out: &mut Writer,
    row: &s::FunctionHeader,
) -> Result<(), &'static str> {
    id(out, row.declaration.0)?;
    id(out, row.scope.0)?;
    option(out, row.parent.map(|r| r.0))?;
    super::recipe_columns::signature_write(out, row.callable)?;
    super::recipe_columns::signature_write(out, row.original)?;
    id(out, row.regions.total)?;
    id(out, row.regions.captures)?;
    for bounds in [
        row.regions.early,
        row.regions.late,
        row.regions.bounds,
        row.inputs,
    ] {
        range(out, bounds)?;
    }
    id(out, row.declared_inputs)?;
    range(out, row.formals)?;
    out.u8(u8::from(row.receiver.is_some()))?;
    if let Some(receiver) = row.receiver {
        for value in [receiver.formal, receiver.base.0, receiver.references] {
            id(out, value)?;
        }
        out.u8(u8::from(receiver.mutable_binding))?;
    }
    super::recipe_columns::effect_write(out, row.constness)?;
    match row.body {
        s::FunctionBody::Absent => out.u8(0),
        s::FunctionBody::Original(body) => {
            out.u8(1)?;
            id(out, body.0)
        }
    }
}
pub(super) fn function_read(read: &mut Input<'_, '_>) -> Result<s::FunctionHeader, &'static str> {
    let declaration = DeclarationRef(read.id()?);
    let scope = DeclarationRef(read.id()?);
    let parent = read.option()?.map(DeclarationRef);
    let callable = super::recipe_columns::signature_read(read)?;
    let original = super::recipe_columns::signature_read(read)?;
    let regions = s::RegionScheme {
        total: read.id()?,
        captures: read.id()?,
        early: read.range()?,
        late: read.range()?,
        bounds: read.range()?,
    };
    let inputs = read.range()?;
    let declared_inputs = read.id()?;
    let formals = read.range()?;
    let receiver = if read.boolean()? {
        Some(s::Receiver {
            formal: read.id()?,
            base: g::TypeRef(read.id()?),
            references: read.id()?,
            mutable_binding: read.boolean()?,
        })
    } else {
        None
    };
    let constness = super::recipe_columns::function_effect_read(
        read,
        "source callable effect requires a wider source profile",
    )?;
    let body = match read.reader.u8()? {
        0 => s::FunctionBody::Absent,
        1 => s::FunctionBody::Original(BodyRef(read.id()?)),
        _ => return Err("invalid source required/provided body tag"),
    };
    Ok(s::FunctionHeader {
        declaration,
        scope,
        parent,
        callable,
        original,
        regions,
        inputs,
        declared_inputs,
        formals,
        receiver,
        constness,
        body,
    })
}
pub(super) fn formal_write(out: &mut Writer, row: &s::Formal) -> Result<(), &'static str> {
    option(out, row.declaration.map(|r| r.0))?;
    id(out, row.ordinal)
}
pub(super) fn formal_read(read: &mut Input<'_, '_>) -> Result<s::Formal, &'static str> {
    Ok(s::Formal {
        declaration: read.option()?.map(DeclarationRef),
        ordinal: read.id()?,
    })
}
pub(super) fn member_write(out: &mut Writer, row: &s::MemberLink) -> Result<(), &'static str> {
    out.u8(u8::from(row.trait_member.is_some()))?;
    if let Some(member) = row.trait_member {
        address_write(out, member)?;
    }
    option(out, row.implementation.map(|r| r.0))
}
pub(super) fn member_read(read: &mut Input<'_, '_>) -> Result<s::MemberLink, &'static str> {
    Ok(s::MemberLink {
        trait_member: if read.boolean()? {
            Some(address_read(read)?)
        } else {
            None
        },
        implementation: read.option()?.map(DeclarationRef),
    })
}
pub(super) fn role_write(out: &mut Writer, row: &s::RoleBinding) -> Result<(), &'static str> {
    // These profile-local IDs are independent of the compiler's LangItem order.
    out.u8(match row.role {
        LangItem::Sized => 0,
        LangItem::MetaSized => 1,
        LangItem::PointeeSized => 2,
        LangItem::LegacyReceiver => 3,
        LangItem::Clone => 4,
        LangItem::CloneFn => 5,
        LangItem::Copy => 6,
        LangItem::Destruct => 7,
        LangItem::Freeze => 8,
        _ => return Err("source role requires a wider source profile"),
    })?;
    address_write(out, row.declaration)
}
pub(super) fn role_read(read: &mut Input<'_, '_>) -> Result<s::RoleBinding, &'static str> {
    let role = match read.reader.u8()? {
        0 => LangItem::Sized,
        1 => LangItem::MetaSized,
        2 => LangItem::PointeeSized,
        3 => LangItem::LegacyReceiver,
        4 => LangItem::Clone,
        5 => LangItem::CloneFn,
        6 => LangItem::Copy,
        tag @ (7 | 8) => {
            if !read.callables {
                read.require_provided("unknown source function role ID")?;
            }
            if tag == 7 {
                LangItem::Destruct
            } else {
                LangItem::Freeze
            }
        }
        _ => return Err("unknown source function role ID"),
    };
    Ok(s::RoleBinding {
        role,
        declaration: address_read(read)?,
    })
}
