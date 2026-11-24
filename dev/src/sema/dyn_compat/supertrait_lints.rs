//! Use the checked direct bounds and the existing dyn-compatibility query.
use super::{Declarations, Error as Compatibility, Queries, predicate};
use crate::{
    resolve::{
        expansion::Report,
        lint_policy::Level,
        model::{DefId, DefKind},
        package::Error,
    },
    support::{bindings::LangItem, diagnostics::WarningCode},
    types::ItemId,
};

pub(in crate::sema) fn check(report: &Report, facts: &mut Declarations) -> Result<(), Error> {
    if !facts.safety_policies.checks_multiple_supertraits() {
        return Ok(());
    }
    let defaults = [LangItem::PointeeSized, LangItem::MetaSized, LangItem::Sized].map(|role| {
        report
            .resolution
            .source_bindings
            .as_deref()?
            .language(role)
            .map(|declaration| ItemId(declaration.definition.0))
    });
    let mut queries = Queries::default();
    for (index, definition) in report.resolution.definitions.iter().enumerate() {
        let Some(key) = definition.node else {
            continue;
        };
        if definition.kind != DefKind::Trait
            || facts.safety_policies.multiple_supertrait_upcastable(key) == Level::Allow
        {
            continue;
        }
        let id = DefId(
            u32::try_from(index)
                .map_err(|_| Error::unlocated("supertrait lint identity exceeds 32 bits"))?,
        );
        if !has_multiple(facts, id, &defaults)
            .map_err(|error| Declarations::error(report, key, error.to_string()))?
        {
            continue;
        }
        match queries.demand(report, facts, id) {
            Ok(_) => {}
            Err(Compatibility::Incompatible { .. }) => continue,
            Err(error) => return Err(Declarations::error(report, key, error.to_string())),
        }
        let unit = report.package.unit(key.file);
        facts.must_use_diagnostic(
            report,
            key,
            unit.configured.ast.node(key.node).span.start,
            0,
            WarningCode::MultipleSupertraitUpcastable,
            "dyn-compatible trait has multiple direct supertraits".into(),
        )?;
    }
    Ok(())
}

fn has_multiple(
    facts: &Declarations,
    id: DefId,
    defaults: &[Option<ItemId>; 3],
) -> Result<bool, Compatibility> {
    let Some(header) = facts
        .generics
        .traits
        .as_ref()
        .and_then(|traits| traits.header(ItemId(id.0)))
    else {
        return Ok(false);
    };
    let mut first = false;
    for (index, input) in header.inputs.view().iter().enumerate() {
        if matches!(
            input.value,
            crate::types::parameters::clauses::Clause::UnstableFeature(_)
        ) {
            continue;
        }
        let value = predicate(&facts.types, input.value)?;
        if let Some((application, _)) = value.bound.trait_parts()
            && header.implies_input(&facts.types, index, input)?
        {
            let trait_ = facts
                .types
                .trait_application(application)
                .ok_or("direct supertrait has no actual trait application")?
                .item;
            if !defaults.contains(&Some(trait_)) {
                if first {
                    return Ok(true);
                }
                first = true;
            }
        }
    }
    Ok(false)
}
