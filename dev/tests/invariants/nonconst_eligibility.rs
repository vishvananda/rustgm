//! Source trait constness is not inherited by a marked runtime member.
use super::*;
use crate::{pipeline, support::diagnostics::LintCap, test_source};
use std::fmt::Write;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const BASE: &str = "pa31/tests/spec/source-nonconst-members/";
fn input(name: &str) -> TestResult<String> {
    Ok(test_source::fixture(&format!("{BASE}{name}.rs"))?)
}
fn report(source: &str, tag: &str) -> TestResult<Report> {
    let mut config = crate::expand::config::Config::default();
    config.set_lint_cap(LintCap::Allow);
    Ok(test_source::source_provider_config(source, tag, &config)?)
}
fn method(report: &Report, facts: &Declarations, name: &str) -> TestResult<DefId> {
    facts
        .functions
        .iter()
        .find(|function| {
            is_trait_member(facts, function.definition)
                && report
                    .package
                    .names
                    .get(report.resolution.definitions[function.definition.0 as usize].name)
                    == name
        })
        .map(|function| function.definition)
        .ok_or_else(|| "actual trait method".into())
}
fn local(facts: &mut Declarations) -> TestResult<&mut super::super::Headers> {
    Ok(&mut facts
        .must_use_contracts
        .as_deref_mut()
        .and_then(|row| row.headers.as_deref_mut())
        .ok_or("local header owner")?
        .local)
}
#[test]
fn actual_trait_member_choice_keeps_impl_and_default_contexts_distinct() -> TestResult {
    let source = input("source-runtime")?;
    let report = report(&source, "nonconst_contexts")?;
    let facts = pipeline::analyze_source(&report)?.facts;
    let runtime = method(&report, &facts, "runtime")?;
    let constant = method(&report, &facts, "constant")?;
    let required = method(&report, &facts, "required")?;
    for id in [runtime, required] {
        assert!(matches!(
            facts.source_non_const_trait_method_contract(&report, id)?,
            ContractKnowledge::KnownPresent(_)
        ));
        assert!(!facts.is_const_function(id));
    }
    assert!(matches!(
        facts.source_non_const_trait_method_contract(&report, constant)?,
        ContractKnowledge::KnownAbsent
    ));
    for (id, expected) in [(runtime, false), (constant, true)] {
        let function = facts.functions[facts.function_indices[&id]];
        let body = function.body.ok_or("actual default body")?;
        assert_eq!(
            facts
                .source_function_context(&report, id, body)?
                .checks_const(),
            expected
        );
        assert_eq!(facts.default_self_clause(id, body)?.is_some(), expected);
    }
    for function in &facts.functions {
        if facts
            .associated
            .owner(function.definition)
            .and_then(|id| facts.associated.get(id))
            .is_some_and(|owner| matches!(owner.kind, OwnerKind::Impl { constant: true, .. }))
        {
            assert!(facts.is_const_function(function.definition));
            assert_eq!(
                facts.function_context(function.definition, function.body.ok_or("impl body")?)?,
                crate::sema::function_context::FunctionContext::ConstFunction
            );
        }
    }
    Ok(())
}
#[test]
fn real_source_runtime_required_and_const_calls_emit_observer_lowir() -> TestResult {
    for name in ["source-runtime", "source-default-const"] {
        let source = input(name)?;
        let report = report(&source, "nonconst_native")?;
        let pipeline::Analysis {
            mut facts,
            prepared,
        } = pipeline::analyze_source(&report)?;
        if name == "source-default-const" {
            check_default_entries(&report, &mut facts)?;
        }
        let program = crate::lower::instances::build_roots_prepared(
            &report,
            &mut facts,
            crate::lower::Roots {
                entry: None,
                exports: &[],
            },
            prepared,
        )
        .map_err(|error| format!("{name}: {error}"))?;
        let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../obj/parallel/nonconst-members-v1/candidate")
            .join(name);
        std::fs::create_dir_all(&directory)?;
        std::fs::write(directory.join("source.rs"), source)?;
        crate::lowir::writer::write(
            &program,
            &mut std::fs::File::create(directory.join("provider.lowir"))?,
        )?;
    }
    Ok(())
}
fn check_default_entries(report: &Report, facts: &mut Declarations) -> TestResult {
    let definition = method(report, facts, "constant")?;
    let callable = facts
        .methods
        .calls
        .values()
        .find(|call| call.method == definition)
        .ok_or("actual default call")?
        .callable;
    let instance = crate::types::instance::Instance::new(&facts.types, callable)?;
    let execution = facts.check_const_default(report, instance)?;
    let function = facts.functions[facts.function_indices[&definition]];
    let mut original = crate::body::ir::Body::new(
        crate::body::ir::Owner::Function {
            definition,
            signature: function.signature,
            execution: None,
        },
        function.body.ok_or("actual default source")?,
        crate::types::TypeId::UNIT,
    );
    assert!(execution.prepare(report, facts, &mut original).is_err());
    assert!(!original.const_context);
    assert!(facts.check_const_member_execution(definition).is_err());
    let Some(crate::types::Kind::FunctionItem {
        item,
        arguments,
        signature,
    }) = facts.types.kind(callable)
    else {
        return Err("actual default function item".into());
    };
    let Some(crate::types::Kind::Function {
        parameters,
        bound_regions,
        unsafe_,
        convention,
        ..
    }) = facts.types.kind(signature)
    else {
        return Err("actual default signature".into());
    };
    let signature = facts.types.intern(crate::types::Kind::Function {
        parameters,
        bound_regions,
        unsafe_,
        convention,
        result: crate::types::TypeId::UNIT,
    })?;
    let wrong = facts.types.intern(crate::types::Kind::FunctionItem {
        item,
        arguments,
        signature,
    })?;
    let wrong = crate::types::instance::Instance::new(&facts.types, wrong)?;
    assert!(facts.check_const_default(report, wrong).is_err());
    let key = facts.functions[facts.function_indices[&definition]].node;
    let choice = local(facts)?
        .declarations
        .get_mut(&key)
        .and_then(|header| header.promises.as_deref_mut())
        .ok_or("default promises")?
        .member
        .take()
        .ok_or("completed default absence")?;
    assert!(facts.check_const_default(report, instance).is_err());
    local(facts)?
        .declarations
        .get_mut(&key)
        .and_then(|header| header.promises.as_deref_mut())
        .ok_or("default promises")?
        .member = Some(choice);
    facts.check_const_default(report, instance)?;
    Ok(())
}
#[test]
fn ordinary_configured_macro_and_stability_owners_retain_actual_member_semantics() -> TestResult {
    for name in [
        "required-runtime",
        "default-runtime",
        "ordinary-trait",
        "const-impl",
        "ordinary-impl-body",
        "cfg-disabled",
        "cfg-selected",
        "unsafe-runtime",
        "macro",
        "macro-allowance",
        "staged-api",
    ] {
        let source = input(name)?;
        let report = test_source::resolve(&source, true)?;
        let pipeline::Analysis {
            mut facts,
            prepared,
        } = pipeline::analyze_source(&report).map_err(|error| format!("{name}: {error}"))?;
        crate::lower::instances::inspect_prepared(&report, &mut facts, prepared, |_, _| ())
            .map_err(|error| format!("{name}: {error}"))?;
    }
    Ok(())
}
#[test]
fn marker_never_grants_const_unsafe_owner_form_or_stability_permissions() -> TestResult {
    for name in [
        "unmarked-default",
        "const-call-denied",
        "ufcs-const-denied",
        "const-impl-body",
        "wrong-free",
        "wrong-inherent",
        "wrong-impl-method",
        "wrong-trait",
        "list",
        "value",
        "duplicate",
        "unsafe-wrapper",
        "missing-gate",
        "unsafe-denied",
        "unsafe-const-denied",
        "always-owner-denied",
        "staged-const-policy-denied",
    ] {
        let source = input(name)?;
        let result = test_source::resolve(&source, true)
            .map_err(Into::into)
            .and_then(|report| {
                let pipeline::Analysis {
                    mut facts,
                    prepared,
                } = pipeline::analyze_source(&report)?;
                crate::lower::instances::inspect_prepared(
                    &report,
                    &mut facts,
                    prepared,
                    |_, _| (),
                )?;
                Ok::<_, Box<dyn std::error::Error>>(())
            });
        let error = result
            .err()
            .ok_or_else(|| format!("{name}: invalid source accepted"))?;
        assert!(
            !error.to_string().contains("unresolved attribute macro"),
            "{name}: {error}"
        );
        println!("{name}: {error}");
    }
    Ok(())
}
#[test]
fn replay_requires_current_choice_owner_effect_function_and_source_image() -> TestResult {
    let source = input("source-runtime")?;
    let report = report(&source, "nonconst_replay")?;
    let mut facts = pipeline::analyze_source(&report)?.facts;
    let id = method(&report, &facts, "runtime")?;
    let key = facts.functions[facts.function_indices[&id]].node;
    let choice = local(&mut facts)?
        .declarations
        .get_mut(&key)
        .and_then(|header| header.promises.as_deref_mut())
        .ok_or("actual promises")?
        .member
        .take()
        .ok_or("actual choice")?;
    assert!(
        facts
            .source_non_const_trait_method_contract(&report, id)
            .is_err()
    );
    local(&mut facts)?
        .declarations
        .get_mut(&key)
        .and_then(|header| header.promises.as_deref_mut())
        .ok_or("actual promises")?
        .member = Some(choice);
    let foreign = self::report(&source, "nonconst_other_source")?;
    assert!(
        facts
            .source_non_const_trait_method_contract(&foreign, id)
            .is_err()
    );
    let Some(Choice::Marked(marker)) = local(&mut facts)?
        .declarations
        .get_mut(&key)
        .and_then(|header| header.promises.as_deref_mut())
        .and_then(|promises| promises.member.as_mut())
    else {
        return Err("actual source marker".into());
    };
    let old = marker.effect;
    marker.effect = usize::MAX - 1;
    assert!(
        facts
            .source_non_const_trait_method_contract(&report, id)
            .is_err()
    );
    if let Some(Choice::Marked(marker)) = local(&mut facts)?
        .declarations
        .get_mut(&key)
        .and_then(|header| header.promises.as_deref_mut())
        .and_then(|promises| promises.member.as_mut())
    {
        marker.effect = old;
    }
    facts.source_non_const_trait_method_contract(&report, id)?;
    Ok(())
}
#[test]
fn cold_trait_choices_and_failed_warm_queries_are_paid_and_scale_linearly() -> TestResult {
    ordinary_default_indexes_scale()?;
    let source = input("source-runtime")?;
    let (foundation, _) = source
        .split_once("fn helper()")
        .ok_or("actual foundation")?;
    let mut previous = None;
    for width in [8, 32, 128] {
        let mut source = foundation.to_owned();
        for index in 0..width {
            writeln!(
                source,
                "#[rustc_const_unstable(feature=\"const_trait_impl\",issue=\"143802\")]pub const trait T{index}:Sized{{#[rustc_non_const_trait_method]fn runtime(&self)->u32{{7}}fn ordinary(&self)->u32{{11}}}}"
            )?;
        }
        let report = report(&source, "nonconst_scaling")?;
        let facts = pipeline::analyze_source(&report)?.facts;
        let storage =
            super::super::source_contract_storage(&facts).ok_or("actual source payments")?;
        println!(
            "nonconst width={width} work={} bytes={}",
            storage.0, storage.1
        );
        if let Some((work, bytes)) = previous {
            assert!(storage.0 <= work * 5 && storage.1 <= bytes * 5);
        }
        previous = Some((storage.0, storage.1));
    }
    let report = report(&source, "nonconst_budget")?;
    let mut facts = pipeline::analyze_source(&report)?.facts;
    let id = method(&report, &facts, "runtime")?;
    let before = super::super::source_contract_storage(&facts).ok_or("actual payments")?;
    for _ in 0..128 {
        facts.source_non_const_trait_method_contract(&report, id)?;
    }
    let after = super::super::source_contract_storage(&facts).ok_or("warm payments")?;
    assert_eq!(before.1, after.1);
    assert!(after.0 > before.0);
    local(&mut facts)?
        .steps
        .set(crate::support::crates::MAX_COMPONENTS);
    assert!(
        facts
            .source_non_const_trait_method_contract(&report, id)
            .is_err()
    );
    assert!(
        facts
            .source_non_const_trait_method_contract(&report, id)
            .is_err()
    );
    assert!(local(&mut facts)?.steps.get() > crate::support::crates::MAX_COMPONENTS);
    println!(
        "Stability Draft/Role/Frame/marker-row={:?}",
        crate::resolve::stability::member_marker_layouts()
    );
    println!(
        "Marker={} Choice={} Promises={} Header={} Node={} Body={}",
        size_of::<Marker>(),
        size_of::<Choice>(),
        size_of::<super::super::Promises>(),
        size_of::<Header>(),
        size_of::<crate::syntax::ast::Node>(),
        size_of::<crate::body::ir::Body>()
    );
    println!(
        "Transient DefaultExecution={}",
        size_of::<crate::sema::function_context::DefaultExecution>()
    );
    assert_eq!(size_of::<Header>(), 80);
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    Ok(())
}
fn ordinary_default_indexes_scale() -> TestResult {
    let mut previous = None;
    for width in [8, 32, 128] {
        let mut source = String::from("#![feature(const_trait_impl)]const trait Defaults{");
        for index in 0..width {
            write!(source, "fn f{index}()->u8{{7}}")?;
        }
        source.push('}');
        let report = test_source::resolve(&source, true)?;
        let pipeline::Analysis {
            mut facts,
            prepared,
        } = pipeline::analyze_source(&report)?;
        assert!(!facts.uses_source_copy());
        let before = facts.body_work.canonical_steps;
        crate::lower::instances::build_roots_prepared(
            &report,
            &mut facts,
            crate::lower::Roots {
                entry: None,
                exports: &[],
            },
            prepared,
        )?;
        let work = facts.body_work.canonical_steps - before;
        let (_, bytes) =
            super::super::source_contract_storage(&facts).ok_or("ordinary header payments")?;
        println!("ordinary default indexes width={width} work={work} bytes={bytes}");
        if let Some((old_work, old_bytes)) = previous {
            assert!(work <= old_work * 5 && bytes <= old_bytes * 5);
        }
        previous = Some((work, bytes));
    }
    Ok(())
}
#[test]
fn failed_marker_publication_keeps_choice_empty_and_ordinary_free_functions_sparse() -> TestResult {
    let source = input("source-runtime")?;
    let report = report(&source, "nonconst_no_publication")?;
    let mut facts = pipeline::analyze_source(&report)?.facts;
    let id = method(&report, &facts, "runtime")?;
    let key = facts.functions[facts.function_indices[&id]].node;
    let mut actual = local(&mut facts)?
        .declarations
        .get_mut(&key)
        .ok_or("actual header")?
        .promises
        .take()
        .ok_or("actual promises")?;
    let Choice::Marked(marker) = actual.member.take().ok_or("actual member")? else {
        return Err("marked member".into());
    };
    let owner = header(&facts, key)
        .and_then(|row| row.owners.get(marker.owner_slot))
        .ok_or("actual owner")?
        .key;
    local(&mut facts)?
        .bytes
        .set(crate::support::crates::MAX_BYTES);
    for _ in 0..2 {
        assert!(
            consume(
                &report,
                &mut facts,
                owner,
                key,
                marker.attribute,
                marker.effect
            )
            .is_err()
        );
        assert!(header(&facts, key).is_some_and(|row| row.promises.is_none()));
    }
    assert!(local(&mut facts)?.bytes.get() > crate::support::crates::MAX_BYTES);
    let ordinary = test_source::resolve("fn plain()->u32{7}", true)?;
    let facts = pipeline::analyze_source(&ordinary)?.facts;
    assert!(
        super::super::source_contract_storage(&facts)
            .is_none_or(|storage| storage.0 == 0 && storage.1 == 0 && storage.2 == 0)
    );
    Ok(())
}
#[test]
fn marked_trait_constness_has_an_explicit_old_transport_boundary() -> TestResult {
    let source = input("source-runtime")?;
    let report = report(&source, "nonconst_transport")?;
    let facts = pipeline::analyze_source(&report)?.facts;
    let marked = method(&report, &facts, "runtime")?;
    let error = facts
        .reject_function_advisory_transport(&report, marked)
        .err()
        .ok_or("marked const trait member transport lost its restriction")?;
    assert!(
        error
            .to_string()
            .contains("non-const trait member restrictions"),
        "{error}"
    );
    let unmarked = method(&report, &facts, "constant")?;
    facts.reject_function_advisory_transport(&report, unmarked)?;
    Ok(())
}
#[test]
fn exact_original_iterator_method_keeps_its_real_const_generic_header_and_body() -> TestResult {
    let header = test_source::fixture(&format!("{BASE}original-trait-header.txt"))?;
    let method = test_source::fixture(&format!("{BASE}original-next-chunk.txt"))?;
    let source = format!("{header}\n{method}\n}}");
    let ast =
        crate::syntax::expression::parse_crate(&source).map_err(|error| format!("{error:?}"))?;
    let actual = ast
        .items
        .iter()
        .find(|item| {
            item.name.is_some_and(|name| {
                &source[name.start as usize..name.end as usize] == "next_chunk_back"
            })
        })
        .ok_or("actual original next_chunk_back")?;
    assert_eq!(actual.generics.end - actual.generics.start, 1);
    assert_eq!(actual.where_clause.end - actual.where_clause.start, 1);
    assert!(matches!(
        actual.kind,
        ItemKind::Function {
            flags: FunctionFlags {
                constant: false,
                asynchronous: false,
                ..
            },
            body: Some(_),
            result: Some(_),
            ..
        }
    ));
    assert!(
        ast.items
            .iter()
            .any(|item| matches!(item.kind, ItemKind::Trait { constant: true, .. }))
    );
    assert_eq!(ast.generics.len(), 1);
    assert_eq!(
        ast.generics[0].flavor,
        crate::syntax::item::GenericFlavor::Const
    );
    assert!(source.contains("crate::array::iter_next_chunk_back(self)"));
    Ok(())
}
