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
    support::{call_traits::CallTrait, constness::Constness},
};

/// Only actual selected copying/conversion headers enter their existing ledger.
pub(in crate::sema) fn selected_roles(
    report: &Report,
    facts: &mut Declarations,
) -> Result<(), Error> {
    if report.package.staged_consumer().is_none() {
        return Err(Error::unlocated(
            "selected role preparation requires its actual private consumer",
        ));
    }
    let mut steps = 0usize;
    let checked = (|| {
        let catalog = catalog(report, facts)?;
        let mut complete = super::RoleSet::EMPTY;
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
                complete.insert(LangItem::CloneFn);
            } else if !header.members.is_empty() {
                return Err("source copying marker must not declare members");
            }
            complete.insert(role);
        }
        if catalog.role(LangItem::CloneFn)?.is_some() && !complete.contains(LangItem::CloneFn) {
            return Err("source language member has no validated actual trait owner");
        }
        if let Some(declaration) = catalog.role(LangItem::From)? {
            conversion(report, facts, declaration, &mut steps)?;
            complete.insert(LangItem::From);
        }
        Ok(complete)
    })();
    let fresh = facts
        .source_bindings
        .as_deref()
        .is_some_and(|catalog| catalog.traits.is_none());
    if checked.is_ok() {
        steps = steps.saturating_add(2 * (super::completion_set::WORDS - 1));
    }
    crates::charge_source_headers(
        facts,
        steps,
        if checked.as_ref().is_ok_and(|roles| !roles.is_empty()) && fresh {
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
        prior.roles.extend(complete);
    } else if !complete.is_empty() {
        catalog.traits = Some(Box::new(Validated {
            roles: complete,
            methods: [None; 3],
            pointee: None,
            discriminant: None,
            range: None,
            field: None,
            comparisons: crate::sema::compiler_items::comparison_contract::Sealed::default(),
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
    let (mut roles, callables, needs_pointee) = checked.map_err(Error::unlocated)?;
    let pointee = if needs_pointee {
        super::super::pointee::prepare_imported(report, facts, &mut roles)?
    } else {
        None
    };
    let entries: Vec<_> = [CallTrait::FnOnce, CallTrait::FnMut, CallTrait::Fn]
        .into_iter()
        .filter_map(|kind| {
            Some((
                callables.roles[kind as usize]?,
                kind,
                callables.methods[kind as usize]?,
            ))
        })
        .collect();
    facts
        .associated
        .register_imported_callables(&entries, callables.output)
        .map_err(Error::unlocated)?;
    facts
        .source_bindings
        .as_deref_mut()
        .ok_or_else(|| Error::unlocated("source role catalog disappeared"))?
        .traits = Some(Box::new(Validated {
        roles,
        methods: callables.methods,
        pointee,
        discriminant: None,
        range: None,
        field: None,
        comparisons: crate::sema::compiler_items::comparison_contract::Sealed::default(),
    }));
    Ok(())
}

/// `Destruct` and the auto markers, each admitted as an empty free trait;
/// version thirteen admits Send, Sync, Unpin and `UnsafeUnpin` as Freeze is.
const FREE_MARKERS: [LangItem; 6] = [
    LangItem::Destruct,
    LangItem::Freeze,
    LangItem::Send,
    LangItem::Sync,
    LangItem::Unpin,
    LangItem::UnsafeUnpin,
];

fn trait_headers(
    report: &Report,
    facts: &Declarations,
    steps: &mut usize,
) -> Result<(super::RoleSet, super::invocation::Imported, bool), &'static str> {
    let catalog = catalog(report, facts)?;
    if catalog.traits.is_some() {
        return Err("source roles were validated twice");
    }
    let mut complete = super::RoleSet::EMPTY;
    let mut needs_pointee = false;
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
                copying_role(report, facts, catalog, (role, declaration), steps)?;
                complete.insert(role);
                if role == LangItem::Clone {
                    complete.insert(LangItem::CloneFn);
                }
            }
            LangItem::CloneFn => {}
            role if FREE_MARKERS.contains(&role) => {
                free_marker(report, facts, catalog, role, declaration, steps)?;
                complete.insert(role);
            }
            LangItem::Future
            | LangItem::FutureOutput
            | LangItem::FuturePoll
            | LangItem::AsyncDropInPlace => {
                facts
                    .associated
                    .imported_owner()
                    .ok_or("imported compiler role lost its actual owner/schema graph")?
                    .compiler_operation_role(facts, declaration, role, steps)?;
                complete.insert(role);
            }
            LangItem::BikeshedGuaranteedNoDrop => {
                let header = Trait::current(report, facts, declaration, steps)?;
                header.profile(&facts.types, header.flags, 0)?;
                complete.insert(role);
            }
            LangItem::From => {
                conversion(report, facts, declaration, steps)?;
                complete.insert(role);
            }
            role if super::super::operator_contract::is_operator(role) => {
                super::super::operator_contract::imported(report, facts, declaration, role, steps)?;
                complete.insert(role);
            }
            LangItem::Tuple => {
                tuple_marker(report, facts, declaration, steps)?;
                complete.insert(role);
            }
            // Grouped roles are checked together once the loop completes.
            LangItem::FnOnce
            | LangItem::FnMut
            | LangItem::Fn
            | LangItem::FnOnceOutput
            | LangItem::RangeMin
            | LangItem::RangeMax
            | LangItem::RangeSub
            | LangItem::ConstParamTy
            | LangItem::StructuralPartialEq => {}
            LangItem::PointeeTrait | LangItem::MetadataType | LangItem::DynMetadata => {
                needs_pointee = true;
            }
            role if role == LangItem::Panic || LangItem::GUARD_PANICS.contains(&role) => {
                super::super::runtime_panics::imported(facts, declaration, role, steps)?;
                complete.insert(role);
            }
            role if role.is_runtime_import() => {
                super::super::headers::weak_imports::imported(facts, declaration, steps)?;
                complete.insert(role);
            }
            role if nominal_role(role) => {
                check_nominal_role(facts, catalog, declaration, role, steps)?;
                complete.insert(role);
            }
            _ => return Err("imported source role requires its wider explicit header profile"),
        }
    }
    if catalog.role(LangItem::CloneFn)?.is_some() && !complete.contains(LangItem::CloneFn) {
        return Err("source language member has no validated actual trait owner");
    }
    let callables = grouped_roles(report, facts, catalog, steps, &mut complete)?;
    Ok((complete, callables, needs_pointee))
}

