//! Projected candidate conditions reuse semantic selection without granting body permission.
use crate::{native_references as native, resolve, semantic};
use rustgm::{expand::config::Config, pipeline, support::diagnostics::LintCap};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const ORDINARY: &str = "pa20/tests/spec/projected-method-inputs";
const SOURCE: &str = "pa31/tests/spec/projected-method-inputs";

#[test]
fn property_conditions_follow_trait_inference_with_concrete_projections() -> Result {
    let cases = resolve::fixture("pa20/tests/spec/late-method-properties/cases.tsv")?;
    assert_eq!(cases.lines().count(), 4);
    for line in cases.lines() {
        let (path, status) = line.split_once('\t').ok_or("late property columns")?;
        let result = super::prove(&resolve::fixture(path)?);
        match (status, result) {
            ("0", Ok(())) => {}
            ("1", Err(error)) if error.to_string().contains("multiple applicable") => {}
            (_, result) => return Err(format!("{path}: {result:?}").into()),
        }
    }
    Ok(())
}

#[test]
fn a_semantic_size_request_leaves_the_next_property_condition_usable() -> Result {
    use rustgm::types::{
        self, GenericArgs, ItemId, Kind, Primitive, TypeId,
        aggregate::{self, Definitions, Repr, Variant},
        infer::{Context, properties::Query},
        parameters::environment::Environment,
        sizedness::Rules,
    };
    use std::{collections::HashMap, sync::Arc};
    let mut store = types::Store::default();
    let family = ItemId(700);
    let output = ItemId(701);
    let owner = ItemId(702);
    let byte = TypeId::primitive(Primitive::U8);
    store.declare_projection(
        output,
        types::projection::Schema {
            trait_item: family,
            sized: false,
            copies: types::copy::Properties::default(),
        },
    )?;
    let application = store.intern_trait_application(types::traits::Application {
        item: family,
        arguments: GenericArgs::EMPTY,
    })?;
    let terminal = store.intern(Kind::Projection(types::projection::Application {
        item: output,
        subject: byte,
        trait_application: application,
        arguments: GenericArgs::EMPTY,
    }))?;
    let mut fields = Definitions::default();
    let tail = store.intern_list(&[terminal])?;
    fields.insert(
        owner,
        aggregate::Kind::Struct,
        Repr::default(),
        &[Variant {
            fields: tail,
            discriminant: 0,
        }],
    )?;
    let mut rules = Rules::default();
    assert!(
        rules
            .prepare(&mut store, &fields, owner, |_, _, _| Ok(false))?
            .is_none()
    );
    let nominal = store.intern(Kind::Adt(owner, GenericArgs::EMPTY))?;
    store.seal_nominal_sizes(Arc::new(rules))?;
    let environment = Arc::new(Environment::new(&store, &[])?);
    let mut properties = Query::new(environment);
    let mut inference = Context::default();
    let selected = HashMap::<ItemId, types::copy::Declaration>::new();
    for ty in [nominal, byte, nominal, byte] {
        let term = inference.known(&store, ty)?;
        let result = properties.try_demand(&mut inference, &mut store, &fields, &selected, term)?;
        if ty == nominal {
            assert!(
                result.is_none(),
                "an unpaid size condition granted permission"
            );
        } else {
            let result = result.ok_or("unrelated property condition lost its query frontier")?;
            assert!(result.sized && result.abilities.copy);
        }
    }
    Ok(())
}

#[test]
fn projected_candidate_work_scales_with_calls_and_ignores_unrelated_types() -> Result {
    let mut previous = None;
    for width in [8, 32, 128] {
        let mut ordinary = None;
        for noise in [0, 128] {
            let text = resolve::fixture(&format!("{ORDINARY}/width-{width}-noise-{noise}.rs"))?;
            let (_, facts) = semantic::analyze(&text)?;
            let work = [
                facts.methods.selection_probes,
                facts.methods.selection_steps,
                facts.body_work.inference_steps,
                facts.body_work.trait_selections,
            ];
            println!("projected candidate width={width} noise={noise} work={work:?}");
            if noise == 0 {
                if let Some(before) = previous {
                    for (old, actual) in std::iter::zip(before, work) {
                        assert!(actual <= 4 * old + 2048, "{width}: {before:?}->{work:?}");
                    }
                }
                previous = Some(work);
                ordinary = Some(work);
            } else {
                assert_eq!(ordinary, Some(work), "width {width} unrelated type work");
            }
        }
    }
    Ok(())
}

