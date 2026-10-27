//! Source discriminator authority, both normalizers and actual consumers share one owner.
use super::*;
use crate::{
    pipeline,
    resolve::{expansion::Report, model::DefKind},
    sema::{
        declarations, header_attributes,
        model::{Declarations, Scalar},
        nominals::traits::normal_form,
        traits::{assumptions::Assumptions, implementations::Availability, selection},
        values::{Reader, View},
    },
    types::{
        GenericArgs, ItemId, Kind, Primitive, TypeId,
        infer::{Context, Instantiation},
        parameters::predicates::{Bound, Predicate},
        projection,
        substitute::{Normalizer, Resolution, Step},
    },
};
use std::mem::size_of;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn source(name: &str) -> TestResult<String> {
    Ok(crate::test_source::fixture(&format!(
        "pa31/tests/spec/source-discriminants/{name}.rs"
    ))?)
}
fn provider(name: &str) -> TestResult<Report> {
    Ok(crate::test_source::source_provider(
        &source(name)?,
        "source_discriminants",
    )?)
}
fn app(facts: &mut Declarations, ty: TypeId) -> TestResult<projection::Application> {
    let catalog = facts.source_bindings.as_deref().ok_or("source catalog")?;
    let parent = catalog
        .validated_language(LangItem::DiscriminantKind)?
        .ok_or("actual trait")?;
    let member = catalog
        .validated_language(LangItem::DiscriminantType)?
        .ok_or("actual member")?;
    Ok(projection::Application {
        item: ItemId(member.definition.0),
        subject: ty,
        trait_application: facts.types.intern_trait_application(
            crate::types::traits::Application {
                item: ItemId(parent.definition.0),
                arguments: GenericArgs::EMPTY,
            },
        )?,
        arguments: GenericArgs::EMPTY,
    })
}
fn token(facts: &mut Declarations) -> TestResult<Selection> {
    let application = app(facts, TypeId::primitive(Primitive::I8))?;
    let Readiness::Ready(selected) = select(
        facts,
        Predicate {
            ty: application.subject,
            bound: Bound::Trait(application.trait_application),
        },
    )?
    else {
        return Err("completed protocol".into());
    };
    Ok(selected)
}
#[test]
fn actual_headers_and_source_incarnations_authenticate_one_cold_owner() -> TestResult {
    let mut report = provider("setup-followup/minimal")?;
    let mut facts = declarations::analyze(&report)?;
    let selected = token(&mut facts)?;
    selected.current(&facts)?;
    authenticate(&report, &facts)?;
    let other = provider("setup-followup/minimal")?;
    let other_facts = declarations::analyze(&other)?;
    assert!(selected.current(&other_facts).is_err());
    assert!(authenticate(&other, &facts).is_err());
    let copy = report.package.units[0].source.physical_copy();
    assert_eq!(copy.original(), report.package.units[0].source.original());
    report.package.units[0].source = copy;
    assert!(authenticate(&report, &facts).is_err());
    let full = source("setup-followup/full-bound")?;
    let offset = full
        .find("#[lang=\"dispatch_from_dyn\"]")
        .ok_or("actual dynamic prerequisite")?;
    let error = crate::test_source::source_provider(&full, "full_discriminants")
        .err()
        .ok_or("unsupported dynamic receiver role was fabricated")?;
    assert_eq!(
        error,
        format!("file 0: {offset}: unsupported source language item")
    );
    Ok(())
}
#[test]
fn canonical_query_search_and_body_normalize_actual_declared_repr() -> TestResult {
    let report = provider("consumers/opened-scope")?;
    // This group owns actual checked-body/normalizer facts; the next group owns reached CTFE.
    let mut analysis = pipeline::analyze(&report, declarations::analyze(&report)?)?;
    observe_call_signatures(&report, &analysis.facts)?;
    let enum_id = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find_map(|(i, d)| {
            (d.kind == DefKind::Enum && report.package.names.get(d.name) == "Signed")
                .then_some(ItemId(u32::try_from(i).ok()?))
        })
        .ok_or("actual enum")?;
    let ty = analysis
        .facts
        .types
        .intern(Kind::Adt(enum_id, GenericArgs::EMPTY))?;
    let application = app(&mut analysis.facts, ty)?;
    let root = analysis.facts.types.intern(Kind::Projection(application))?;
    let mut fold = Normalizer::default();
    let Step::Resolve(request) = fold.start(&mut analysis.facts.types, root)? else {
        return Err("projection request".into());
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
        return Err("completed source unexpectedly pending".into());
    };
    assert_eq!(
        projected.resolution,
        Some(Resolution::Type(TypeId::primitive(Primitive::I8)))
    );
    assert!(matches!(
        projected.source,
        Some(selection::Source::Discriminant(_))
    ));
    search_argument(&report, &mut analysis.facts, root)?;
    let owner = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find_map(|(i, d)| {
            (d.kind == DefKind::Function && report.package.names.get(d.name) == "unresolved")
                .then_some(crate::resolve::model::DefId(u32::try_from(i).ok()?))
        })
        .ok_or("actual generic owner")?;
    let input = analysis
        .facts
        .generics
        .parameters
        .get(&owner)
        .and_then(|p| p.first())
        .ok_or("real formal")?
        .argument;
    let crate::types::arguments::Argument::Type(input) = input else {
        return Err("real type formal".into());
    };
    let application = app(&mut analysis.facts, input)?;
    let unresolved = analysis.facts.types.intern(Kind::Projection(application))?;
    let mut unknown = Normalizer::default();
    let Step::Resolve(request) = unknown.start(&mut analysis.facts.types, unresolved)? else {
        return Err("unknown projection".into());
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
        return Err("unknown result was confused with unfinished source".into());
    };
    assert_eq!(projected.resolution, Some(Resolution::Opaque));

    Ok(())
}
fn observe_call_signatures(report: &Report, facts: &Declarations) -> TestResult {
    let ast = &report.package.units[0].configured.ast;
    let mut first_owner = None;
    for (index, node) in facts.nodes[0].iter().enumerate() {
        let Some(crate::sema::model::Operation::Call { signature, .. }) = node.operation else {
            continue;
        };
        let id = crate::syntax::ast::NodeId::checked(u32::try_from(index)?).ok_or("actual node")?;
        let crate::syntax::ast::Kind::Call { callee, .. } = ast.node(id).kind else {
            continue;
        };
        let key = crate::resolve::unit::NodeKey {
            file: crate::resolve::unit::FileId(0),
            node: id,
        };
        let Some(callable) = facts
            .node(crate::resolve::unit::NodeKey {
                node: callee,
                ..key
            })
            .and_then(|n| n.ty)
        else {
            continue;
        };
        let Some(Kind::FunctionItem {
            item,
            signature: public,
            arguments,
        }) = facts.types.kind(callable)
        else {
            continue;
        };
        if facts
            .source_bindings
            .as_deref()
            .ok_or("source catalog")?
            .intrinsic_kind(crate::resolve::model::DefId(item.0))?
            != Some(crate::support::bindings::SourceIntrinsic::DiscriminantValue)
        {
            continue;
        }
        let Some(crate::sema::model::Operation::Call {
            source_use: Some(source_use),
            ..
        }) = node.operation
        else {
            return Err("checked call source use".into());
        };
        let owner = source_use.owner(facts)?;
        let catalog = facts.source_bindings.as_deref().ok_or("source catalog")?;
        catalog.validate_discriminant_opening(report, facts, owner, key, (callable, signature))?;
        assert!(
            catalog
                .validate_discriminant_opening(report, facts, owner, key, (callable, public))
                .is_err()
        );
        if let Some(other) = first_owner {
            if other != owner {
                assert!(
                    catalog
                        .validate_discriminant_opening(
                            report,
                            facts,
                            other,
                            key,
                            (callable, signature)
                        )
                        .is_err()
                );
            }
        } else {
            first_owner = Some(owner);
        }
        eprintln!(
            "actual-discriminant-call node={id:?} offset={} item={item:?} args={arguments:?} public={public:?}:{:?} opened={signature:?}:{:?}",
            ast.node(id).span.start,
            facts.types.kind(public),
            facts.types.kind(signature)
        );
        for (label, ty) in [("public", public), ("opened", signature)] {
            if let Some(Kind::Function { parameters, .. }) = facts.types.kind(ty) {
                for input in facts
                    .types
                    .list(parameters)
                    .ok_or("actual signature inputs")?
                {
                    eprintln!(
                        "actual-discriminant-input {label} {input:?} {:?}",
                        facts.types.kind(*input)
                    );
                }
            }
        }
    }
    Ok(())
}