fn copying_role(
    report: &Report,
    facts: &Declarations,
    catalog: &Catalog,
    (role, declaration): (LangItem, super::super::Role),
    steps: &mut usize,
) -> Result<(), &'static str> {
    let header = Trait::current(report, facts, declaration, steps)?;
    header.profile(&facts.types, flags(), 0)?;
    let expected = match role {
        LangItem::Clone => LangItem::Sized,
        LangItem::Copy => LangItem::Clone,
        _ => LangItem::PointeeSized,
    };
    *steps = steps.saturating_add(header.inputs.len());
    copying::require_header_super(facts, catalog, &header, expected, Constness::Ordinary)?;
    if role == LangItem::Clone {
        clone_member(facts, catalog, &header, steps)?;
    } else if !header.members.is_empty() {
        return Err("source copying marker must not declare members");
    }
    Ok(())
}

fn nominal_role(role: LangItem) -> bool {
    matches!(
        role,
        LangItem::PhantomData | LangItem::TypeInfo | LangItem::TypeId
    ) || LangItem::NOMINAL_WRAPPERS.contains(&role)
}

fn check_nominal_role(
    facts: &Declarations,
    catalog: &Catalog,
    declaration: super::super::Role,
    role: LangItem,
    steps: &mut usize,
) -> Result<(), &'static str> {
    match role {
        LangItem::PhantomData => {
            super::super::phantom::imported(facts, catalog, declaration, steps)
        }
        LangItem::TypeInfo | LangItem::TypeId => {
            super::super::reflection::imported(facts, declaration, steps)
        }
        _ => super::super::wrappers::imported(facts, declaration, role, steps),
    }
}

