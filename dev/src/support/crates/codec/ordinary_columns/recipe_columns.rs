//! Header recipes retain separate type/region/binder and actual trait identities.
use super::{Input, Writer, address_read, address_write, g, id, option, range, s};
use crate::support::constness::Constness;

pub(super) fn type_write(out: &mut Writer, row: &s::Type) -> Result<(), &'static str> {
    match *row {
        // Scalar has a permanent repr(u8) shared by all existing wire profiles.
        s::Type::Scalar(scalar) => out.bytes(&[0, scalar as u8]),
        s::Type::Parameter(slot) => {
            out.u8(1)?;
            id(out, slot.0)
        }
        s::Type::Reference {
            mutable,
            region,
            referent,
        } => {
            out.bytes(&[2, u8::from(mutable)])?;
            id(out, region.0)?;
            id(out, referent.0)
        }
        s::Type::Function(signature) => {
            out.u8(3)?;
            signature_write(out, signature)
        }
        s::Type::FunctionItem(application) | s::Type::ApplicationSignature(application) => {
            out.u8(if matches!(*row, s::Type::FunctionItem(_)) {
                4
            } else {
                5
            })?;
            id(out, application.0)
        }
    }
}
pub(super) fn type_read(read: &mut Input<'_, '_>) -> Result<s::Type, &'static str> {
    Ok(match read.reader.u8()? {
        0 => s::Type::Scalar(read.reader.scalar()?),
        1 => s::Type::Parameter(s::SlotRef(read.id()?)),
        2 => s::Type::Reference {
            mutable: read.boolean()?,
            region: s::RegionRef(read.id()?),
            referent: g::TypeRef(read.id()?),
        },
        3 => s::Type::Function(signature_read(read)?),
        tag @ (4 | 5) => {
            read.require_calls("unknown source function type recipe")?;
            let application = s::FunctionApplicationRef(read.id()?);
            if tag == 4 {
                s::Type::FunctionItem(application)
            } else {
                s::Type::ApplicationSignature(application)
            }
        }
        _ => return Err("unknown source function type recipe"),
    })
}
pub(super) fn signature_write(out: &mut Writer, row: g::Signature) -> Result<(), &'static str> {
    id(out, row.binder.0)?;
    range(out, row.parameters)?;
    id(out, row.result.0)?;
    out.u8(u8::from(row.unsafe_))?;
    out.u8(match row.convention {
        g::Convention::Rust => 0,
        g::Convention::RustCall => 1,
        g::Convention::C => 2,
    })?;
    out.u8(u8::from(row.variadic))
}
pub(super) fn signature_read(read: &mut Input<'_, '_>) -> Result<g::Signature, &'static str> {
    let binder = g::BinderRef(read.id()?);
    let parameters = read.range()?;
    let result = g::TypeRef(read.id()?);
    let unsafe_ = read.boolean()?;
    let convention = match read.reader.u8()? {
        0 => g::Convention::Rust,
        1 => g::Convention::RustCall,
        2 => g::Convention::C,
        _ => return Err("unknown source function convention"),
    };
    Ok(g::Signature {
        binder,
        parameters,
        result,
        unsafe_,
        convention,
        variadic: read.boolean()?,
    })
}
pub(super) fn root_write(out: &mut Writer, row: g::TypeRef) -> Result<(), &'static str> {
    id(out, row.0)
}
pub(super) fn root_read(read: &mut Input<'_, '_>) -> Result<g::TypeRef, &'static str> {
    Ok(g::TypeRef(read.id()?))
}
pub(super) fn root_region_write(out: &mut Writer, row: s::RegionRef) -> Result<(), &'static str> {
    id(out, row.0)
}
pub(super) fn root_region_read(read: &mut Input<'_, '_>) -> Result<s::RegionRef, &'static str> {
    Ok(s::RegionRef(read.id()?))
}
pub(super) fn region_write(out: &mut Writer, row: &s::Region) -> Result<(), &'static str> {
    match *row {
        s::Region::Static => out.u8(0),
        s::Region::Bound {
            binder,
            depth,
            slot,
        } => {
            out.u8(1)?;
            id(out, binder.0)?;
            out.u16(depth)?;
            id(out, slot)
        }
        s::Region::Parameter { slot, depth } => {
            out.u8(2)?;
            id(out, slot.0)?;
            out.u16(depth)
        }
        s::Region::Variable { body, slot } => {
            out.u8(3)?;
            id(out, body.0)?;
            id(out, slot)
        }
    }
}
pub(super) fn region_read(read: &mut Input<'_, '_>) -> Result<s::Region, &'static str> {
    Ok(match read.reader.u8()? {
        0 => s::Region::Static,
        1 => s::Region::Bound {
            binder: g::BinderRef(read.id()?),
            depth: read.reader.u16()?,
            slot: read.id()?,
        },
        2 => s::Region::Parameter {
            slot: s::SlotRef(read.id()?),
            depth: read.reader.u16()?,
        },
        3 => {
            read.require_regional("unknown source function region recipe")?;
            s::Region::Variable {
                body: super::BodyRef(read.id()?),
                slot: read.id()?,
            }
        }
        _ => return Err("unknown source function region recipe"),
    })
}
pub(super) fn argument_write(out: &mut Writer, row: s::Argument) -> Result<(), &'static str> {
    let (tag, index) = match row {
        s::Argument::Type(root) => (0, root.0),
        s::Argument::Region(region) => (1, region.0),
    };
    out.u8(tag)?;
    id(out, index)
}
pub(super) fn argument_read(read: &mut Input<'_, '_>) -> Result<s::Argument, &'static str> {
    Ok(match read.reader.u8()? {
        0 => s::Argument::Type(g::TypeRef(read.id()?)),
        1 => s::Argument::Region(s::RegionRef(read.id()?)),
        _ => return Err("unknown source generic argument kind"),
    })
}
pub(super) fn application_write(
    out: &mut Writer,
    row: &s::TraitApplication,
) -> Result<(), &'static str> {
    address_write(out, row.declaration)?;
    id(out, row.subject.0)?;
    range(out, row.arguments)
}
pub(super) fn application_read(
    read: &mut Input<'_, '_>,
) -> Result<s::TraitApplication, &'static str> {
    Ok(s::TraitApplication {
        declaration: address_read(read)?,
        subject: g::TypeRef(read.id()?),
        arguments: read.range()?,
    })
}
pub(super) fn function_write(
    out: &mut Writer,
    row: &s::FunctionApplication,
) -> Result<(), &'static str> {
    address_write(out, row.declaration)?;
    range(out, row.arguments)
}
pub(super) fn function_read(
    read: &mut Input<'_, '_>,
) -> Result<s::FunctionApplication, &'static str> {
    Ok(s::FunctionApplication {
        declaration: address_read(read)?,
        arguments: read.range()?,
    })
}
pub(super) fn clause_write(out: &mut Writer, row: &s::Clause) -> Result<(), &'static str> {
    id(out, row.owner.0)?;
    id(out, row.site)?;
    option(out, row.binder.map(|r| r.0))?;
    requirement_write(out, &row.requirement)?;
    option(out, row.self_subject.map(|r| r.0))
}
pub(super) fn clause_read(read: &mut Input<'_, '_>) -> Result<s::Clause, &'static str> {
    Ok(s::Clause {
        owner: super::DeclarationRef(read.id()?),
        site: read.id()?,
        binder: read.option()?.map(g::BinderRef),
        requirement: requirement_read(read)?,
        self_subject: read.option()?.map(s::SlotRef),
    })
}
pub(super) fn requirement_write(
    out: &mut Writer,
    row: &s::Requirement,
) -> Result<(), &'static str> {
    match row {
        s::Requirement::Trait {
            application,
            effect,
        } => {
            out.u8(0)?;
            id(out, application.0)?;
            effect_write(out, *effect)
        }
        s::Requirement::TypeOutlives { subject, shorter } => {
            out.u8(1)?;
            id(out, subject.0)?;
            id(out, shorter.0)
        }
        s::Requirement::Outlives(bound) => {
            out.u8(2)?;
            bound_write(out, *bound)
        }
        s::Requirement::UnstableFeature(name) => {
            out.u8(3)?;
            out.string(name)
        }
    }
}
pub(super) fn requirement_read(read: &mut Input<'_, '_>) -> Result<s::Requirement, &'static str> {
    Ok(match read.reader.u8()? {
        0 => {
            let application = s::TraitApplicationRef(read.id()?);
            let effect = effect_read(
                read,
                "source qualified request requires a wider source profile",
            )?;
            s::Requirement::Trait {
                application,
                effect,
            }
        }
        1 => s::Requirement::TypeOutlives {
            subject: g::TypeRef(read.id()?),
            shorter: s::RegionRef(read.id()?),
        },
        2 => s::Requirement::Outlives(bound_read(read)?),
        3 => {
            if !read.profile.feature_predicates() {
                return Err("feature predicates require source metadata version four");
            }
            s::Requirement::UnstableFeature(read.string()?.into_boxed_str())
        }
        _ => return Err("unknown source function requirement"),
    })
}
pub(super) fn effect_write(out: &mut Writer, effect: Constness) -> Result<(), &'static str> {
    out.u8(match effect {
        Constness::Ordinary => 0,
        Constness::Const => 1,
        Constness::Conditional => 2,
    })
}
pub(super) fn effect_read(
    read: &mut Input<'_, '_>,
    unsupported: &'static str,
) -> Result<Constness, &'static str> {
    let tag = read.reader.u8()?;
    if tag != 0 {
        read.require_provided(unsupported)?;
    }
    match tag {
        0 => Ok(Constness::Ordinary),
        1 => Ok(Constness::Const),
        2 => Ok(Constness::Conditional),
        _ => Err("unknown source constness effect"),
    }
}
/// Function declaration eligibility is unconditional in version seven.
/// Trait predicates keep their distinct Provided-only qualified reader.
pub(super) fn function_effect_read(
    read: &mut Input<'_, '_>,
    unsupported: &'static str,
) -> Result<Constness, &'static str> {
    if !read.profile.unconditional_callables() {
        return effect_read(read, unsupported);
    }
    match read.reader.u8()? {
        0 => Ok(Constness::Ordinary),
        1 => Ok(Constness::Const),
        2 if read.profile.conversion_roles() => Ok(Constness::Conditional),
        2 => Err("source conditional callable effect requires its wider profile"),
        _ => Err("unknown source constness effect"),
    }
}
/// A const definition is usable at runtime or CTFE; its call use is Conditional.
/// Exact original-context equality and genuine const targets are still required.
pub(super) fn call_effect_read(
    read: &mut Input<'_, '_>,
    unsupported: &'static str,
) -> Result<Constness, &'static str> {
    if !read.profile.unconditional_callables() {
        return effect_read(read, unsupported);
    }
    match read.reader.u8()? {
        0 => Ok(Constness::Ordinary),
        2 => Ok(Constness::Conditional),
        1 => Err("source closed-const call requires its wider profile"),
        _ => Err("unknown source constness effect"),
    }
}
pub(super) fn bound_write(out: &mut Writer, row: s::RegionBound) -> Result<(), &'static str> {
    id(out, row.longer.0)?;
    id(out, row.shorter.0)
}
pub(super) fn bound_read(read: &mut Input<'_, '_>) -> Result<s::RegionBound, &'static str> {
    Ok(s::RegionBound {
        longer: s::RegionRef(read.id()?),
        shorter: s::RegionRef(read.id()?),
    })
}
