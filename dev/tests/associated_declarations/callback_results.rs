//! Callback output equations precede inherent lookup on an inferred result.
use crate::{resolve as source, semantic};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn wrapped_callback_premises_keep_actual_inputs_and_formation() -> Result {
    let mut cases = Vec::new();
    for owner in ["pa24", "pa31"] {
        for name in [
            "plain-free-chain",
            "plain-associated-chain",
            "plain-split",
            "plain-annotated",
            "plain-explicit-factory",
            "plain-direct-closure",
            "plain-inherent-byref",
            "detail-factory-self",
            "detail-factory-inputs",
            "detail-factory-first",
            "detail-factory-second",
            "detail-factory-callback",
            "detail-unused-projected",
            "detail-unused-plain",
            "detail-invoke-plain",
            "detail-consume-plain",
        ] {
            cases.push((owner, name, None));
        }
        for (name, cause) in [
            ("opponents-wrong-output", "incompatible types"),
            (
                "opponents-wrong-input",
                "associated type requires an unambiguous validated implementation",
            ),
            (
                "opponents-missing-bound-explicit",
                "associated type requires an unambiguous validated implementation",
            ),
            (
                "opponents-unpaid-parent",
                "trait obligation requires an unambiguous matching assumption or implementation",
            ),
        ] {
            cases.push((owner, name, Some(cause)));
        }
    }
    // Ordinary overrides, ordinary const imports and multiple-input premises
    // remain required positives in their manifests; no failing outcome is blessed.
    for name in ["plain-trait-byref", "plain-trait-byref-forward"] {
        cases.push(("pa31", name, None));
    }
    let mut failures = Vec::new();
    for (owner, name, cause) in cases {
        let text = source::fixture(&format!("{owner}/tests/spec/by-ref-fold/{name}.rs"))?;
        let report = if owner == "pa31" {
            source::source_provider(&text, "wrapped_callback_premises")?
        } else {
            semantic::resolve(&text)?
        };
        let outcome = rustgm::pipeline::analyze_source(&report)
            .and_then(|analysis| complete(&report, analysis));
        match (cause, outcome) {
            (None, Ok(_)) => {}
            (Some(cause), Err(error)) if error.to_string().contains(cause) => {}
            (cause, outcome) => failures.push(format!(
                "{owner}/{name}: expected {cause:?}, got {:?}",
                outcome.err().map(|error| error.to_string())
            )),
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
fn conditional_wrapped_callback_factories_infer_actual_projected_inputs() -> Result {
    // Full lowering retains the existing source const-call owner requirement.
    // These four accepting artifact contracts remain in pending.json.
    for name in [
        "associated-chain",
        "annotated",
        "explicit-factory",
        "trait-byref",
    ] {
        let text = source::fixture(&format!("pa31/tests/spec/by-ref-fold/const-{name}.rs"))?;
        let report = source::source_provider(&text, "conditional_wrapped_callback")?;
        rustgm::pipeline::analyze_source(&report)?;
    }
    Ok(())
}

#[test]
fn wrapped_callback_premises_scale_with_actual_projected_arguments() -> Result {
    for owner in ["pa24", "pa31"] {
        let mut previous = None;
        for width in [8, 32, 128] {
            let mut baseline = None;
            for noise in [0, 128] {
                let text = source::fixture(&format!(
                    "{owner}/tests/spec/by-ref-fold/scale-width-{width}-noise-{noise}.rs"
                ))?;
                let report = if owner == "pa31" {
                    source::source_provider(&text, "wrapped_callback_scaling")?
                } else {
                    semantic::resolve(&text)?
                };
                let analysis = rustgm::pipeline::analyze_source(&report)?;
                let work = [
                    analysis.facts.body_work.inference_steps,
                    analysis.facts.body_work.trait_selections,
                    analysis.facts.methods.selection_steps,
                    analysis.facts.methods.selection_probes,
                ];
                complete(&report, analysis)?;
                println!("wrapped {owner} width={width} noise={noise} work={work:?}");
                if noise == 0 {
                    if let Some(old) = previous {
                        for (current, before) in work.into_iter().zip(old) {
                            assert!(current <= 4 * before + 256);
                        }
                    }
                    previous = Some(work);
                    baseline = Some(work);
                } else {
                    assert_eq!(Some(work), baseline, "unrelated types caused body work");
                }
            }
        }
    }
    Ok(())
}

#[test]
fn callable_output_shapes_reach_consumers_without_waiving_projection_proof() -> Result {
    let mut failures = Vec::new();
    for owner in ["pa24", "pa31"] {
        for (name, accept) in [
            ("chain", true),
            ("split", true),
            ("annotated", true),
            ("qualified", true),
            ("explicit-constructor", true),
            ("closure", true),
            ("known-item", true),
            ("equal-projections", true),
            ("array-item", true),
            ("literal-call-input", true),
            ("parameter-call-input", true),
            ("wrong-output-reject", false),
            ("wrong-item-reject", false),
            ("different-projection-reject", false),
            ("missing-parent-reject", false),
        ] {
            let text = source::fixture(&format!(
                "{owner}/tests/spec/callable-result-heads/{name}.rs"
            ))?;
            let report = if owner == "pa31" {
                source::source_provider(&text, "callable_output_shapes")?
            } else {
                semantic::resolve(&text)?
            };
            let analysis = rustgm::pipeline::analyze_source(&report);
            if !accept {
                let cause = match name {
                    "wrong-item-reject" => {
                        "body types require a provable associated-result equality"
                    }
                    "missing-parent-reject" => {
                        "associated type requires an unambiguous implementation"
                    }
                    _ => "incompatible types",
                };
                if !analysis
                    .as_ref()
                    .is_err_and(|error| error.to_string().contains(cause))
                {
                    failures.push(format!(
                        "{owner}/{name}: expected {cause}, got {:?}",
                        analysis.err().map(|error| error.to_string())
                    ));
                }
                continue;
            }
            let outcome = analysis.and_then(|analysis| complete(&report, analysis));
            if let Err(error) = outcome {
                failures.push(format!("{owner}/{name}: {error}"));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

fn complete(
    report: &rustgm::resolve::expansion::Report,
    rustgm::pipeline::Analysis {
        mut facts,
        prepared,
    }: rustgm::pipeline::Analysis,
) -> std::result::Result<rustgm::lowir::program::Program, rustgm::resolve::package::Error> {
    rustgm::lower::instances::build_roots_prepared(
        report,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )
}

#[test]
fn nonempty_callable_binders_keep_their_actual_lifetime_requirements() -> Result {
    let mut failures = Vec::new();
    for owner in ["pa24", "pa31"] {
        for (name, reject) in [
            ("higher-ranked-input", false),
            ("static-input-reject", true),
        ] {
            let text = source::fixture(&format!(
                "{owner}/tests/spec/callable-result-heads/{name}.rs"
            ))?;
            let report = if owner == "pa31" {
                source::source_provider(&text, "nonempty_callable_binder")?
            } else {
                semantic::resolve(&text)?
            };
            let result = rustgm::pipeline::analyze_source(&report)
                .and_then(|analysis| complete(&report, analysis))
                .map(|_| ());
            match result {
                Ok(()) if !reject => {}
                Err(error)
                    if reject
                        && error
                            .to_string()
                            .contains("required lifetime does not outlive its target") => {}
                result => failures.push(format!(
                    "{owner}/{name}: expected reject={reject}, got {result:?}"
                )),
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
fn projected_callback_results_scale_without_unrelated_type_work() -> Result {
    let mut previous = None;
    for width in [8, 32, 128] {
        let mut baseline = None;
        for noise in [0, 128] {
            let text = source::fixture(&format!(
                "pa31/tests/spec/callable-result-heads/width-{width}-noise-{noise}.rs"
            ))?;
            let report = source::source_provider(&text, "callback_result_scaling")?;
            let analysis = rustgm::pipeline::analyze_source(&report)?;
            let work = [
                analysis.facts.body_work.inference_steps,
                analysis.facts.body_work.trait_selections,
                analysis.facts.methods.selection_steps,
                analysis.facts.methods.selection_probes,
            ];
            complete(&report, analysis)?;
            println!("callback width={width} noise={noise} work={work:?}");
            if noise == 0 {
                if let Some(old) = previous {
                    for (before, actual) in std::iter::zip(old, work) {
                        assert!(actual <= 4 * before + 2048, "{old:?}->{work:?}");
                    }
                }
                previous = Some(work);
                baseline = Some(work);
            } else {
                assert_eq!(baseline, Some(work));
            }
        }
    }
    Ok(())
}

#[test]
fn input_reconciliation_defers_partial_projections_without_false_cycles() -> Result {
    use rustgm::types::{
        ItemId, Primitive, Store, TypeId,
        copy::Properties,
        infer::{Context, Domain, Error, arguments::Argument, refinement::Refinement},
        projection::Schema,
        substitute::Bound,
    };
    let mut store = Store::default();
    let item = ItemId(804);
    store.declare_projection(
        item,
        Schema {
            trait_item: ItemId(803),
            sized: true,
            copies: Properties::default(),
        },
    )?;
    let mut source = Context::default();
    let a = source.variable(Domain::Any)?;
    let b = source.variable(Domain::Any)?;
    let (mut query, roots) =
        Refinement::new(&store, &mut source, &[Argument::Type(a), Argument::Type(b)])?;
    let [Argument::Type(a), Argument::Type(b)] = roots.as_slice() else {
        return Err("missing imported input roots".into());
    };
    let unknown = query.context.variable(Domain::Any)?;
    let known = query
        .context
        .known(&store, TypeId::primitive(Primitive::U8))?;
    let partial = query.context.projected(&store, item, unknown, &[], &[])?;
    let complete = query.context.projected(&store, item, known, &[], &[])?;
    query.context.equate(&store, *a, partial)?;
    query.context.equate(&store, *b, complete)?;
    query
        .publish(&mut store, &mut source, &mut Bound::new(Vec::new()))
        .map_err(|error| format!("input publication: {error}"))?;
    assert_eq!(
        query.refresh(&mut store, &mut source),
        Err(Error::Normalization)
    );
    Ok(())
}

#[test]
fn generic_method_values_keep_input_correlations_and_actual_parent_proof() -> Result {
    let mut failures = Vec::new();
    for owner in ["pa24", "pa31"] {
        for (name, cause) in [
            ("chain-last", None),
            ("method-value-one", None),
            ("method-value-qualified", None),
            ("method-value-explicit-output", None),
            ("flatten-last", None),
            ("flatten-last-unfixed-accumulator", None),
            ("flatten-method-local", None),
            ("flatten-method-free", None),
            ("flatten-parameter-first", None),
            ("map-member", None),
            ("map-member-projected", None),
            ("map-associated", None),
            ("method-value-paid-parent", None),
            ("method-value-concrete-owner", None),
            ("method-value-wrong-result", Some("incompatible types")),
            (
                "method-value-unpaid-parent",
                Some(
                    "trait obligation requires an unambiguous matching assumption or implementation",
                ),
            ),
            (
                "method-value-wrong-owner",
                Some("associated type requires an unambiguous validated implementation"),
            ),
            (
                "method-value-concrete-owner-reject",
                Some("associated type requires an unambiguous validated implementation"),
            ),
        ] {
            let text = source::fixture(&format!(
                "{owner}/tests/spec/callable-result-heads/{name}.rs"
            ))?;
            let report = if owner == "pa31" {
                source::source_provider(&text, "method_value_inputs")?
            } else {
                semantic::resolve(&text)?
            };
            let result = rustgm::pipeline::analyze_source(&report)
                .and_then(|analysis| complete(&report, analysis))
                .map(|_| ());
            match (cause, result) {
                (None, Ok(())) => {}
                (Some(cause), Err(error)) if error.to_string().contains(cause) => {}
                (cause, result) => failures.push(format!("{owner}/{name}: {cause:?}: {result:?}")),
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
fn shared_projected_callback_inputs_scale_through_the_independent_anchor() -> Result {
    let mut previous = None;
    for width in [8, 32, 128] {
        let mut baseline = None;
        for noise in [0, 128] {
            let text = source::fixture(&format!(
                "pa31/tests/spec/callable-result-heads/multi-input-width-{width}-noise-{noise}.rs"
            ))?;
            let report = source::source_provider(&text, "shared_callback_inputs")?;
            let analysis = rustgm::pipeline::analyze_source(&report)?;
            let work = [
                analysis.facts.body_work.inference_steps,
                analysis.facts.body_work.trait_selections,
                analysis.facts.methods.selection_steps,
                analysis.facts.methods.selection_probes,
            ];
            complete(&report, analysis)?;
            println!("callback inputs={width} noise={noise} work={work:?}");
            if noise == 0 {
                if let Some(old) = previous {
                    for (before, actual) in std::iter::zip(old, work) {
                        assert!(actual <= 4 * before + 4096, "{old:?}->{work:?}");
                    }
                }
                previous = Some(work);
                baseline = Some(work);
            } else {
                assert_eq!(baseline, Some(work));
            }
        }
    }
    Ok(())
}

// Generated locally in each test from the registered Source callable prefix.
mod method_result_context {
    use super::{Result, source};
    use rustgm::{
        resolve::unit::{FileId, NodeKey},
        sema::{members::Selection, model::Operation},
        syntax::ast::{Kind, NodeId},
        types,
    };

    const FOUNDATION: &str = r"
pub trait FromIterator<T>: Sized {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self;
}

pub trait Iterator {
    type Item;
    fn collect<B: FromIterator<Self::Item>>(self) -> B
    where
        Self: Sized,
    {
        FromIterator::from_iter(self)
    }
}

pub trait IntoIterator {
    type Item;
    type IntoIter: Iterator<Item = Self::Item>;
    fn into_iter(self) -> Self::IntoIter;
}

impl<I: Iterator> IntoIterator for I {
    type Item = I::Item;
    type IntoIter = I;
    fn into_iter(self) -> I { self }
}

pub enum Ordering { Less, Equal, Greater }
pub trait Ord {
    fn cmp(&self, other: &Self) -> Ordering;
}

pub struct Vec<T> { slot: T }
impl<T> FromIterator<T> for Vec<T> {
    fn from_iter<I: IntoIterator<Item = T>>(_iter: I) -> Self { loop {} }
}
impl<T> Vec<T> {
    fn sort_by<F: FnMut(&T, &T) -> Ordering>(&mut self, _f: F) {}
}

";

    fn source_with(tail: &str) -> Result<String> {
        let fixture = source::fixture("pa31/tests/spec/callable-result-heads/map-associated.rs")?;
        let (prefix, _) = fixture
            .split_once("enum Option<T>")
            .ok_or("Source callable prefix")?;
        Ok(format!("{prefix}{FOUNDATION}{tail}"))
    }

    fn checks_iterator_and_fields(tail: &str, ordinary_call: bool) -> Result {
        let text = source_with(tail)?;
        let report = source::source_provider(&text, "method_result_context")?;
        let facts = rustgm::pipeline::analyze_source(&report)?.facts;
        let ast = &report.package.unit(FileId(0)).configured.ast;
        let mut collect = 0;
        let mut fields = 0;
        let mut free = 0;
        for (index, node) in ast.nodes.iter().enumerate() {
            let key = NodeKey {
                file: FileId(0),
                node: NodeId::new(u32::try_from(index)?),
            };
            if let Kind::MethodCall { name, .. } = node.kind
                && text.get(name.start as usize..name.end as usize) == Some("collect")
            {
                let call = facts.methods.calls.get(&key).ok_or("actual collect call")?;
                let owner = facts
                    .associated
                    .trait_owner(call.method)
                    .ok_or("collect trait")?;
                let definition = &report.resolution.definitions[owner.0 as usize];
                assert_eq!(report.package.names.get(definition.name), "Iterator");
                collect += 1;
            }
            if let Some(Operation::Field {
                field: Selection::Tuple(0),
                ..
            }) = facts.node(key).and_then(|fact| fact.operation)
            {
                fields += 1;
            }
            if ordinary_call && let Kind::Call { callee, .. } = node.kind {
                let callee = NodeKey {
                    node: callee,
                    ..key
                };
                if let Some(types::Kind::FunctionItem { item, .. }) = facts
                    .node(callee)
                    .and_then(|fact| fact.ty)
                    .and_then(|ty| facts.types.kind(ty))
                {
                    let definition = &report.resolution.definitions[item.0 as usize];
                    if report.package.names.get(definition.name) == "collect_pairs" {
                        free += 1;
                    }
                }
            }
        }
        assert_eq!(collect, 1);
        assert_eq!(fields, 2);
        assert_eq!(free, usize::from(ordinary_call));
        Ok(())
    }

    fn rejects(tail: &str, operand: &str, causes: &[&str]) -> Result {
        let text = source_with(tail)?;
        let report = source::source_provider(&text, "method_result_context")?;
        let error = rustgm::pipeline::analyze_source(&report)
            .err()
            .ok_or("invalid method result was accepted")?;
        let start = text.rfind(operand).ok_or("actual rejected expression")?;
        assert_eq!(error.file.source(), Some(FileId(0)));
        assert!(
            (start..start + operand.len()).contains(&(error.span.start as usize)),
            "{error}"
        );
        assert!(
            causes.iter().any(|cause| error.message.starts_with(cause)),
            "{error}"
        );
        Ok(())
    }

    #[test]
    fn partial_nominal_method_results_keep_owned_iterator_proofs() -> Result {
        for tail in [
            r"pub fn sort_pairs<K: Ord, V, I: IntoIterator<Item = (K, V)>>(items: I) -> Vec<(K, V)> {
    let mut inputs: Vec<_> = items.into_iter().collect();
    inputs.sort_by(|a, b| a.0.cmp(&b.0));
    inputs
}
",
            r"pub fn sort_pairs<K: Ord, V, I: Iterator<Item = (K, V)>>(iter: I) -> Vec<(K, V)> {
    let mut inputs: Vec<_> = iter.collect();
    inputs.sort_by(|a, b| a.0.cmp(&b.0));
    inputs
}
",
            r"pub fn sort_pairs<K: Ord, V, I: IntoIterator<Item = (K, V)>>(items: I) -> Vec<(K, V)> {
    let mut inputs: Vec<(K, V)> = items.into_iter().collect();
    inputs.sort_by(|a, b| a.0.cmp(&b.0));
    inputs
}
",
        ] {
            checks_iterator_and_fields(tail, false)?;
        }
        rejects(
            r"pub fn sort_pairs<K, I: IntoIterator<Item = K>>(items: I) -> Vec<K> {
    let mut inputs: Vec<_> = items.into_iter().collect();
    inputs.sort_by(|a, b| a.0.cmp(&b.0));
    inputs
}
",
            "a.0",
            &["field access requires a known tuple or struct type"],
        )?;
        rejects(
            r"pub fn wrong<K, I: Iterator<Item = (K, u16)>>(iter: I) -> Vec<(K, u8)> {
    let inputs: Vec<(_, u8)> = iter.collect();
    inputs
}
",
            "iter.collect()",
            &[
                "incompatible types",
                "body types require a provable associated-result equality",
                "trait obligation requires an unambiguous matching assumption or implementation",
            ],
        )
    }

    #[test]
    fn partial_nominal_free_call_refreshes_its_actual_goal_owner() -> Result {
        checks_iterator_and_fields(
            r"fn collect_pairs<I: Iterator, B: FromIterator<I::Item>>(iter: I) -> B {
    iter.collect()
}

pub fn sort_pairs<K: Ord, V, I: Iterator<Item = (K, V)>>(iter: I) -> Vec<(K, V)> {
    let mut inputs: Vec<_> = collect_pairs(iter);
    inputs.sort_by(|a, b| a.0.cmp(&b.0));
    inputs
}
",
            true,
        )
    }
}