fn rejected(error: &str) -> bool {
    ![
        "normalization",
        "not implemented",
        "invalid",
        "budget",
        "exhausted",
        "lost its",
    ]
    .iter()
    .any(|cause| error.contains(cause))
}

#[test]
fn projected_receivers_preserve_bound_checks_and_trait_fallback() -> Result {
    let mut failures = Vec::new();
    let cases = resolve::fixture(&format!("{ORDINARY}/cases.tsv"))?;
    assert_eq!(cases.lines().count(), 21);
    for line in cases.lines() {
        let (path, status) = line.split_once('\t').ok_or("candidate outcome columns")?;
        let result = super::prove(&resolve::fixture(path)?);
        match (status, result) {
            ("0", Ok(())) => {}
            ("1", Err(error)) if rejected(&error.to_string()) => {}
            (_, result) => failures.push(format!("{path}: {:?}", result.err())),
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
fn actual_reborrow_places_prepare_fields_without_moving_borrowed_values() -> Result {
    for (name, accepted) in [
        ("shared", true),
        ("mutable", true),
        ("concrete", true),
        ("move-out", false),
        ("drop-native", true),
    ] {
        let text = resolve::fixture(&format!("pa18/tests/spec/borrowed-place-fields/{name}.rs"))?;
        match (accepted, super::prove(&text)) {
            (true, Ok(())) => {}
            (false, Err(error)) if rejected(&error.to_string()) => {}
            (_, result) => return Err(format!("{name}: {:?}", result.err()).into()),
        }
    }
    for name in ["copy-bound", "generic-referent", "generic-value"] {
        let text = resolve::fixture(&format!("pa18/tests/spec/shared-call-reborrow/{name}.rs"))?;
        checked_library(&text)?;
    }
    let text = resolve::fixture("pa18/tests/spec/shared-call-reborrow/mutable-generic-move.rs")?;
    let error = checked_library(&text)
        .err()
        .ok_or("generic mutable reference was reused after a move")?;
    assert!(rejected(&error.to_string()), "{error}");
    Ok(())
}

fn checked_library(text: &str) -> Result {
    let report = semantic::resolve(text)?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    rustgm::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?;
    Ok(())
}

fn analyzed(path: &str) -> Result {
    let mut config = Config::default();
    config.set_lint_cap(LintCap::Allow);
    let text = resolve::fixture(path)?;
    let report = resolve::source_provider_config(&text, "projected_inputs", &config)?;
    pipeline::analyze_source(&report)?;
    Ok(())
}

#[test]
fn source_slice_receivers_keep_nested_projection_information() -> Result {
    for name in [
        "slice-direct",
        "slice-plain",
        "slice-copy",
        "slice-copy-explicit",
        "slice-copy-field",
        "slice-copy-field-typed",
        "slice-copy-concrete",
    ] {
        analyzed(&format!("{SOURCE}/{name}.rs"))?;
    }
    Ok(())
}

#[test]
fn existing_projection_lookup_requirements_keep_their_declared_language_outcomes() -> Result {
    for name in [
        "free-inferred",
        "core-default-method",
        "unique-inherent",
        "explicit-rigid",
        "explicit-concrete",
        "explicit-concrete-fail",
        "explicit-concrete-fallback",
        "inferred-concrete",
        "inferred-concrete-fail",
        "free-copy",
        "free-sized",
        "method-rigid",
        "method-unresolved",
    ] {
        let result = analyzed(&format!(
            "pa31/tests/spec/projection-lookup-conditions/{name}.rs"
        ));
        if matches!(name, "explicit-concrete-fail" | "inferred-concrete-fail") {
            let error = result.err().ok_or("unpaid projected candidate accepted")?;
            assert!(rejected(&error.to_string()), "{name}: {error}");
        } else {
            result?;
        }
    }
    Ok(())
}

#[test]
#[ignore = "requires the qualified CPPGM backend"]
fn projected_method_priority_executes_the_independently_selected_bodies() -> Result {
    let text = resolve::fixture(&format!("{ORDINARY}/priority-native.rs"))?;
    native::check_literal_cases_capped(
        "pa20-projected-method-inputs/native",
        &[("priority", &text, false)],
    )
}

#[test]
#[ignore = "requires the qualified CPPGM backend"]
fn borrowed_place_preparation_keeps_the_actual_destructor_execution() -> Result {
    let text = resolve::fixture("pa18/tests/spec/borrowed-place-fields/drop-native.rs")?;
    native::check_literal_cases_capped(
        "pa18-borrowed-place-fields/native",
        &[("drop", &text, false)],
    )
}
