//! Real source caller data is separate from callable proof and source signatures.
use crate::{
    body,
    const_eval::code::Code,
    sema, test_source,
    types::{TypeId, instance::Instance},
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const PROVIDER: &str = "pa31/tests/spec/source-location/caller-location.rs";

#[test]
fn direct_source_caller_locations_use_const_frames_and_runtime_hidden_abi() -> TestResult {
    let source = test_source::fixture(PROVIDER)?;
    let report = test_source::source_provider(&source, "caller_native")?;
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(&report)?;
    let owner = facts
        .source_bindings
        .as_deref()
        .and_then(|catalog| catalog.call_contexts.as_deref())
        .ok_or("actual caller context")?;
    println!(
        "caller context owner={} Header={} CallableInputs={} CallerSite={} Operand={} Code={} Body={} Rvalue={} Value={}",
        size_of_val(owner),
        size_of::<super::super::Header>(),
        size_of::<body::ir::ResolvedCallables>(),
        size_of::<body::ir::CallerSite>(),
        size_of::<crate::const_eval::code::PreparedOperand>(),
        size_of::<Code>(),
        size_of::<body::ir::Body>(),
        size_of::<body::ir::Rvalue>(),
        size_of::<body::ir::Value>()
    );
    let program = crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?;
    for name in [
        "caller_line",
        "caller_column",
        "constant_line",
        "constant_column",
        "filename_pointer",
        "filename_length",
    ] {
        assert!(
            program
                .symbols
                .iter()
                .any(|symbol| symbol.name == name && symbol.body.is_some()),
            "{name}"
        );
    }
    let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/all-attribute-selected-output-v1/native/source-caller");
    std::fs::create_dir_all(&out)?;
    std::fs::write(out.join("provider.rs"), source)?;
    crate::lowir::writer::write(
        &program,
        &mut std::fs::File::create(out.join("provider.lowir"))?,
    )?;
    Ok(())
}

#[test]
fn caller_metadata_cannot_replace_targets_or_forge_snapshot_identity() -> TestResult {
    let source =
        test_source::fixture("pa31/tests/spec/source-functions-transport/minimal-provider.rs")?
            + "fn f()->u32{7}fn g()->u32{f()}";
    let report = test_source::source_provider(&source, "caller_storage")?;
    let facts = sema::declarations::analyze(&report)?;
    let mut facts = sema::bodies::analyze(&report, facts)?;
    let mut program = body::build_concrete(&report, &facts)?;
    let actual = program
        .bodies
        .iter_mut()
        .find(|body| {
            body.values
                .iter()
                .any(|value| matches!(value.kind, body::ir::Rvalue::Call { .. }))
        })
        .ok_or("actual ordinary call body")?;
    assert!(!actual.has_resolved_callables());
    let (index, callee) = actual
        .values
        .iter()
        .enumerate()
        .find_map(|(index, value)| {
            if let body::ir::Rvalue::Call { callee, .. } = value.kind {
                Some((index, callee))
            } else {
                None
            }
        })
        .ok_or("call")?;
    let callable = actual.values[callee.0 as usize].ty;
    let location = sema::values::Evaluator::new(&report, &mut facts)
        .borrow(
            sema::values::Constant::scalar(sema::model::Scalar::Integer(1)),
            TypeId::primitive(crate::types::Primitive::U32),
        )?
        .location()
        .ok_or("real test allocation")?;
    let site = body::ir::CallerSite {
        owner: actual.source.source().ok_or("source owner")?,
        value: body::ir::ValueId(u32::try_from(index)?),
        callable,
        location,
    };
    assert!(
        sema::values::Reader::new(&facts)
            .caller_location_value(location)
            .is_err()
    );
    for invalid in 0..3 {
        let mut changed = site;
        match invalid {
            0 => changed.owner.node = report.package.unit(changed.owner.file).configured.ast.root,
            1 => changed.value = body::ir::ValueId(u32::MAX),
            _ => changed.callable = TypeId::UNIT,
        }
        let mut altered = actual.clone();
        altered.append_caller_site(changed, &facts)?;
        assert!(
            altered.seal_caller_sites(&facts).is_err(),
            "changed site {invalid}"
        );
        assert!(
            Code::snapshot(&altered).is_err(),
            "changed snapshot {invalid}"
        );
    }
    actual.append_caller_site(site, &facts)?;
    actual.seal_caller_sites(&facts)?;
    assert!(!actual.has_resolved_callables());
    let instance = Instance::new(&facts.types, callable)?;
    actual.install_callable_targets(crate::support::hash::HashMap::from_iter([(
        instance, instance,
    )]))?;
    assert!(actual.has_resolved_callables());
    assert_eq!(actual.caller_site(site.value)?, Some(site));
    let snapshot = Code::snapshot(actual)?;
    assert_eq!(snapshot.caller_site(site.value)?, Some(site));
    assert_eq!(snapshot.operands.len(), 1);
    assert_eq!(snapshot.caller_site_bytes(), 40);
    actual.append_caller_site(site, &facts)?;
    assert!(actual.seal_caller_sites(&facts).is_err());
    assert!(Code::snapshot(actual).is_err());
    assert_eq!(size_of::<body::ir::Body>(), 456);
    assert_eq!(size_of::<body::ir::Rvalue>(), 20);
    assert_eq!(size_of::<body::ir::Value>(), 24);
    assert_eq!(size_of::<body::ir::ResolvedCallables>(), 56);
    assert_eq!(
        size_of::<super::super::Header>(),
        72 + size_of::<Option<Box<super::super::promotability::Marker>>>()
    );
    assert_eq!(size_of::<crate::const_eval::code::PreparedOperand>(), 32);
    assert_eq!(size_of::<Code>(), 304);
    Ok(())
}

#[test]
fn intrinsic_const_policy_and_track_caller_opponents_remain_actual_source_checks() -> TestResult {
    let source = test_source::fixture(PROVIDER)?;
    for (label, changed, expected) in [
        (
            "feature",
            source.replace("feature(core_intrinsics,", "feature("),
            "source intrinsic const API requires",
        ),
        (
            "marker",
            source.replace("#[rustc_intrinsic_const_stable_indirect]", ""),
            "const-stable body cannot expose",
        ),
        (
            "shape",
            source.replace(
                "pub const fn caller_location()",
                "pub const fn caller_location(value:u8)",
            ),
            "caller_location",
        ),
    ] {
        let outcome =
            test_source::source_provider(&changed, "caller_negative").and_then(|report| {
                crate::pipeline::analyze_source(&report)
                    .map(|_| ())
                    .map_err(|error| error.to_string())
            });
        let error = outcome
            .err()
            .ok_or_else(|| format!("{label} was accepted"))?;
        assert!(error.contains(expected), "{label}: {error}");
    }
    Ok(())
}

fn selected_export(
    report: &crate::resolve::expansion::Report,
    name: &str,
) -> TestResult<crate::resolve::model::DefId> {
    report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, definition)| {
            definition.kind == crate::resolve::model::DefKind::Function
                && report.package.names.get(definition.name) == name
        })
        .map(|(index, _)| u32::try_from(index).map(crate::resolve::model::DefId))
        .transpose()?
        .ok_or_else(|| format!("actual {name}").into())
}
fn selected_call(
    report: &crate::resolve::expansion::Report,
    facts: &sema::model::Declarations,
    definition: crate::resolve::model::DefId,
) -> TestResult<(
    crate::resolve::unit::NodeKey,
    crate::resolve::unit::NodeKey,
    TypeId,
)> {
    use crate::syntax::ast::Kind;
    let owner = facts
        .functions
        .iter()
        .find(|function| function.definition == definition)
        .and_then(|function| function.body)
        .ok_or("actual selected caller body")?;
    let ast = &report.package.unit(owner.file).configured.ast;
    let Kind::Block {
        tail: Some(call), ..
    } = ast.nodes[owner.node.get() as usize].kind
    else {
        return Err("actual call tail".into());
    };
    let Kind::Call { callee, .. } = ast.nodes[call.get() as usize].kind else {
        return Err("actual UFCS Call".into());
    };
    let callable = facts
        .node(crate::resolve::unit::NodeKey {
            node: callee,
            ..owner
        })
        .and_then(|node| node.ty)
        .ok_or("actual solved callee")?;
    Ok((
        owner,
        crate::resolve::unit::NodeKey {
            node: call,
            ..owner
        },
        callable,
    ))
}
fn checked_selected_site(
    report: &crate::resolve::expansion::Report,
    facts: &mut sema::model::Declarations,
    definition: crate::resolve::model::DefId,
    tracked: bool,
) -> TestResult<(crate::resolve::unit::NodeKey, TypeId)> {
    let (owner, call, callable) = selected_call(report, facts, definition)?;
    let Some(sema::model::Operation::Call {
        source_use: Some(source_use),
        ..
    }) = facts.node(call).and_then(|node| node.operation)
    else {
        return Err("actual source call use".into());
    };
    assert_eq!(
        source_use
            .caller_site(facts, owner, call.node, callable)?
            .is_some(),
        tracked
    );
    let ast = &report.package.unit(call.file).configured.ast;
    let crate::syntax::ast::Kind::Call { callee, .. } = ast.nodes[call.node.get() as usize].kind
    else {
        return Err("actual Call".into());
    };
    let key = crate::resolve::unit::NodeKey {
        node: callee,
        ..call
    };
    let selected = sema::dispatch::selected_definition(facts, key, callable)?
        .ok_or("actual selected target")?;
    assert_eq!(facts.checked_caller_tracking(report, selected)?, tracked);
    Ok((key, callable))
}
fn wrong_selection(
    facts: &mut sema::model::Declarations,
    source: crate::resolve::unit::NodeKey,
    other: crate::resolve::unit::NodeKey,
    callable: TypeId,
) -> TestResult {
    let copied = facts
        .generics
        .trait_selections
        .get(&source)
        .ok_or("actual source selections")?
        .iter()
        .map(|selection| sema::generics::ImplementationSelection {
            owner: selection.owner,
            arguments: selection.arguments,
        })
        .collect();
    let old = facts.generics.trait_selections.insert(other, copied);
    assert!(sema::dispatch::selected_definition(facts, other, callable)?.is_none());
    facts
        .generics
        .trait_selections
        .insert(other, old.ok_or("actual prior selections")?);
    Ok(())
}
#[test]
fn selected_impl_tracking_uses_exact_self_rhs_and_prepares_real_runtime_and_const_sites()
-> TestResult {
    let source = test_source::fixture("pa31/tests/spec/source-location/selected-tracking.rs")?;
    let report = test_source::source_provider(&source, "selected_tracking")?;
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(&report)?;
    assert!(facts.array_lengths.values().any(|&bound| bound == 375));
    let roots = [
        "selected_runtime_line",
        "selected_constant_line",
        "untracked_rhs",
        "untracked_self",
    ]
    .iter()
    .map(|name| selected_export(&report, name))
    .collect::<TestResult<Vec<_>>>()?;
    let (key, _) = checked_selected_site(&report, &mut facts, roots[0], true)?;
    for &definition in &roots[2..] {
        let (other, callable) = checked_selected_site(&report, &mut facts, definition, false)?;
        wrong_selection(&mut facts, key, other, callable)?;
    }
    let program = crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &roots,
        },
        prepared,
    )?;
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/source-div-v1/candidate/selected-tracking");
    std::fs::create_dir_all(&folder)?;
    std::fs::write(folder.join("source.rs"), source)?;
    crate::lowir::writer::write(
        &program,
        &mut std::fs::File::create(folder.join("provider.lowir"))?,
    )?;
    Ok(())
}
#[test]
fn untracked_selected_impls_keep_actual_const_calls_without_any_location_provider() -> TestResult {
    let source = test_source::fixture("pa31/tests/spec/source-location/untracked-selection.rs")?;
    let report = test_source::source_provider(&source, "untracked_selection")?;
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(&report)?;
    assert!(
        facts
            .source_bindings
            .as_deref()
            .ok_or("actual source catalog")?
            .language(crate::support::bindings::LangItem::PanicLocation)
            .is_none()
    );
    assert!(facts.array_lengths.values().any(|&bound| bound == 73));
    let roots = [
        selected_export(&report, "untracked_no_location")?,
        selected_export(&report, "untracked_no_location_const")?,
    ];
    crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &roots,
        },
        prepared,
    )?;
    Ok(())
}

#[test]
fn unused_tracked_calls_type_without_location_but_reached_calls_require_actual_data() -> TestResult
{
    let source = test_source::fixture("pa31/tests/spec/source-location/deferred-tracking.rs")?;
    let report = test_source::source_provider(&source, "deferred_tracking")?;
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(&report)?;
    assert!(
        facts
            .source_bindings
            .as_deref()
            .ok_or("actual source catalog")?
            .language(crate::support::bindings::LangItem::PanicLocation)
            .is_none()
    );
    let root = selected_export(&report, "tracked_without_location")?;
    let error = crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[root],
        },
        prepared,
    )
    .err()
    .ok_or("reached tracked call accepted without actual caller data")?;
    let message = error.to_string();
    assert!(
        message.contains("tracked call lacks its actual sealed caller site"),
        "{message}"
    );
    Ok(())
}
