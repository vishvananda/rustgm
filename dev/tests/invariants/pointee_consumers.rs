//! Both normalization engines consume genuine source roles and complete applications.
use super::{LangItem, MetadataPlan, MetadataStep};
use crate::{
    pipeline,
    resolve::expansion::Report,
    sema::{
        declarations, header_attributes,
        model::Declarations,
        nominals::traits::normal_form,
        traits::{assumptions::Assumptions, implementations::Availability, selection},
    },
    types::{
        GenericArgs, ItemId, Kind, Primitive, TypeId,
        infer::{Context, Instantiation},
        parameters::predicates::{Bound, Predicate},
        projection,
        substitute::{Normalizer, Resolution, Step},
    },
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn provider(name: &str) -> TestResult<Report> {
    let source =
        crate::test_source::fixture(&format!("pa31/tests/spec/pointee-normalization/{name}.rs"))?;
    Ok(crate::test_source::source_provider(
        &source,
        "pointee_normalization",
    )?)
}
fn application(facts: &mut Declarations, subject: TypeId) -> TestResult<projection::Application> {
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or("actual source catalog")?;
    let parent = catalog
        .validated_language(LangItem::PointeeTrait)?
        .ok_or("actual Pointee")?;
    let member = catalog
        .validated_language(LangItem::MetadataType)?
        .ok_or("actual Metadata")?;
    let trait_application =
        facts
            .types
            .intern_trait_application(crate::types::traits::Application {
                item: ItemId(parent.definition.0),
                arguments: GenericArgs::EMPTY,
            })?;
    Ok(projection::Application {
        item: ItemId(member.definition.0),
        subject,
        trait_application,
        arguments: GenericArgs::EMPTY,
    })
}
#[test]
fn canonical_projection_argument_search_and_real_body_use_share_source_authority() -> TestResult {
    let report = provider("stack-selection")?;
    let capture = crate::sema::copy_trace::Capture::new()?;
    let mut analysis = pipeline::analyze_source(&report)?;
    let application = application(&mut analysis.facts, TypeId::primitive(Primitive::U8))?;
    let root = analysis.facts.types.intern(Kind::Projection(application))?;
    let mut fold = Normalizer::default();
    let Step::Resolve(request) = fold.start(&mut analysis.facts.types, root)? else {
        return Err("real projection request".into());
    };
    let normal_form::Progress::Ready(projected) = normal_form::begin(
        &mut analysis.facts,
        None,
        request,
        1,
        false,
        Availability::Validated,
    )?
    else {
        return Err("complete declaration unexpectedly pending".into());
    };
    let unit = TypeId::UNIT;
    assert_eq!(projected.resolution, Some(Resolution::Type(unit)));
    assert!(matches!(
        projected.source,
        Some(selection::Source::Pointee(_))
    ));
    search_argument(&report, &mut analysis.facts, root)?;
    role_publication_preserves_the_existing_owner(&report, &mut analysis.facts)?;
    let events = capture.rows()?;
    assert!(
        events
            .iter()
            .any(|row| row.starts_with("source-metadata-outer")),
        "{events:?}"
    );
    assert!(
        events
            .iter()
            .any(|row| row.starts_with("source-metadata-search")),
        "{events:?}"
    );
    Ok(())
}
fn role_publication_preserves_the_existing_owner(
    report: &Report,
    facts: &mut Declarations,
) -> TestResult {
    use crate::sema::compiler_items::roles;
    let ledger = facts
        .source_bindings
        .as_deref()
        .and_then(|catalog| catalog.traits.as_deref())
        .ok_or("validated source roles")?;
    let owner = std::rc::Rc::clone(ledger.pointee.as_ref().ok_or("actual Pointee owner")?);
    let roles = ledger.roles;
    let mask = (1 << LangItem::PointeeTrait.index())
        | (1 << LangItem::MetadataType.index())
        | (1 << LangItem::DynMetadata.index());
    assert_ne!(roles & (1 << LangItem::Copy.index()), 0);
    assert!(
        roles::validate(report, facts).is_err(),
        "duplicate regular validation succeeded"
    );
    {
        let ledger = facts
            .source_bindings
            .as_deref_mut()
            .and_then(|catalog| catalog.traits.as_deref_mut())
            .ok_or("same role ledger")?;
        ledger.roles = mask;
        ledger.methods[0] = Some(owner.plan.member.definition);
    }
    assert!(
        roles::validate(report, facts).is_err(),
        "foreign callable slot accepted as early Pointee family"
    );
    {
        let ledger = facts
            .source_bindings
            .as_deref_mut()
            .and_then(|catalog| catalog.traits.as_deref_mut())
            .ok_or("same role ledger")?;
        ledger.methods = [None; 3];
        ledger.pointee = None;
    }
    assert!(
        roles::validate(report, facts).is_err(),
        "bare role bits accepted without an actual source seal"
    );
    {
        let ledger = facts
            .source_bindings
            .as_deref_mut()
            .and_then(|catalog| catalog.traits.as_deref_mut())
            .ok_or("same role ledger")?;
        assert_eq!(
            ledger.roles, mask,
            "failed extension published regular roles"
        );
        ledger.pointee = Some(std::rc::Rc::clone(&owner));
    }
    roles::validate(report, facts)?;
    let ledger = facts
        .source_bindings
        .as_deref()
        .and_then(|catalog| catalog.traits.as_deref())
        .ok_or("extended same owner")?;
    assert_eq!(ledger.roles, roles);
    assert!(std::rc::Rc::ptr_eq(
        ledger.pointee.as_ref().ok_or("retained owner")?,
        &owner
    ));
    Ok(())
}

fn search_argument(report: &Report, facts: &mut Declarations, root: TypeId) -> TestResult {
    let sink = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, row)| {
            row.kind == crate::resolve::model::DefKind::Trait
                && report.package.names.get(row.name) == "Sink"
        })
        .ok_or("genuine Sink")?
        .0;
    let sink = ItemId(u32::try_from(sink)?);
    let args = facts
        .types
        .intern_arguments(&[crate::types::arguments::Argument::Type(root)])?;
    let parent = facts
        .types
        .intern_trait_application(crate::types::traits::Application {
            item: sink,
            arguments: args,
        })?;
    let mut context = Context::default();
    let mut substitution = Instantiation::new(&mut context, &facts.types, &[])?;
    let goal = context.instantiate_predicate(
        &mut facts.types,
        Predicate {
            ty: TypeId::primitive(Primitive::U8),
            bound: Bound::Trait(parent),
        },
        &mut substitution,
    )?;
    let assumptions = Assumptions::default();
    let selection::Selection::Proven(evidence) = selection::select_available(
        facts,
        &mut context,
        &goal,
        selection::Caller {
            assumptions: &assumptions,
            environment: None,
            first_region: 1,
            retain_sized: false,
        },
        Availability::Validated,
    )?
    else {
        return Err("actual generic argument normalization did not prove Sink<()>".into());
    };
    assert!(!evidence.provisional_size && evidence.sized.is_empty());
    Ok(())
}
#[test]
fn concrete_tails_and_erased_nominal_inputs_keep_real_source_bodies() -> TestResult {
    for name in [
        "with-copy/concrete-results",
        "with-copy/changed-tail",
        "projected-tail-and-prefix",
        "invalid-input-formation",
        "universal-parameter",
    ] {
        let report = provider(name)?;
        pipeline::analyze_source(&report).map_err(|error| format!("{name}: {error}"))?;
    }
    Ok(())
}
#[test]
fn sized_fallback_never_overrides_assumed_parent_or_actual_equality() -> TestResult {
    for name in [
        "with-copy/sized-parameter",
        "explicit-equality",
        "result-obligation-positive",
    ] {
        pipeline::analyze_source(&provider(name)?).map_err(|error| format!("{name}: {error}"))?;
    }
    for (name, function, assumed) in [
        ("assumed-parent", "from_assumption", true),
        ("unknown-result", "unknown", false),
    ] {
        let report = provider(name)?;
        opaque_parent_is_not_unit(&report, function, assumed)?;
        let error = pipeline::analyze_source(&report)
            .err()
            .ok_or("unproved unit result accepted")?;
        assert_eq!(
            error.message, "associated type projection requires normalization",
            "{name}: {error}"
        );
    }
    let error = pipeline::analyze_source(&provider("result-obligation-negative")?)
        .err()
        .ok_or("unproved actual result requirement accepted")?;
    assert!(
        error
            .message
            .contains("user trait obligation requires a matching assumption or implementation"),
        "{error}"
    );
    Ok(())
}
/// Inspect the actual scoped declaration environment before judging the body's
/// narrow diagnostic boundary. Neither absence nor a type hint supplies unit.
fn opaque_parent_is_not_unit(report: &Report, name: &str, assumed: bool) -> TestResult {
    let mut facts = declarations::analyze(report)?;
    let function = *facts
        .functions
        .iter()
        .find(|function| {
            report
                .package
                .names
                .get(report.resolution.definitions[function.definition.0 as usize].name)
                == name
        })
        .ok_or("actual negative function")?;
    let Kind::Function { parameters, .. } = facts
        .types
        .kind(function.signature)
        .ok_or("actual signature")?
    else {
        return Err("function signature is not callable".into());
    };
    let root = *facts
        .types
        .list(parameters)
        .and_then(|values| values.first())
        .ok_or("actual Metadata parameter")?;
    let Kind::Projection(application) = facts.types.kind(root).ok_or("actual parameter kind")?
    else {
        return Err("opaque Metadata parameter was silently replaced".into());
    };
    let environment = facts
        .generics
        .environments
        .get(&function.definition)
        .cloned()
        .ok_or("real caller environment")?;
    let mut fold = Normalizer::default();
    let Step::Resolve(request) = fold.start(&mut facts.types, root)? else {
        return Err("actual opaque Metadata request".into());
    };
    let normal_form::Progress::Ready(projected) = normal_form::begin(
        &mut facts,
        Some(environment),
        request,
        1,
        false,
        Availability::Validated,
    )?
    else {
        return Err("complete scoped Metadata unexpectedly pending".into());
    };
    assert_eq!(projected.resolution, Some(Resolution::Opaque));
    if assumed {
        let Some(selection::Source::Assumption(premise)) = projected.source else {
            return Err("scoped Pointee assumption lost".into());
        };
        assert_eq!(
            premise,
            Predicate {
                ty: application.subject,
                bound: Bound::Trait(application.trait_application)
            }
        );
    } else {
        assert!(matches!(
            projected.source,
            Some(selection::Source::Pointee(_))
        ));
    }
    Ok(())
}