fn grouped_roles(
    report: &Report,
    facts: &Declarations,
    catalog: &Catalog,
    steps: &mut usize,
    complete: &mut super::RoleSet,
) -> Result<super::invocation::Imported, &'static str> {
    super::parameter_types::imported(report, facts, catalog, steps, complete)?;
    if super::range_members::imported(report, facts, catalog, steps)? {
        for role in super::range_members::ROLES {
            complete.insert(role);
        }
    }
    let callables = super::invocation::imported(report, facts, catalog, steps)?;
    for (role, admitted) in [
        (
            LangItem::FnOnce,
            callables.roles[CallTrait::FnOnce as usize],
        ),
        (LangItem::FnMut, callables.roles[CallTrait::FnMut as usize]),
        (LangItem::Fn, callables.roles[CallTrait::Fn as usize]),
        (LangItem::FnOnceOutput, callables.output),
    ] {
        if admitted.is_some() {
            complete.insert(role);
        }
    }
    Ok(callables)
}

/// The imported Tuple marker: an empty, parameterless deny-implementation trait.
fn tuple_marker(
    report: &Report,
    facts: &Declarations,
    declaration: super::super::Role,
    steps: &mut usize,
) -> Result<(), &'static str> {
    facts.associated.require_imported_role(declaration)?;
    let header = Trait::current(report, facts, declaration, steps)?;
    header.profile(&facts.types, flags(), 0)?;
    if header.written.parameters != 0
        || header.written.supertraits != 0
        || !header.members.is_empty()
        || !facts
            .associated
            .imported_denies_implementation(declaration.definition)?
    {
        return Err("imported Tuple marker changed its empty deny-implementation declaration");
    }
    Ok(())
}

fn conversion(
    report: &Report,
    facts: &Declarations,
    declaration: super::super::Role,
    steps: &mut usize,
) -> Result<(), &'static str> {
    *steps = steps.saturating_add(64 + LangItem::ALL.len());
    facts.associated.require_imported_role(declaration)?;
    facts
        .associated
        .imported_owner()
        .ok_or("imported From lost its actual header owner")?
        .conversion_role(&facts.types, declaration)?;
    let header = Trait::current(report, facts, declaration, steps)?;
    super::conversion::header(facts, &header)
}

fn free_marker(
    report: &Report,
    facts: &Declarations,
    catalog: &Catalog,
    role: LangItem,
    declaration: super::super::Role,
    steps: &mut usize,
) -> Result<(), &'static str> {
    // The existing neutral-attribute adapter visits the fixed role table.
    *steps = steps.saturating_add(4 + LangItem::ALL.len());
    let client = report
        .package
        .source_client()
        .ok_or("free callable marker requires its actual source client")?;
    let imported = report
        .resolution
        .crates
        .as_deref()
        .ok_or("free callable marker lacks its actual import inventory")?;
    let index = client.require_inventory(imported.dependencies())?;
    let descriptor = imported.dependencies().artifacts()[index as usize]
        .source_descriptor()
        .ok_or("free callable marker lacks its actual source descriptor")?;
    if !descriptor.profile().unconditional_callables() {
        return Err("free callable marker requires its explicit version-seven profile");
    }
    let header = Trait::current(report, facts, declaration, steps)?;
    header.profile(
        &facts.types,
        Flags {
            unsafe_: matches!(
                role,
                LangItem::Freeze | LangItem::Send | LangItem::Sync | LangItem::UnsafeUnpin
            ),
            constant: role == LangItem::Destruct,
            auto_: role != LangItem::Destruct,
        },
        0,
    )?;
    *steps = steps.saturating_add(header.inputs.len());
    if !header.members.is_empty() || header.written.parameters != 0 {
        return Err("free callable marker changed its empty actual declaration");
    }
    if role == LangItem::Destruct {
        if !facts
            .associated
            .imported_denies_implementation(declaration.definition)?
            || !header
                .attributes
                .contains(crate::resolve::bindings::HeaderAttribute::DynIncompatible)
        {
            return Err("source Destruct changed its actual declaration restrictions");
        }
        copying::require_header_super(
            facts,
            catalog,
            &header,
            LangItem::PointeeSized,
            Constness::Ordinary,
        )?;
    } else if header.written.supertraits != 0 {
        return Err(if role == LangItem::Freeze {
            "source Freeze changed its actual empty supertrait list"
        } else {
            "source auto marker changed its actual empty supertrait list"
        });
    }
    Ok(())
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
