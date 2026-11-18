//! Retain the authored personality as a native definition, with no unwind consumer.
use super::{Declaration, Error, LangItem, Report};
use crate::{
    sema::{
        foreign::{Function, Registry},
        header_attributes,
        model::Declarations,
    },
    support::linkage::{Linkage, Naming},
    syntax::item::Safety,
    types::{Kind, callable::Convention},
};

pub(crate) const SYMBOL: &str = "rust_eh_personality";

pub(crate) fn complete(report: &Report, facts: &mut Declarations) -> Result<(), Error> {
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(());
    };
    let Some(declaration) = catalog.language(LangItem::EhPersonality) else {
        return Ok(());
    };
    let error = |message| Declarations::error(report, declaration.origin, message);
    catalog.require_provider_report(report).map_err(error)?;
    header_attributes::charge_source_contract_query(facts, 8)?;
    let definition = declaration.definition;
    super::headers::functions::complete(report, facts, catalog, LangItem::EhPersonality)?;
    let function = checked(facts, declaration).map_err(error)?;
    let payments = header_attributes::source_payments(facts.must_use_contracts.as_deref())?;
    if let Some(registry) = facts.foreign.as_deref_mut() {
        registry
            .bind_personality(definition, function, |growth| {
                payments.charge(growth.work, growth.bytes)
            })
            .map_err(error)?;
    } else {
        payments.charge(1, size_of::<Registry>()).map_err(error)?;
        let mut registry = Registry::default();
        registry
            .bind_personality(definition, function, |growth| {
                payments.charge(growth.work, growth.bytes)
            })
            .map_err(error)?;
        facts.foreign = Some(Box::new(registry));
    }
    Ok(())
}

fn checked(facts: &Declarations, declaration: Declaration) -> Result<Function, &'static str> {
    let header = super::headers::functions::current(facts, declaration)?;
    let definition = declaration.definition;
    if !facts.typed_function_bodies.contains(&definition)
        || facts
            .generics
            .parameters_in_scope(definition)
            .next()
            .is_some()
        || !matches!(
            facts.types.kind(header.signature),
            Some(Kind::Function {
                bound_regions: 0,
                convention: Convention::C,
                ..
            })
        )
    {
        return Err(
            "source eh_personality native definition requires its checked monomorphic C body",
        );
    }
    let unsafe_ = matches!(
        facts.types.kind(header.signature),
        Some(Kind::Function { unsafe_: true, .. })
    );
    Ok(Function {
        signature: header.signature,
        safety: if unsafe_ {
            Safety::Unsafe
        } else {
            Safety::Default
        },
        // The paid registry transaction installs the canonical exact name.
        naming: Naming::Rust,
        linkage: Linkage::Export,
    })
}

pub(in crate::sema) fn require_transport(facts: &Declarations) -> Result<(), &'static str> {
    if let Some(catalog) = facts.source_bindings.as_deref()
        && catalog.role(LangItem::EhPersonality)?.is_some()
    {
        return Err("selected Source eh_personality requires its checked artifact transport");
    }
    Ok(())
}