#[test]
fn dynamic_regions_and_failed_retention_replays_keep_the_actual_source_owner() -> TestResult {
    let positive = pipeline::analyze_source(&provider("with-copy/dynamic-region")?)?;
    let contracts = crate::body::regions::collect(&positive.facts)?;
    crate::borrow::regions::check(
        &positive.facts.types,
        &positive.facts.lifetimes.variances,
        &contracts,
    )?;
    let negative = pipeline::analyze_source(&provider("with-copy/dynamic-escape")?)?;
    let contracts = crate::body::regions::collect(&negative.facts)?;
    let error = crate::borrow::regions::check(
        &negative.facts.types,
        &negative.facts.lifetimes.variances,
        &contracts,
    )
    .err()
    .ok_or("dynamic object region escaped actual body contract proof")?;
    assert!(error.message.contains("lifetime"), "{error}");
    let report = provider("with-copy/concrete-results")?;
    let mut facts = declarations::analyze(&report)?;
    let application = application(&mut facts, TypeId::primitive(Primitive::U8))?;
    let super::Readiness::Ready(selected) = super::select(
        &mut facts,
        Predicate {
            ty: application.subject,
            bound: Bound::Trait(application.trait_application),
        },
    )?
    else {
        return Err("complete source Pointee not ready".into());
    };
    let payments = header_attributes::source_payments(facts.must_use_contracts.as_deref())?;
    let payment_layout = size_of_val(&payments);
    println!(
        "Pointee HeaderLedger={} Payments={} Cell={} usize={}",
        size_of::<header_attributes::Ledger>(),
        payment_layout,
        size_of::<std::cell::Cell<usize>>(),
        size_of::<usize>()
    );
    let mut plan = MetadataPlan::new(&mut facts, &selected, application)?;
    assert!(matches!(plan.advance(&mut facts)?, MetadataStep::Ready(_)));
    assert!(
        header_attributes::charge_source_retention(&facts, 1, crate::support::crates::MAX_BYTES)
            .is_err()
    );
    let failed =
        header_attributes::source_contract_storage(&facts).ok_or("existing source owner")?;
    assert!(plan.advance(&mut facts).is_err());
    assert!(plan.advance(&mut facts).is_err());
    let retried =
        header_attributes::source_contract_storage(&facts).ok_or("retained source owner")?;
    assert!(retried.0 > failed.0);
    assert_eq!(retried.1, failed.1);
    assert_eq!(
        (failed.2, failed.3, failed.4),
        (retried.2, retried.3, retried.4)
    );
    let ordinary = declarations::analyze(&crate::test_source::resolve("fn ordinary(){}", true)?)?;
    assert!(header_attributes::source_contract_storage(&ordinary).is_none());
    assert!(header_attributes::charge_source_retention(&ordinary, 1, 1).is_err());
    assert!(header_attributes::source_contract_storage(&ordinary).is_none());
    Ok(())
}
