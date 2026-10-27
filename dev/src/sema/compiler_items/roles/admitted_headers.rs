//! First-profile imported role headers are declarations, never executable proof.
use super::{Catalog, Declarations, Error, LangItem, Report, Validated, copying};
use crate::{
    sema::{
        compiler_items::{
            headers::{Flags, Method, Trait, clone_method},
            sizing,
        },
        crates,
    },
    support::constness::Constness,
};

/// Only actual selected copying declarations enter their existing ledger.
pub(in crate::sema) fn selected_copying(
    report: &Report,
    facts: &mut Declarations,
) -> Result<(), Error> {
    if report.package.staged_consumer().is_none() {
        return Err(Error::unlocated(
            "selected Copy preparation requires its actual private consumer",
        ));
    }
    let mut steps = 0usize;
    let checked = (|| {
        let catalog = catalog(report, facts)?;
        let mut complete = 0u64;
        for role in [LangItem::Clone, LangItem::Copy] {
            steps += 1;
            let Some(declaration) = catalog.role(role)? else {
                continue;
            };
            let header = Trait::current(report, facts, declaration, &mut steps)?;
            header.profile(
                &facts.types,
                Flags {
                    unsafe_: false,
                    constant: role == LangItem::Clone && header.flags.constant,
                    auto_: false,
                },
                0,
            )?;
            steps = steps.saturating_add(header.inputs.len());
            copying::require_header_super(
                facts,
                catalog,
                &header,
                if role == LangItem::Clone {
                    LangItem::Sized
                } else {
                    LangItem::Clone
                },
                Constness::Ordinary,
            )?;
            if role == LangItem::Clone {
                clone_member(facts, catalog, &header, &mut steps)?;
                complete |= super::bit(LangItem::CloneFn);
            } else if !header.members.is_empty() {
                return Err("source copying marker must not declare members");
            }
            complete |= super::bit(role);
        }
        if catalog.role(LangItem::CloneFn)?.is_some()
            && complete & super::bit(LangItem::CloneFn) == 0
        {
            return Err("source language member has no validated actual trait owner");
        }
        Ok(complete)
    })();
    let fresh = facts
        .source_bindings
        .as_deref()
        .is_some_and(|catalog| catalog.traits.is_none());
    crates::charge_source_headers(
        facts,
        steps,
        if checked.as_ref().is_ok_and(|roles| *roles != 0) && fresh {
            size_of::<Validated>()
        } else {
            0
        },
    )?;
    let complete = checked.map_err(Error::unlocated)?;
    let catalog = facts
        .source_bindings
        .as_deref_mut()
        .ok_or_else(|| Error::unlocated("selected Copy catalog disappeared"))?;
    if let Some(prior) = catalog.traits.as_deref_mut() {
        prior.roles |= complete;
    } else if complete != 0 {
        catalog.traits = Some(Box::new(Validated {
            roles: complete,
            methods: [None; 3],
            pointee: None,
            discriminant: None,
        }));
    }
    Ok(())
}

pub(in crate::sema::compiler_items) fn sizing(
    report: &Report,
    facts: &mut Declarations,
) -> Result<(), Error> {
    let mut steps = 0;
    let checked = (|| {
        let catalog = catalog(report, facts)?;
        if catalog.sizing.is_some() {
            return Err("source sizing declarations were validated twice");
        }
        let mut definitions = [crate::resolve::model::DefId(0); 3];
        for (index, role) in [LangItem::Sized, LangItem::MetaSized, LangItem::PointeeSized]
            .into_iter()
            .enumerate()
        {
            steps += 1;
            let declaration = catalog.role(role)?.ok_or(
                "source provider requires distinct Sized, MetaSized and PointeeSized declarations",
            )?;
            let header = Trait::current(report, facts, declaration, &mut steps)?;
            header.profile(&facts.types, flags(), 0)?;
            if header.written.parameters != 0 || !header.members.is_empty() {
                return Err("source sizing trait header is outside the supported provider profile");
            }
            if header.attributes.requires_sizing_profile()
                && !facts
                    .associated
                    .imported_denies_implementation(declaration.definition)?
            {
                return Err(
                    "source sizing attributes require the actual deny-implementation contract",
                );
            }
            steps = steps.saturating_add(header.inputs.len());
            sizing::hierarchy(facts, catalog, role, declaration.definition, &header)?;
            definitions[index] = declaration.definition;
        }
        if definitions[0] == definitions[1]
            || definitions[0] == definitions[2]
            || definitions[1] == definitions[2]
        {
            return Err("source sizing identities must be distinct");
        }
        Ok(definitions)
    })();
    crates::charge_source_headers(facts, steps, 0)?;
    let definitions = checked.map_err(Error::unlocated)?;
    facts
        .source_bindings
        .as_deref_mut()
        .ok_or_else(|| Error::unlocated("source role catalog disappeared"))?
        .sizing = Some(definitions);
    Ok(())
}

