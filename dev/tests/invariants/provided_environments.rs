//! Real default bodies retain local assumptions without changing declaration permission.
use super::*;
use crate::{
    resolve::expansion::Report,
    sema::{function_context::FunctionContext, model::Function},
    types::parameters::{clauses::Clause, predicates::Bound},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn full_family() -> Result<(Report, crate::pipeline::Analysis, Function)> {
    let source = crate::test_source::fixture(
        "pa31/tests/spec/source-header-transport/semantic-provider-qualified.rs",
    )?;
    let report = crate::test_source::source_provider(&source, "body_environment_family")?;
    let analysis = crate::pipeline::analyze_source(&report)?;
    let function = analysis
        .facts
        .functions
        .iter()
        .find(|function| {
            report
                .package
                .names
                .get(report.resolution.definitions[function.definition.0 as usize].name)
                == "clone_from"
        })
        .copied()
        .ok_or("actual provided clone_from")?;
    Ok((report, analysis, function))
}

#[test]
fn defining_environment_full_family_keeps_actual_default_local_to_body() -> Result {
    let (report, analysis, function) = full_family()?;
    let facts = &analysis.facts;
    let source = function.body.ok_or("actual default body")?;
    let FunctionContext::ConstTraitDefault { trait_definition } =
        facts.function_context(function.definition, source)?
    else {
        return Err("provided body lost its defining context".into());
    };
    let declaration = facts
        .generics
        .environments
        .get(&function.definition)
        .ok_or("actual declaration environment")?;
    let body = facts
        .generics
        .body_environments
        .get(&function.definition)
        .ok_or("actual derived body environment")?;
    let Clause::Predicate(local) = facts
        .default_self_clause(function.definition, source)?
        .ok_or("actual local Self clause")?
    else {
        return Err("default Self clause acquired a foreign binder".into());
    };
    assert!(matches!(local.bound, Bound::MaybeConstTrait(_)));
    assert!(!Arc::ptr_eq(declaration, body));
    assert!(!declaration.assumes_predicate(local));
    assert!(body.assumes_predicate(local));
    assert!(declaration.body_aliases().is_none());
    let aliases = body.body_aliases().ok_or("actual body alias ledger")?;
    let scheme = facts
        .lifetimes
        .functions
        .get(&function.definition)
        .ok_or("actual lifetime scheme")?;
    assert!(scheme.total > 0);
    assert_eq!(aliases.len(), scheme.total as usize);
    for (slot, &alias) in aliases.ordered().iter().enumerate() {
        assert_eq!(aliases.slot(alias), Some(u16::try_from(slot)?));
    }
    let header = facts
        .function_header(function.definition)?
        .ok_or("actual callable header")?;
    assert!(
        header
            .environment
            .is_some_and(|actual| Arc::ptr_eq(actual, body))
    );
    assert_eq!(
        facts.associated.constant_trait(trait_definition),
        Some(true)
    );
    assert!(!facts.is_const_function(function.definition));
    let ordinary_impl = facts
        .functions
        .iter()
        .find(|candidate| {
            candidate.body.is_some()
                && report
                    .package
                    .names
                    .get(report.resolution.definitions[candidate.definition.0 as usize].name)
                    == "clone"
        })
        .ok_or("original ordinary u8 Clone body")?;
    assert!(!facts.is_const_function(ordinary_impl.definition));
    assert!(facts.functions.iter().any(|candidate| {
        candidate.body.is_none()
            && facts.associated.trait_owner(candidate.definition) == Some(trait_definition)
    }));
    Ok(())
}

#[test]
fn defining_environment_rejects_incomplete_actual_alias_scope_before_publication() -> Result {
    let (_, analysis, function) = full_family()?;
    let mut facts = analysis.facts;
    let declaration = Arc::clone(
        facts
            .generics
            .environments
            .get(&function.definition)
            .ok_or("actual declaration")?,
    );
    let original = Arc::clone(
        facts
            .generics
            .body_environments
            .get(&function.definition)
            .ok_or("actual body environment")?,
    );
    let aliases = original
        .body_aliases()
        .ok_or("actual aliases")?
        .ordered()
        .to_vec();
    let universals = u32::try_from(aliases.len())?;
    assert!(universals > 0);
    let clause = facts
        .default_self_clause(function.definition, function.body.ok_or("body")?)?
        .ok_or("actual defining Self clause")?;
    let prepared = Prepared::for_declaration(
        &mut facts,
        function.definition,
        Some(function.signature),
        universals,
        Some(Arc::clone(&declaration)),
        &[clause],
    )?;
    assert!(
        prepared
            .finish(universals, aliases[..aliases.len() - 1].to_vec())
            .is_err()
    );
    assert!(Arc::ptr_eq(
        facts
            .generics
            .body_environments
            .get(&function.definition)
            .ok_or("body")?,
        &original,
    ));
    assert!(Arc::ptr_eq(
        facts
            .generics
            .environments
            .get(&function.definition)
            .ok_or("declaration")?,
        &declaration,
    ));
    assert!(
        Prepared::for_declaration(
            &mut facts,
            function.definition,
            Some(function.signature),
            universals,
            None,
            &[clause],
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn defining_environment_ordinary_path_keeps_the_actual_declaration_arc() -> Result {
    let report = crate::test_source::resolve("fn ordinary<T>(value:T)->T{value}", true)?;
    let mut facts = crate::pipeline::analyze_source(&report)?.facts;
    let function = facts.functions.first().copied().ok_or("actual function")?;
    let declaration = Arc::clone(
        facts
            .generics
            .environments
            .get(&function.definition)
            .ok_or("declaration")?,
    );
    let result = Prepared::for_declaration(
        &mut facts,
        function.definition,
        Some(function.signature),
        0,
        Some(Arc::clone(&declaration)),
        &[],
    )?
    .finish(0, Vec::new())?;
    assert!(!result.changed);
    assert!(
        result
            .environment
            .as_ref()
            .is_some_and(|actual| Arc::ptr_eq(actual, &declaration))
    );
    assert!(
        !facts
            .generics
            .body_environments
            .contains_key(&function.definition)
    );
    Ok(())
}

#[test]
fn defining_environment_derives_builtin_equalities_from_actual_lifetime_inputs() -> Result {
    for (bounds, expected) in [("'a:'b,'b:'a,", true), ("'a:'b,", false)] {
        let source = format!(
            "fn correlated<'a,'b,T>(value:(&'a (),&'b (),T)) where {bounds} for<'c> (&'c (),&'c (),T):Copy {{}}"
        );
        let report = crate::test_source::resolve(&source, true)?;
        let facts = crate::pipeline::analyze_source(&report)?.facts;
        let function = facts.functions.first().copied().ok_or("actual function")?;
        let crate::types::Kind::Function { parameters, .. } = facts
            .types
            .kind(function.signature)
            .ok_or("actual signature")?
        else {
            return Err("actual declaration lost its signature type".into());
        };
        let actual = *facts
            .types
            .list(parameters)
            .ok_or("actual formal list")?
            .first()
            .ok_or("actual formal tuple")?;
        let declaration = facts
            .generics
            .environments
            .get(&function.definition)
            .ok_or("actual declaration environment")?;
        let body = facts
            .generics
            .body_environments
            .get(&function.definition)
            .ok_or("actual derived region environment")?;
        assert!(declaration.has_region_builtins());
        assert!(declaration.body_aliases().is_none());
        assert!(
            !declaration
                .assumed_with(&facts.types, actual, &mut None)?
                .copy
        );
        assert_eq!(
            body.assumed_with(&facts.types, actual, &mut None)?.copy,
            expected
        );
    }
    Ok(())
}