fn search_argument(report: &Report, facts: &mut Declarations, root: TypeId) -> TestResult {
    let sink = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find_map(|(i, d)| {
            (d.kind == DefKind::Trait && report.package.names.get(d.name) == "Sink")
                .then_some(ItemId(u32::try_from(i).ok()?))
        })
        .ok_or("actual Sink trait")?;
    let args = facts
        .types
        .intern_arguments(&[crate::types::arguments::Argument::Type(root)])?;
    let application = facts
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
            bound: Bound::Trait(application),
        },
        &mut substitution,
    )?;
    let assumptions = Assumptions::default();
    assert!(matches!(
        selection::select_available(
            facts,
            &mut context,
            &goal,
            selection::Caller {
                assumptions: &assumptions,
                environment: None,
                first_region: 1,
                retain_sized: false
            },
            Availability::Validated
        )?,
        selection::Selection::Proven(_)
    ));
    Ok(())
}
#[test]
fn real_const_and_lowered_calls_preserve_logical_tags_and_argument_effects() -> TestResult {
    let report = provider("consumers/actual-calls")?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    let reader = Reader::new(&facts);
    for (name, bits) in [("HIGH", 1), ("TAG", 300), ("ZERO", 0), ("NONE", 0)] {
        let id = report
            .resolution
            .definitions
            .iter()
            .enumerate()
            .find_map(|(i, d)| {
                (d.kind == DefKind::Const && report.package.names.get(d.name) == name)
                    .then_some(crate::resolve::model::DefId(u32::try_from(i).ok()?))
            })
            .ok_or("actual constant")?;
        let value = reader
            .definition(id)
            .ok_or("mandatory constant was not evaluated")?;
        assert!(
            matches!(reader.view(value)?,View::Scalar(Scalar::Integer(actual)) if actual==bits),
            "{name}"
        );
    }
    let program = crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?;
    assert!(program.symbols.iter().filter(|s| s.body.is_some()).count() >= 5);
    selected_local_calls()?;
    let fault = provider("consumers/reference-fault")?;
    let error = pipeline::analyze_source(&fault)
        .err()
        .ok_or("erased nonenum result skipped its invalid reference argument")?;
    assert!(
        error
            .to_string()
            .contains("constant pointer has no allocation provenance"),
        "{error}"
    );

    Ok(())
}

