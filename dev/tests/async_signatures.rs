//! Actual source Future results retain output equations and required authored bodies.
use crate::resolve as test_source;
use rustgm::{pipeline, sema, types::Kind};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn analyze(path: &str) -> TestResult<rustgm::sema::model::Declarations> {
    let source = test_source::fixture(path)?;
    let report = test_source::source_provider(&source, "async_signatures")?;
    Ok(pipeline::analyze_source(&report)?.facts)
}

#[test]
fn source_async_calls_prove_output_and_keep_opaque_argument_identity() -> TestResult {
    let manifest = test_source::fixture("pa42/tests/spec/async-signatures/cases.tsv")?;
    let mut count = 0;
    let mut failures = Vec::new();
    for line in manifest.lines() {
        let (path, status) = line.split_once('\t').ok_or("two case columns")?;
        let result = analyze(path);
        match (status, result) {
            ("0", Ok(_)) => {}
            ("0", Err(error)) => failures.push(format!("{path}: {error}")),
            ("1", Ok(_)) => failures.push(format!("accepted {path}")),
            ("1", Err(error)) => {
                let message = error.to_string();
                if [
                    "hidden type requires",
                    "storage requires",
                    "semantic contract is not implemented",
                    "invalid or exhausted",
                ]
                .iter()
                .any(|fence| message.contains(fence))
                {
                    failures.push(format!("{path}: {message}"));
                }
            }
            _ => return Err("invalid declared outcome".into()),
        }
        count += 1;
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    assert_eq!(count, 22);
    Ok(())
}

#[test]
fn async_body_output_is_checked_without_inventing_future_storage() -> TestResult {
    let source = test_source::fixture("pa42/tests/spec/async-signatures/scalar.rs")?;
    let report = test_source::source_provider(&source, "async_representation")?;
    let analysis = pipeline::analyze_source(&report)?;
    let facts = analysis.facts;
    let function = facts
        .functions
        .iter()
        .find(|function| {
            report
                .package
                .names
                .get(report.resolution.definitions[function.definition.0 as usize].name)
                == "value"
        })
        .ok_or("missing source async function")?;
    let body = function.body.ok_or("missing authored body")?;
    assert_eq!(
        facts.node(body).and_then(|node| node.ty),
        Some(facts.opaques.body_results[&function.definition])
    );
    let Some(Kind::Function { result, .. }) = facts.types.kind(function.signature) else {
        return Err("missing actual signature".into());
    };
    let Some(Kind::Opaque(id, _)) = facts.types.kind(result) else {
        return Err("async result is not an opaque identity".into());
    };
    let declaration = facts
        .opaques
        .declarations
        .get(&id)
        .ok_or("missing async declaration")?;
    assert!(
        declaration.hidden.is_none(),
        "signature checking fabricated a future state machine"
    );
    assert_eq!(declaration.bounds.len(), 2);
    assert_ne!(facts.opaques.body_results[&function.definition], result);
    Ok(())
}

#[test]
fn trait_async_method_keeps_a_required_declaration_and_source_projection_consumer() -> TestResult {
    let source = test_source::fixture("pa42/tests/spec/async-signatures/trait-method.rs")?;
    let report = test_source::source_provider(&source, "async_trait")?;
    let facts = sema::declarations::analyze(&report)?;
    let method = facts
        .functions
        .iter()
        .find(|function| {
            report
                .package
                .names
                .get(report.resolution.definitions[function.definition.0 as usize].name)
                == "compute"
        })
        .ok_or("missing required async method")?;
    assert!(method.body.is_none());
    let definition = method.definition;
    let analysis = pipeline::analyze(&report, facts)?;
    assert!(
        analysis
            .facts
            .functions
            .iter()
            .any(|function| function.definition == definition && function.body.is_none())
    );
    let consumer = analysis
        .facts
        .functions
        .iter()
        .find(|function| {
            report
                .package
                .names
                .get(report.resolution.definitions[function.definition.0 as usize].name)
                == "consumer"
        })
        .ok_or("missing actual consumer")?;
    assert_eq!(
        analysis
            .facts
            .node(consumer.body.ok_or("missing consumer body")?)
            .and_then(|node| node.ty),
        Some(rustgm::types::TypeId::UNIT)
    );
    Ok(())
}

#[test]
fn async_cfg_demand_fails_before_publishing_an_eager_body() -> TestResult {
    let source = test_source::fixture("pa42/tests/spec/async-signatures/scalar.rs")?;
    let report = test_source::source_provider(&source, "async_cfg")?;
    let analysis = pipeline::analyze_source(&report)?;
    let facts = analysis.facts;
    let function = facts
        .functions
        .iter()
        .find(|function| {
            report
                .package
                .names
                .get(report.resolution.definitions[function.definition.0 as usize].name)
                == "value"
        })
        .ok_or("missing source async function")?;
    let mut prepared = rustgm::body::templates::Prepared::default();
    let before = prepared.work();
    let error = prepared
        .function(&report, &facts, function)
        .err()
        .ok_or("async body became an ordinary eager CFG")?;
    assert!(
        error
            .message
            .contains("async state-machine construction is not implemented"),
        "{error}"
    );
    assert_eq!(prepared.work(), before);
    Ok(())
}

#[test]
fn source_async_capture_arguments_reuse_lifetime_type_and_const_coordinates() -> TestResult {
    use rustgm::types::arguments::Argument;
    for (name, kinds) in [
        ("borrowed-capture", "l"),
        ("generic", "t"),
        ("const-capture", "c"),
    ] {
        let facts = analyze(&format!("pa42/tests/spec/async-signatures/{name}.rs"))?;
        let mut found = false;
        for declaration in facts.opaques.declarations.values() {
            let Some(Kind::Opaque(_, arguments)) = facts.types.kind(declaration.application) else {
                return Err("invalid async application".into());
            };
            let actual: String = facts
                .types
                .arguments(arguments)
                .ok_or("missing captures")?
                .map(|argument| match argument {
                    Argument::Lifetime(_) => 'l',
                    Argument::Type(_) => 't',
                    Argument::Const(_) => 'c',
                })
                .collect();
            assert_eq!(actual, kinds, "{name}");
            assert!(declaration.hidden.is_none());
            found = true;
        }
        assert!(found, "{name}");
    }
    Ok(())
}

#[test]
fn independent_companions_preserve_each_authored_body_byte_for_byte() -> TestResult {
    let manifest = test_source::fixture("pa42/tests/spec/async-signatures/cases.tsv")?;
    for line in manifest.lines() {
        let (path, _) = line.split_once('\t').ok_or("two case columns")?;
        let source = test_source::fixture(path)?;
        let (_, body) = source.split_once("#[lang=\"future_trait\"] pub trait Future { #[lang=\"future_output\"] type Output; }\n").ok_or("actual source prefix")?;
        let name = std::path::Path::new(path)
            .file_name()
            .ok_or("fixture name")?
            .to_str()
            .ok_or("fixture UTF-8")?;
        let oracle = test_source::fixture(&format!(
            "pa42/tests/spec/async-signatures/independent/{name}"
        ))?;
        let (_, oracle_body) = oracle
            .split_once("use core::future::Future;\n")
            .ok_or("independent source prefix")?;
        assert_eq!(body.as_bytes(), oracle_body.as_bytes(), "{path}");
    }
    Ok(())
}

#[test]
fn nested_authored_opaque_output_infers_without_inventing_future_storage() -> TestResult {
    let source = test_source::fixture("pa42/tests/spec/async-signatures/nested-output.rs")?;
    let report = test_source::source_provider(&source, "async_nested")?;
    let facts = pipeline::analyze_source(&report)?.facts;
    let function = facts
        .functions
        .iter()
        .find(|function| {
            report
                .package
                .names
                .get(report.resolution.definitions[function.definition.0 as usize].name)
                == "value"
        })
        .ok_or("missing source async function")?;
    let Some(Kind::Function { result, .. }) = facts.types.kind(function.signature) else {
        return Err("missing function signature".into());
    };
    let Some(Kind::Opaque(future, _)) = facts.types.kind(result) else {
        return Err("missing future identity".into());
    };
    let ids = &facts.opaques.owners[&function.definition];
    assert_eq!(ids.len(), 2);
    assert!(facts.opaques.declarations[&future].hidden.is_none());
    let authored = ids
        .iter()
        .find(|id| **id != future)
        .ok_or("missing authored opaque result")?;
    assert_eq!(
        facts.opaques.declarations[authored].hidden,
        Some(rustgm::types::TypeId::primitive(
            rustgm::types::Primitive::U8
        ))
    );
    Ok(())
}

#[test]
fn required_unsized_output_declarations_preserve_the_actual_source_schema() -> TestResult {
    let manifest =
        test_source::fixture("pa42/tests/spec/async-signatures/reviewed-sized-cases.tsv")?;
    let mut count = 0;
    for line in manifest.lines() {
        let (path, status) = line.split_once('\t').ok_or("two case columns")?;
        assert_eq!(status, "0");
        let source = test_source::fixture(path)?;
        let report = test_source::source_provider(&source, "async_output_schema")?;
        let facts = pipeline::analyze_source(&report)?.facts;
        let output = facts
            .associated
            .members()
            .iter()
            .find(|member| {
                report
                    .package
                    .names
                    .get(report.resolution.definitions[member.definition.0 as usize].name)
                    == "Output"
            })
            .ok_or("missing actual source Output")?;
        let schema = facts
            .types
            .projection_schema(rustgm::types::ItemId(output.definition.0))
            .ok_or("missing actual Output schema")?;
        assert_eq!(schema.sized, !path.ends_with("relaxed-output-required.rs"));
        count += 1;
    }
    assert_eq!(count, 3);
    Ok(())
}

#[test]
fn source_pin_and_async_signatures_require_real_receiver_proof() -> TestResult {
    let mut count = 0;
    for line in test_source::fixture("pa43/tests/spec/pin-async-signatures/cases.tsv")?.lines() {
        let (path, status) = line.split_once('\t').ok_or("case columns")?;
        match (status, analyze(path)) {
            ("0", Ok(facts)) => {
                let declaration = facts
                    .opaques
                    .declarations
                    .values()
                    .next()
                    .ok_or("actual required async future")?;
                let function = facts
                    .functions
                    .iter()
                    .find(|f| f.definition == declaration.owner)
                    .ok_or("actual required method")?;
                assert!(function.body.is_none());
                assert!(declaration.hidden.is_none());
                assert_eq!(
                    facts.opaques.body_results.get(&function.definition),
                    Some(&rustgm::types::TypeId::UNIT)
                );
                let Some(Kind::Opaque(_, arguments)) = facts.types.kind(declaration.application)
                else {
                    return Err("actual future application".into());
                };
                let arguments = facts
                    .types
                    .arguments(arguments)
                    .ok_or("actual captures")?
                    .collect::<Vec<_>>();
                assert!(
                    arguments
                        .iter()
                        .any(|a| matches!(a, rustgm::types::arguments::Argument::Lifetime(_)))
                );
                assert!(arguments.iter().any(|a| matches!(a, rustgm::types::arguments::Argument::Type(t) if matches!(facts.types.kind(*t),Some(Kind::Parameter(_))))));
            }
            ("0", Err(error)) => return Err(format!("{path}: {error}").into()),
            ("1", Ok(_)) => return Err(format!("accepted {path}").into()),
            ("1", Err(error)) => {
                let message = error.to_string();
                assert!(
                    !message.contains("not implemented")
                        && !message.contains("invalid or exhausted"),
                    "{path}: {message}"
                );
            }
            _ => return Err("invalid declared outcome".into()),
        }
        count += 1;
    }
    assert_eq!(count, 3);
    Ok(())
}