pub(in crate::sema::compiler_items) fn traits(
    report: &Report,
    facts: &mut Declarations,
) -> Result<(), Error> {
    let mut steps = 0;
    let checked = trait_headers(report, facts, &mut steps);
    let bytes = if checked.is_ok() {
        size_of::<Validated>()
    } else {
        0
    };
    crates::charge_source_headers(facts, steps, bytes)?;
    let roles = checked.map_err(Error::unlocated)?;
    facts
        .source_bindings
        .as_deref_mut()
        .ok_or_else(|| Error::unlocated("source role catalog disappeared"))?
        .traits = Some(Box::new(Validated {
        roles,
        methods: [None; 3],
        pointee: None,
        discriminant: None,
    }));
    Ok(())
}

fn trait_headers(
    report: &Report,
    facts: &Declarations,
    steps: &mut usize,
) -> Result<u64, &'static str> {
    let catalog = catalog(report, facts)?;
    if catalog.traits.is_some() {
        return Err("source roles were validated twice");
    }
    let mut complete = 0;
    for role in LangItem::ALL {
        *steps = steps.saturating_add(1);
        let Some(declaration) = catalog.role(role)? else {
            continue;
        };
        match role {
            LangItem::Sized | LangItem::MetaSized | LangItem::PointeeSized => {
                catalog.validated_role(role)?;
            }
            LangItem::Clone | LangItem::Copy | LangItem::LegacyReceiver => {
                let header = Trait::current(report, facts, declaration, steps)?;
                header.profile(&facts.types, flags(), 0)?;
                let expected = match role {
                    LangItem::Clone => LangItem::Sized,
                    LangItem::Copy => LangItem::Clone,
                    _ => LangItem::PointeeSized,
                };
                *steps = steps.saturating_add(header.inputs.len());
                copying::require_header_super(
                    facts,
                    catalog,
                    &header,
                    expected,
                    Constness::Ordinary,
                )?;
                if role == LangItem::Clone {
                    clone_member(facts, catalog, &header, steps)?;
                    complete |= crate::sema::compiler_items::role_bit(LangItem::CloneFn);
                } else if !header.members.is_empty() {
                    return Err("source copying marker must not declare members");
                }
                complete |= crate::sema::compiler_items::role_bit(role);
            }
            LangItem::CloneFn => {}
            _ => return Err("imported source role requires its wider explicit header profile"),
        }
    }
    if catalog.role(LangItem::CloneFn)?.is_some()
        && complete & (crate::sema::compiler_items::role_bit(LangItem::CloneFn)) == 0
    {
        return Err("source language member has no validated actual trait owner");
    }
    Ok(complete)
}

fn clone_member(
    facts: &Declarations,
    catalog: &Catalog,
    header: &Trait<'_>,
    steps: &mut usize,
) -> Result<(), &'static str> {
    *steps = steps.saturating_add(1);
    let method = catalog
        .role(LangItem::CloneFn)?
        .ok_or("source Clone requires its actual clone_fn declaration")?;
    let function = facts
        .declaration_header(method.definition)?
        .ok_or("source clone_fn signature is not prepared")?;
    if function.declaration != method.origin || function.body.is_some() {
        return Err("imported required clone_fn changed its actual declaration or body absence");
    }
    let parameters = facts
        .generics
        .parameters
        .get(&method.definition)
        .ok_or("imported clone_fn omitted its actual complete formal contract")?;
    clone_method(
        &facts.types,
        header,
        &Method {
            definition: method.definition,
            parent: method.parent,
            signature: function.signature,
            parameters: parameters.len(),
        },
    )
}

fn flags() -> Flags {
    Flags {
        unsafe_: false,
        constant: false,
        auto_: false,
    }
}
fn catalog<'a>(report: &Report, facts: &'a Declarations) -> Result<&'a Catalog, &'static str> {
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or("imported roles lack their actual catalog")?;
    catalog.require_provider_report(report)?;
    Ok(catalog)
}
