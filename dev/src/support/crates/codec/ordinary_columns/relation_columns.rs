//! Complete unpaid original requests remain separate from immutable operation rows.
use super::{
    BodyRef, DeclarationRef, Input, Writer, address_read, address_write, crates, g, id,
    recipe_columns as types, s, write, write_copy,
};

pub(super) fn body_write(out: &mut Writer, row: &s::BodyInputs) -> Result<(), &'static str> {
    id(out, row.body.0)?;
    id(out, row.frame.0)?;
    write_copy(out, &row.parameters, super::owner_rows::slot_ref_write)?;
    environment_write(out, row.environment)?;
    write_copy(out, &row.assumptions, types::bound_write)?;
    write(out, &row.type_assumptions, located_write)?;
    write(out, &row.obligations, obligation_write)?;
    write(out, &row.normalizations, normalization_write)?;
    out.u8(u8::from(row.builtins.is_some()))?;
    if let Some(builtins) = &row.builtins {
        builtins_write(out, builtins)?;
    }
    Ok(())
}
pub(super) fn body_read(read: &mut Input<'_, '_>) -> Result<s::BodyInputs, &'static str> {
    Ok(s::BodyInputs {
        body: BodyRef(read.id()?),
        frame: g::BinderRef(read.id()?),
        parameters: read.list(crates::MAX_COMPONENTS, 4, super::owner_rows::slot_ref_read)?,
        environment: environment_read(read)?,
        assumptions: read.list(crates::MAX_COMPONENTS, 8, types::bound_read)?,
        type_assumptions: read.list(crates::MAX_COMPONENTS, 11, located_read)?,
        obligations: read.list(crates::MAX_COMPONENTS, 10, obligation_read)?,
        normalizations: read.list(crates::MAX_COMPONENTS, 25, normalization_read)?,
        builtins: if read.boolean()? {
            Some(builtins_read(read)?)
        } else {
            None
        },
    })
}
fn environment_write(out: &mut Writer, row: s::EnvironmentOrigin) -> Result<(), &'static str> {
    match row {
        s::EnvironmentOrigin::Absent => out.u8(0),
        s::EnvironmentOrigin::Declaration(owner) => {
            out.u8(1)?;
            id(out, owner.0)
        }
        s::EnvironmentOrigin::Contract => out.u8(2),
        s::EnvironmentOrigin::ConstTraitDefault { member, trait_ } => {
            out.u8(3)?;
            id(out, member.0)?;
            address_write(out, trait_)
        }
    }
}
pub(super) fn environment_read(
    read: &mut Input<'_, '_>,
) -> Result<s::EnvironmentOrigin, &'static str> {
    Ok(match read.reader.u8()? {
        0 => s::EnvironmentOrigin::Absent,
        1 => s::EnvironmentOrigin::Declaration(DeclarationRef(read.id()?)),
        2 => s::EnvironmentOrigin::Contract,
        3 => {
            read.require_provided("unknown source ordinary environment origin")?;
            s::EnvironmentOrigin::ConstTraitDefault {
                member: DeclarationRef(read.id()?),
                trait_: address_read(read)?,
            }
        }
        _ => return Err("unknown source ordinary environment origin"),
    })
}
fn site_write(out: &mut Writer, row: &s::InputSite) -> Result<(), &'static str> {
    match *row {
        s::InputSite::Body(site) => {
            out.u8(0)?;
            id(out, site)
        }
        s::InputSite::Declaration { owner, site } => {
            out.u8(1)?;
            id(out, owner.0)?;
            id(out, site)
        }
    }
}
fn site_read(read: &mut Input<'_, '_>) -> Result<s::InputSite, &'static str> {
    Ok(match read.reader.u8()? {
        0 => s::InputSite::Body(read.id()?),
        1 => s::InputSite::Declaration {
            owner: DeclarationRef(read.id()?),
            site: read.id()?,
        },
        _ => return Err("unknown source input-site ownership domain"),
    })
}
fn located_write(out: &mut Writer, row: &s::LocatedRequirement) -> Result<(), &'static str> {
    site_write(out, &row.site)?;
    types::requirement_write(out, &row.value)
}
fn located_read(read: &mut Input<'_, '_>) -> Result<s::LocatedRequirement, &'static str> {
    Ok(s::LocatedRequirement {
        site: site_read(read)?,
        value: types::requirement_read(read)?,
    })
}
fn obligation_write(out: &mut Writer, row: &s::Obligation) -> Result<(), &'static str> {
    site_write(out, &row.site)?;
    match row.kind {
        s::InputKind::Coercion {
            actual,
            expected,
            unsize,
            reference,
        } => {
            out.u8(0)?;
            id(out, actual.0)?;
            id(out, expected.0)?;
            out.u8(u8::from(unsize))?;
            out.u8(u8::from(reference.is_some()))?;
            if let Some(reference) = reference {
                id(out, reference.dereferences)?;
                out.bytes(&[u8::from(reference.unsize), u8::from(reference.two_phase)])?;
            }
            Ok(())
        }
        s::InputKind::WellFormed(root) => {
            out.u8(1)?;
            id(out, root.0)
        }
        s::InputKind::Equality(bound) => {
            out.u8(2)?;
            types::bound_write(out, bound)
        }
        s::InputKind::Outlives(bound) => {
            out.u8(3)?;
            types::bound_write(out, bound)
        }
        s::InputKind::TypeOutlives { ty, shorter } => {
            out.u8(4)?;
            id(out, ty.0)?;
            id(out, shorter.0)
        }
        s::InputKind::Variable {
            slot,
            needs_provenance,
        } => {
            out.u8(5)?;
            id(out, slot)?;
            out.u8(u8::from(needs_provenance))
        }
        s::InputKind::Membership { region, captures } => {
            out.u8(6)?;
            id(out, region.0)?;
            super::range(out, captures)
        }
        s::InputKind::LocalBorrow(region) => {
            out.u8(7)?;
            id(out, region.0)
        }
        s::InputKind::ScopedWellFormed { ty, frame } => {
            out.u8(8)?;
            id(out, ty.0)?;
            id(out, frame.0)
        }
    }
}
fn obligation_read(read: &mut Input<'_, '_>) -> Result<s::Obligation, &'static str> {
    let site = site_read(read)?;
    let kind = match read.reader.u8()? {
        0 => {
            let actual = g::TypeRef(read.id()?);
            let expected = g::TypeRef(read.id()?);
            // These are explicit raw adjustment fields, never implied permissions.
            let unsize = read.boolean()?;
            if unsize {
                read.require_provided("source adjusted coercion requires a wider source profile")?;
            }
            let reference = if read.boolean()? {
                read.require_provided("source adjusted coercion requires a wider source profile")?;
                Some(s::ReferenceConversion {
                    dereferences: read.id()?,
                    unsize: read.boolean()?,
                    two_phase: read.boolean()?,
                })
            } else {
                None
            };
            s::InputKind::Coercion {
                actual,
                expected,
                unsize,
                reference,
            }
        }
        1 => s::InputKind::WellFormed(g::TypeRef(read.id()?)),
        2 => s::InputKind::Equality(types::bound_read(read)?),
        3 => s::InputKind::Outlives(types::bound_read(read)?),
        4 => s::InputKind::TypeOutlives {
            ty: g::TypeRef(read.id()?),
            shorter: s::RegionRef(read.id()?),
        },
        tag @ (5..=8) => {
            read.require_regional("unknown source ordinary obligation kind")?;
            match tag {
                5 => s::InputKind::Variable {
                    slot: read.id()?,
                    needs_provenance: read.boolean()?,
                },
                6 => s::InputKind::Membership {
                    region: s::RegionRef(read.id()?),
                    captures: read.range()?,
                },
                7 => s::InputKind::LocalBorrow(s::RegionRef(read.id()?)),
                _ => s::InputKind::ScopedWellFormed {
                    ty: g::TypeRef(read.id()?),
                    frame: g::BinderRef(read.id()?),
                },
            }
        }
        _ => return Err("unknown source ordinary obligation kind"),
    };
    Ok(s::Obligation { site, kind })
}
fn normalization_write(out: &mut Writer, row: &s::NormalizationInput) -> Result<(), &'static str> {
    site_write(out, &row.site)?;
    write(out, &row.sources, site_write)?;
    write_copy(out, &row.owned, super::range)?;
    write_copy(out, &row.equalities, types::bound_write)?;
    write_copy(out, &row.outlives, types::bound_write)?;
    write_copy(out, &row.types, outlives_write)
}
fn normalization_read(read: &mut Input<'_, '_>) -> Result<s::NormalizationInput, &'static str> {
    let site = site_read(read)?;
    let sources = read.list(crates::MAX_COMPONENTS, 5, site_read)?;
    let owned = read.regional_list(crates::MAX_COMPONENTS, 8, Input::range)?;
    Ok(s::NormalizationInput {
        site,
        sources,
        owned,
        equalities: read.list(crates::MAX_COMPONENTS, 8, types::bound_read)?,
        outlives: read.list(crates::MAX_COMPONENTS, 8, types::bound_read)?,
        types: read.list(crates::MAX_COMPONENTS, 8, outlives_read)?,
    })
}
fn outlives_write(out: &mut Writer, row: s::TypeOutlives) -> Result<(), &'static str> {
    id(out, row.ty.0)?;
    id(out, row.shorter.0)
}
fn outlives_read(read: &mut Input<'_, '_>) -> Result<s::TypeOutlives, &'static str> {
    Ok(s::TypeOutlives {
        ty: g::TypeRef(read.id()?),
        shorter: s::RegionRef(read.id()?),
    })
}
fn builtins_write(out: &mut Writer, row: &s::BuiltinInputs) -> Result<(), &'static str> {
    environment_write(out, row.environment)?;
    write(out, &row.predicates, located_write)?;
    out.u8(u8::from(row.source_copy.is_some()))?;
    if let Some(copy) = &row.source_copy {
        out.u8(u8::from(copy.declaration.is_some()))?;
        if let Some(declaration) = copy.declaration {
            address_write(out, declaration)?;
        }
        write(out, &copy.goals, requirement_write)?;
    }
    Ok(())
}
fn builtins_read(read: &mut Input<'_, '_>) -> Result<s::BuiltinInputs, &'static str> {
    let environment = environment_read(read)?;
    let predicates = read.list(crates::MAX_COMPONENTS, 11, located_read)?;
    let source_copy = if read.boolean()? {
        Some(s::CopyQueries {
            declaration: if read.boolean()? {
                Some(address_read(read)?)
            } else {
                None
            },
            goals: read.list(crates::MAX_COMPONENTS, 6, types::requirement_read)?,
        })
    } else {
        None
    };
    Ok(s::BuiltinInputs {
        environment,
        predicates,
        source_copy,
    })
}
fn requirement_write(out: &mut Writer, row: &s::Requirement) -> Result<(), &'static str> {
    types::requirement_write(out, row)
}
