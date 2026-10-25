//! A declaration selection and session policy never replace the reached caller.
use super::*;
use crate::{
    expand::config::Config,
    support::runtime_checks::RuntimeCheckConfig,
    types::{arguments::Argument, callable::Convention},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn report(name: &str, enabled: bool) -> Result<crate::resolve::expansion::Report> {
    let source = crate::test_source::fixture(&format!("pa31/tests/spec/ub-checks/{name}.rs"))?;
    let mut config = Config::default();
    config.set_runtime_checks(RuntimeCheckConfig::with_ub_checks(enabled));
    Ok(crate::test_source::source_provider_config(
        &source,
        "ub_checks",
        &config,
    )?)
}
fn target(
    report: &crate::resolve::expansion::Report,
    facts: &Declarations,
) -> Result<(DefId, Instance)> {
    let index = report
        .resolution
        .definitions
        .iter()
        .position(|d| report.package.names.get(d.name) == "ub_checks")
        .ok_or("actual intrinsic")?;
    Ok((
        DefId(u32::try_from(index)?),
        Instance::new(
            &facts.types,
            facts.definitions[index].ok_or("actual function item")?,
        )?,
    ))
}
#[test]
fn ub_selection_rechecks_actual_declaration_signature_source_and_full_arguments() -> Result {
    let report = report("local-fold", false)?;
    let mut facts = crate::pipeline::analyze_source(&report)?.facts;
    let (definition, instance) = target(&report, &facts)?;
    let Some(Kind::FunctionItem {
        item, signature, ..
    }) = facts.types.kind(instance.ty())
    else {
        return Err("item".into());
    };
    let no_parameters = facts.types.intern_list(&[])?;
    let wrong_signature = facts.types.intern(Kind::Function {
        bound_regions: 0,
        parameters: no_parameters,
        result: TypeId::UNIT,
        unsafe_: false,
        convention: Convention::RUST,
    })?;
    let extra = facts
        .types
        .intern_arguments(&[Argument::Type(TypeId::primitive(Primitive::U8))])?;
    let wrong_item = facts.types.intern(Kind::FunctionItem {
        item,
        signature: wrong_signature,
        arguments: crate::types::GenericArgs::EMPTY,
    })?;
    let wrong_arguments = facts.types.intern(Kind::FunctionItem {
        item,
        signature,
        arguments: extra,
    })?;
    let catalog = facts.source_bindings.as_deref().ok_or("catalog")?;
    assert!(catalog.runtime_check_selection(&facts, instance)?.is_some());
    for ty in [wrong_item, wrong_arguments] {
        assert!(
            catalog
                .runtime_check_selection(&facts, Instance::new(&facts.types, ty)?)
                .is_err()
        );
    }
    let index = *facts.function_indices.get(&definition).ok_or("function")?;
    let original = facts.functions[index];
    facts.functions[index].signature = wrong_signature;
    assert!(
        facts
            .source_bindings
            .as_deref()
            .ok_or("catalog")?
            .runtime_check_selection(&facts, instance)
            .is_err()
    );
    facts.functions[index] = original;
    facts.functions[index].node.node =
        crate::syntax::ast::NodeId::new(original.node.node.get() + 1);
    assert!(
        facts
            .source_bindings
            .as_deref()
            .ok_or("catalog")?
            .runtime_check_selection(&facts, instance)
            .is_err()
    );
    facts.functions[index] = original;
    assert!(
        facts
            .source_bindings
            .as_deref()
            .ok_or("catalog")?
            .runtime_check_selection(&facts, instance)?
            .is_some()
    );
    Ok(())
}
#[test]
fn immutable_runtime_profile_and_preserve_policy_reject_catalog_and_lowering_replay() -> Result {
    let original = report("local-fold", false)?;
    let opposite = report("local-fold", true)?;
    let preserve = report("local-preserve", false)?;
    let analysis = crate::pipeline::analyze_source(&original)?;
    let mut facts = analysis.facts;
    let catalog = facts.source_bindings.as_deref().ok_or("catalog")?;
    catalog.require_provider_report(&original)?;
    for changed in [&opposite, &preserve] {
        assert_eq!(
            catalog.require_provider_report(changed),
            Err("source catalog belongs to another runtime-check session policy")
        );
    }
    let error = crate::lower::instances::build_roots_prepared(
        &opposite,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[],
        },
        analysis.prepared,
    )
    .err()
    .ok_or("foreign generation profile accepted")?;
    assert!(
        error
            .to_string()
            .contains("another runtime-check session policy")
    );
    Ok(())
}
#[test]
fn actual_imported_body_carrier_requires_its_untransported_runtime_policy() -> Result {
    use crate::resolve::body_origin::{
        BodyBinding as Binding, CatalogSlot, DeclarationBinding, Input,
    };
    use crate::resolve::model::{Access, DefKind, Definition, Namespace, ScopeId};
    use crate::support::crates::{
        CrateIdentity,
        owner_catalog::{
            BodyCatalog, BodyKind, BodyOwner, BodyRef, DeclarationBody, DeclarationRef,
        },
    };
    let definitions = vec![Definition {
        name: crate::support::names::NameId(0),
        namespace: Namespace::Value,
        kind: DefKind::Function,
        scope: ScopeId(0),
        node: None,
        access: Access::ExternalPrivate,
        members: None,
    }];
    let owners = BodyCatalog::new(
        CrateIdentity::new("provider", "ub_policy")?,
        &[DeclarationBody::Function],
        vec![BodyOwner {
            declaration: DeclarationRef(0),
            parent: None,
            kind: BodyKind::Function,
            ordinal: 0,
        }],
    )?;
    let mut origin_slot = CatalogSlot::default();
    let origins = origin_slot.ensure()?;
    let id = origins
        .install(
            &definitions,
            Input {
                owners,
                edition: crate::support::edition::Edition::Rust2024,
                definitions: vec![DefId(0)],
                declarations: vec![DeclarationBinding {
                    kind: DefKind::Function,
                    sites: 0..1,
                }],
                bindings: vec![Binding {
                    unit: BodyRef(0),
                    sites: 0..1,
                }],
                files: vec![],
                labels: vec![None],
                declaration_labels: vec![None],
            },
        )?
        .body(BodyRef(0))?;
    let report = report("local-fold", false)?;
    let facts = crate::pipeline::analyze_source(&report)?.facts;
    let error = facts
        .source_bindings
        .as_deref()
        .ok_or("catalog")?
        .runtime_check_binding(origins.origin(id)?)
        .err()
        .ok_or("imported policy defaulted")?;
    assert_eq!(
        error,
        "imported runtime-check operations require transported policy provenance"
    );
    Ok(())
}
