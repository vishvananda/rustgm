//! Header recipes retain separate type/region/binder and actual trait identities.
use super::{Input, Writer, address_read, address_write, g, id, option, range, s};
use crate::support::constness::Constness;

/// Version thirteen's pattern constraint kinds after tag 16 and its base.
const NON_NULL: u8 = 0;
const SCALAR_RANGE: u8 = 1;
/// Version thirteen's array type recipe tag, followed by its element and its
/// length's const term.
const ARRAY: u8 = 14;
/// The out-of-line columns some type rows write in place.
pub(super) struct TypeColumns<'a> {
    pub ranges: &'a [s::PatternRange],
    pub lengths: &'a [s::ConstTerm],
}
/// Every type row, with each range pattern's endpoints and each array's
/// length term written in place. Both columns are dense in type-row order,
/// so the reader rebuilds the same refs.
pub(super) fn types_write(
    out: &mut Writer,
    rows: &[s::Type],
    columns: &TypeColumns<'_>,
) -> Result<(), &'static str> {
    out.u32(rows.len())?;
    let mut next = 0_usize;
    let mut lengths = 0_usize;
    for row in rows {
        if let s::Type::Array { element, length } = *row {
            let term = columns
                .lengths
                .get(lengths)
                .filter(|_| length.0 as usize == lengths)
                .ok_or("source array lengths are not in type-row order")?;
            out.u8(ARRAY)?;
            id(out, element.0)?;
            const_term_write(out, *term)?;
            lengths += 1;
        } else if let s::Type::Pattern { base, constraint } = *row {
            out.u8(16)?;
            id(out, base.0)?;
            match constraint {
                s::PatternConstraint::NonNull => out.u8(NON_NULL)?,
                s::PatternConstraint::Range(reference) => {
                    let range = columns
                        .ranges
                        .get(next)
                        .filter(|_| reference.0 as usize == next)
                        .ok_or("source pattern ranges are not in type-row order")?;
                    out.u8(SCALAR_RANGE)?;
                    out.bytes(&range.start.to_le_bytes())?;
                    out.bytes(&range.end.to_le_bytes())?;
                    next += 1;
                }
            }
        } else {
            type_write(out, row)?;
        }
    }
    if next != columns.ranges.len() {
        return Err("source pattern range has no type row");
    }
    if lengths != columns.lengths.len() {
        return Err("source array length has no type row");
    }
    Ok(())
}
fn type_write(out: &mut Writer, row: &s::Type) -> Result<(), &'static str> {
    match *row {
        // Scalar has a permanent repr(u8) shared by all existing wire profiles.
        s::Type::Projection(reference) => {
            out.u8(7)?;
            id(out, reference.0)
        }
        s::Type::OpaqueFuture(reference) => {
            out.u8(8)?;
            id(out, reference.0)
        }
        s::Type::Tuple(elements) => {
            out.u8(9)?;
            range(out, elements)
        }
        s::Type::Str => out.u8(10),
        s::Type::Never => out.u8(11),
        s::Type::RawPointer { mutable, pointee } => {
            out.bytes(&[12, u8::from(mutable)])?;
            id(out, pointee.0)
        }
        s::Type::Slice(element) => {
            out.u8(13)?;
            id(out, element.0)
        }
        s::Type::Scalar(scalar) => out.bytes(&[0, scalar as u8]),
        s::Type::Nominal {
            declaration,
            arguments,
        } => {
            out.u8(6)?;
            address_write(out, declaration)?;
            range(out, arguments)
        }
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
        s::Type::Pattern { .. } => Err("source pattern type requires its range table"),
        s::Type::Array { .. } => Err("source array type requires its length table"),
    }
}
/// The dense out-of-line columns a type table's reader fills.
#[derive(Default)]
pub(super) struct TypeTables {
    pub ranges: Vec<s::PatternRange>,
    pub lengths: Vec<s::ConstTerm>,
}
/// One type row. A range pattern's endpoints and an array's length term are
/// appended to their tables, and the row names them by position.
pub(super) fn type_read(
    read: &mut Input<'_, '_>,
    tables: &mut TypeTables,
) -> Result<s::Type, &'static str> {
    Ok(match read.reader.u8()? {
        ARRAY if read.profile.body_nodes() => {
            let element = g::TypeRef(read.id()?);
            let term = const_term_read(read)?;
            read.reserve(size_of::<s::ConstTerm>())?;
            let reference = u32::try_from(tables.lengths.len())
                .map_err(|_| "source array length table overflow")?;
            tables.lengths.push(term);
            s::Type::Array {
                element,
                length: s::ArrayLengthRef(reference),
            }
        }
        16 if read.profile.body_nodes() => {
            let base = g::TypeRef(read.id()?);
            let constraint = match read.reader.u8()? {
                NON_NULL => s::PatternConstraint::NonNull,
                SCALAR_RANGE => {
                    let range = s::PatternRange {
                        start: read.reader.u128()?,
                        end: read.reader.u128()?,
                    };
                    read.reserve(size_of::<s::PatternRange>())?;
                    let reference = u32::try_from(tables.ranges.len())
                        .map_err(|_| "source pattern range table overflow")?;
                    tables.ranges.push(range);
                    s::PatternConstraint::Range(s::PatternRangeRef(reference))
                }
                _ => return Err("unknown source pattern constraint"),
            };
            s::Type::Pattern { base, constraint }
        }
        7 if read.profile.compiler_operations() || read.profile.body_nodes() => {
            s::Type::Projection(s::ProjectionRef(read.id()?))
        }
        8 if read.profile.compiler_operations() => {
            s::Type::OpaqueFuture(s::OpaqueFutureRef(read.id()?))
        }
        9 if read.profile.body_nodes() => s::Type::Tuple(read.range()?),
        10 if read.profile.body_nodes() => s::Type::Str,
        11 if read.profile.body_nodes() => s::Type::Never,
        12 if read.profile.body_nodes() => s::Type::RawPointer {
            mutable: read.boolean()?,
            pointee: g::TypeRef(read.id()?),
        },
        13 if read.profile.body_nodes() => s::Type::Slice(g::TypeRef(read.id()?)),
        0 => s::Type::Scalar(read.reader.scalar()?),
        1 => s::Type::Parameter(s::SlotRef(read.id()?)),
        6 if read.profile.nominals() => s::Type::Nominal {
            declaration: address_read(read)?,
            arguments: read.range()?,
        },
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
/// Version thirteen's const term kinds after argument tag 2.
const CONST_VALUE: u8 = 0;
const CONST_PARAMETER: u8 = 1;
/// One const term, written in place at its single reference: `0 | ty | bits`
/// (16 bytes LE) or `1 | slot`. Each position decides which tags it admits.
pub(super) fn const_term_write(out: &mut Writer, term: s::ConstTerm) -> Result<(), &'static str> {
    match term {
        s::ConstTerm::Value(value) => {
            out.u8(CONST_VALUE)?;
            id(out, value.ty.0)?;
            out.bytes(&value.bits.to_le_bytes())
        }
        s::ConstTerm::Parameter(slot) => {
            out.u8(CONST_PARAMETER)?;
            id(out, slot.0)
        }
    }
}
/// One const term. Every tag above 1 is rejected by name.
pub(super) fn const_term_read(read: &mut Input<'_, '_>) -> Result<s::ConstTerm, &'static str> {
    Ok(match read.reader.u8()? {
        CONST_VALUE => s::ConstTerm::Value(s::ConstValue {
            ty: g::TypeRef(read.id()?),
            bits: read.reader.u128()?,
        }),
        CONST_PARAMETER => s::ConstTerm::Parameter(s::SlotRef(read.id()?)),
        _ => return Err("unknown source const term"),
    })
}
/// Every generic argument row, with each const argument's term written in
/// place. The terms are dense in argument-row order, so the reader rebuilds
/// the same refs.
pub(super) fn arguments_write(
    out: &mut Writer,
    rows: &[s::Argument],
    consts: &[s::ConstTerm],
) -> Result<(), &'static str> {
    out.u32(rows.len())?;
    let mut next = 0_usize;
    for &row in rows {
        let (tag, index) = match row {
            s::Argument::Type(root) => (0, root.0),
            s::Argument::Region(region) => (1, region.0),
            s::Argument::Const(reference) => {
                let term = consts
                    .get(next)
                    .filter(|_| reference.0 as usize == next)
                    .ok_or("source const terms are not in argument-row order")?;
                out.u8(2)?;
                const_term_write(out, *term)?;
                next += 1;
                continue;
            }
        };
        out.u8(tag)?;
        id(out, index)?;
    }
    if next != consts.len() {
        return Err("source const term has no argument row");
    }
    Ok(())
}
/// One generic argument row. A const argument's term is appended to
/// `consts`, and the row names it by position.
pub(super) fn argument_read(
    read: &mut Input<'_, '_>,
    consts: &mut Vec<s::ConstTerm>,
) -> Result<s::Argument, &'static str> {
    Ok(match read.reader.u8()? {
        0 => s::Argument::Type(g::TypeRef(read.id()?)),
        1 => s::Argument::Region(s::RegionRef(read.id()?)),
        2 if read.profile.body_nodes() => {
            let term = const_term_read(read)?;
            read.reserve(size_of::<s::ConstTerm>())?;
            let reference =
                u32::try_from(consts.len()).map_err(|_| "source const term table overflow")?;
            consts.push(term);
            s::Argument::Const(s::ConstRef(reference))
        }
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
        s::Requirement::ProjectionEqual { projection, value } => {
            out.u8(4)?;
            id(out, projection.0)?;
            id(out, value.0)
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
        4 if read.profile.body_nodes() => s::Requirement::ProjectionEqual {
            projection: g::TypeRef(read.id()?),
            value: g::TypeRef(read.id()?),
        },
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
    // Version thirteen carries the conditional hierarchy of the callable roles;
    // validation admits it only on an actual callable role's own clauses.
    if tag != 0 && !(tag == 2 && read.profile.body_nodes()) {
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