fn selected_local_calls() -> TestResult {
    let mut input = source("consumers/actual-calls")?;
    input.push_str(
        "\nfn main(){let _=signed();let _=payload();let _=none();let _=some();let _=effects();}",
    );
    let report = crate::test_source::source_provider(&input, "selected_discriminants")?;
    let mut roots = Vec::new();
    for name in ["main", "signed", "payload", "none", "some", "effects"] {
        let (index, _) = report
            .resolution
            .definitions
            .iter()
            .enumerate()
            .find(|(_, row)| {
                row.kind == DefKind::Function && report.package.names.get(row.name) == name
            })
            .ok_or("selected source discriminant function")?;
        roots.push(crate::resolve::model::DefId(u32::try_from(index)?));
    }
    let entry = roots[0];
    let selected = crate::lower::local_runtime::analyze(&report, &roots)?;
    let program = selected.build(entry)?;
    assert!(program.symbols.iter().filter(|s| s.body.is_some()).count() >= roots.len());
    assert!(
        crate::lower::local_runtime::analyze(&report, &[entry])?
            .build(entry)
            .is_err(),
        "a genuine unselected source callee gained executable proof"
    );
    Ok(())
}
#[test]
fn source_marker_impls_override_structural_defaults_without_std_leaks() -> TestResult {
    for (name, pass) in [
        ("raw-without-negative", true),
        ("raw-source-negative", false),
        ("reference-without-impl", false),
        ("reference-source-impl", true),
        ("unpin-without-impl", false),
        ("unpin-source-impl", true),
    ] {
        let report = provider(&format!("markers/{name}"))?;
        let result = pipeline::analyze_source(&report);
        assert_eq!(result.is_ok(), pass, "{name}: {:?}", result.err());
    }
    assert!(crate::support::bindings::LangItem::from_name("send").is_none());
    assert!(provider("markers/fake-lang-send").is_err());
    let wrong = source("markers/raw-without-negative")?
        .replace(
            "#[lang=\"sync\"]#[rustc_diagnostic_item=\"Sync\"]",
            "#[lang=\"sync\"]",
        )
        .replace(
            "#[rustc_diagnostic_item=\"Send\"]",
            "#[rustc_diagnostic_item=\"Send\"]#[rustc_diagnostic_item=\"Sync\"]",
        );
    let error = crate::test_source::source_provider(&wrong, "wrong_sync_owner")
        .err()
        .ok_or("foreign Sync diagnostic owner accepted")?;
    assert!(error.contains("different actual language owner"), "{error}");

    Ok(())
}
#[test]
fn actual_signature_features_and_transport_never_follow_spelling() -> TestResult {
    let original = source("setup-followup/minimal")?;
    for (from, to) in [
        (
            "fn discriminant_value<T>(v:&T)",
            "fn discriminant_value<T>(v:&mut T)",
        ),
        ("#[lang=\"discriminant_kind\"]", ""),
        ("#[lang=\"discriminant_type\"]", ""),
        ("#[rustc_intrinsic]", "#[rustc_intrinsic(invalid)]"),
    ] {
        let changed = original.replace(from, to);
        assert_ne!(changed, original);
        let result = crate::test_source::source_provider(&changed, "bad_discriminant");
        assert!(
            result
                .and_then(|r| declarations::analyze(&r)
                    .map(|_| ())
                    .map_err(|e| e.to_string()))
                .is_err(),
            "{from}"
        );
    }
    let report = provider("setup-followup/minimal")?;
    let facts = declarations::analyze(&report)?;
    let error = require_transport(&report, &facts)
        .err()
        .ok_or("transport unexpectedly opened")?;
    assert_eq!(
        error.message,
        "source discriminant protocol transport is not implemented"
    );
    Ok(())
}
#[test]
fn ordinary_layouts_and_failed_work_keep_existing_owners_and_widths() -> TestResult {
    let ordinary = crate::test_source::resolve("fn ordinary(){}", true)?;
    let ordinary_facts = declarations::analyze(&ordinary)?;
    assert!(ordinary_facts.source_bindings.is_none());
    assert!(header_attributes::source_contract_storage(&ordinary_facts).is_none());
    let report = provider("setup-followup/minimal")?;
    let mut facts = declarations::analyze(&report)?;
    let selected = token(&mut facts)?;
    let before = header_attributes::source_contract_storage(&facts).ok_or("same paid ledger")?;
    for _ in 0..32 {
        selected.current(&facts)?;
    }
    let after = header_attributes::source_contract_storage(&facts).ok_or("same paid ledger")?;
    assert!(after.0 > before.0);
    assert_eq!(
        (before.1, before.2, before.3, before.4),
        (after.1, after.2, after.3, after.4)
    );
    assert_eq!(size_of::<selection::Source>(), 16);
    assert_eq!(size_of::<crate::types::Kind>(), 24);
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    assert_eq!(size_of::<crate::sema::model::NodeFacts>(), 80);
    assert_eq!(size_of::<crate::sema::model::Operation>(), 16);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    eprintln!(
        "Discriminant Contracts={} Validated={} Catalog={} Reservations={} Source={} Evidence={} ProjectionQuery={} before={before:?} after={after:?}",
        size_of::<Contracts>(),
        size_of::<super::super::roles::Validated>(),
        size_of::<super::super::Catalog>(),
        size_of::<crate::resolve::bindings::Reservations>(),
        size_of::<selection::Source>(),
        size_of::<selection::Evidence>(),
        size_of::<normal_form::Query>()
    );
    assert!(
        header_attributes::charge_source_contract_query(
            &facts,
            crate::support::crates::MAX_COMPONENTS
        )
        .is_err()
    );
    let failed = header_attributes::source_contract_storage(&facts).ok_or("failed work")?;
    assert!(selected.current(&facts).is_err());
    assert!(
        header_attributes::source_contract_storage(&facts)
            .ok_or("retry work")?
            .0
            > failed.0
    );
    Ok(())
}
