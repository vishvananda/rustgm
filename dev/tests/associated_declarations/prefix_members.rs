//! Projected type prefixes keep parsed identity before ordinary constant selection.
use crate::{native_references as native, resolve as source, semantic};
use rustgm::resolve::{
    model::{DefKind, Namespace},
    unit::NodeKey,
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa20/tests/spec/projected-values";

#[test]
fn value_prefixes_keep_the_original_bounded_type_and_parsed_segments() -> Result {
    for name in ["nested-shorthand", "self-shorthand"] {
        let text = source::fixture(&format!("{PACKET}/{name}.rs"))?;
        let report = semantic::resolve(&text)?;
        let paths = &report.resolution.associated_paths;
        let mut found = false;
        for (key, value) in paths {
            if value.namespace != Namespace::Value {
                continue;
            }
            let prefix = NodeKey {
                node: value.prefix,
                ..*key
            };
            let Some(ty) = paths.get(&prefix) else {
                continue;
            };
            assert_eq!(ty.namespace, Namespace::Type);
            assert_eq!(ty.suffix.len(), 1);
            assert_eq!(value.suffix.len(), 1);
            assert_eq!(ty.owner, value.owner);
            assert!(matches!(
                report.resolution.definitions[ty.owner.0 as usize].kind,
                DefKind::Generic | DefKind::SelfType
            ));
            let ast = &report.package.unit(key.file).configured.ast;
            let (_, segments) = ast.path_segments(key.node).ok_or("parsed source path")?;
            assert!(segments.contains(&prefix.node));
            assert!(segments.contains(&ty.prefix));
            found = true;
        }
        assert!(found, "{name}");
    }
    let text = source::fixture(&format!("{PACKET}/deep-qualified.rs"))?;
    let report = semantic::resolve(&text)?;
    assert!(report.resolution.qualified_paths.iter().any(|(key, path)| {
        path.namespace == Namespace::Value
            && path.trait_path.is_none()
            && report
                .resolution
                .qualified_paths
                .get(&NodeKey {
                    node: path.ty,
                    ..*key
                })
                .is_some_and(|prefix| {
                    prefix.namespace == Namespace::Type && prefix.trait_path.is_some()
                })
    }));
    Ok(())
}

fn projection_input(
    name: &str,
    function: &str,
) -> Result<(
    rustgm::sema::model::Declarations,
    rustgm::types::TypeId,
    std::sync::Arc<rustgm::types::parameters::environment::Environment>,
)> {
    let text = source::fixture(&format!("{PACKET}/{name}.rs"))?;
    let (report, facts) = semantic::analyze(&text)?;
    let ty = report
        .resolution
        .associated_paths
        .iter()
        .find_map(|(key, path)| {
            (path.namespace == Namespace::Type)
                .then(|| facts.node(*key).and_then(|node| node.ty))
                .flatten()
                .filter(|ty| {
                    matches!(
                        facts.types.kind(*ty),
                        Some(rustgm::types::Kind::Projection(_))
                    )
                })
        })
        .ok_or("actual projected prefix")?;
    let owner = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, definition)| {
            definition.kind == DefKind::Function
                && report.package.names.get(definition.name) == function
        })
        .and_then(|(index, _)| u32::try_from(index).ok())
        .map(rustgm::resolve::model::DefId)
        .ok_or("actual generic function")?;
    let environment = facts
        .generics
        .environments
        .get(&owner)
        .cloned()
        .ok_or("actual caller environment")?;
    Ok((facts, ty, environment))
}

#[test]
fn candidate_properties_require_formation_and_keep_copy_absence() -> Result {
    use rustgm::types::infer::{Context, Error, properties::Query};
    for (name, function, copy) in [
        ("nested-qualified", "zero", true),
        ("noncopy-prefix", "value", false),
    ] {
        let (mut facts, ty, environment) = projection_input(name, function)?;
        let mut raw = Context::default();
        let root = raw.known(&facts.types, ty)?;
        assert_eq!(
            raw.head(&facts.types, root).err(),
            Some(Error::Normalization)
        );
        let concrete = raw.known(
            &facts.types,
            rustgm::types::TypeId::primitive(rustgm::types::Primitive::U16),
        )?;
        assert_eq!(
            raw.equate_shape(&facts.types, root, concrete),
            Err(Error::Normalization)
        );
        let mut query = Query::new(std::sync::Arc::clone(&environment));
        assert_eq!(
            query
                .demand(
                    &mut raw,
                    &mut facts.types,
                    &facts.aggregates,
                    &facts.derives,
                    root
                )
                .err(),
            Some("candidate properties require associated type normalization")
        );
        let mut formed = Context::default();
        let root = formed.known(&facts.types, ty)?;
        formed.apply_normalization(&mut facts.types, root, ty)?;
        let _ = formed.head(&facts.types, root)?;
        let mut query = Query::new(std::sync::Arc::clone(&environment));
        let value = query.demand(
            &mut formed,
            &mut facts.types,
            &facts.aggregates,
            &facts.derives,
            root,
        )?;
        assert!(value.sized);
        assert_eq!((value.abilities.copy, value.abilities.clone_), (copy, copy));
        let concrete = formed.known(
            &facts.types,
            rustgm::types::TypeId::primitive(rustgm::types::Primitive::U16),
        )?;
        assert_eq!(
            formed.equate_shape(&facts.types, root, concrete),
            Err(Error::Normalization)
        );
        formed.account_work(1_000_000 - formed.steps)?;
        for _ in 0..2 {
            assert!(
                query
                    .demand(
                        &mut formed,
                        &mut facts.types,
                        &facts.aggregates,
                        &facts.derives,
                        root
                    )
                    .is_err()
            );
        }
    }
    Ok(())
}

#[test]
fn projected_constant_values_retain_all_declared_typed_outcomes() -> Result {
    let mut failures = Vec::new();
    let mut count = 0;
    for row in source::fixture(&format!("{PACKET}/cases.tsv"))?.lines() {
        let (path, expected) = row.split_once('\t').ok_or("projected constant columns")?;
        let text = source::fixture(path)?;
        let result = semantic::analyze(&text).and_then(|(mut report, mut facts)| {
            let entry =
                rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
            rustgm::lower::instances::build(&report, &mut facts, entry).map_err(|e| e.to_string())
        });
        match (expected, result) {
            ("0", Ok(_)) => {}
            ("1", Err(error))
                if ![
                    "not implemented",
                    "missing value resolution",
                    "invalid method candidate",
                    "normalization",
                    "budget",
                    "exhausted",
                ]
                .iter()
                .any(|part| error.contains(part)) => {}
            (_, result) => failures.push(format!(
                "{path}: {}",
                result.err().unwrap_or_else(|| "accepted".into())
            )),
        }
        count += 1;
    }
    assert_eq!(count, 27);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
fn projected_constant_lookup_work_scales_without_unrelated_declarations() -> Result {
    let mut previous = None;
    for width in [8, 32, 128] {
        let mut control = None;
        for noise in [0, 128] {
            let text = source::fixture(&format!("{PACKET}/width-{width}-noise-{noise}.rs"))?;
            let (_, facts) = semantic::analyze(&text)?;
            let work = (
                facts.body_work.inference_steps,
                facts.body_work.canonical_steps,
            );
            if noise == 0 {
                if let Some((inference, canonical)) = previous {
                    assert!(
                        work.0 <= 4 * inference + 256,
                        "width{width} inference {work:?}"
                    );
                    assert!(
                        work.1 <= 4 * canonical + 256,
                        "width{width} canonical {work:?}"
                    );
                }
                previous = Some(work);
                control = Some(work);
            } else {
                let (inference, canonical) = control.ok_or("missing width control")?;
                assert!(
                    work.0 <= inference,
                    "width{width} added inference {inference}->{work:?}"
                );
                assert_eq!(work.1, canonical, "width{width} added canonical work");
            }
        }
    }
    Ok(())
}

#[test]
#[ignore = "requires the qualified CPPGM backend"]
fn projected_constants_execute_actual_selected_values_and_borrows() -> Result {
    let mut inputs = Vec::new();
    for row in source::fixture(&format!("{PACKET}/cases.tsv"))?.lines() {
        let (path, expected) = row.split_once('\t').ok_or("native projected columns")?;
        if expected == "0" {
            let name = path
                .rsplit('/')
                .next()
                .and_then(|name| name.strip_suffix(".rs"))
                .ok_or("native projected name")?
                .to_owned();
            inputs.push((name, source::fixture(path)?));
        }
    }
    assert_eq!(inputs.len(), 20);
    let cases: Vec<_> = inputs
        .iter()
        .map(|(name, text)| (name.as_str(), text.as_str(), false))
        .collect();
    native::check_literal_cases("pa20-projected-values/native", &cases)
}

#[test]
fn source_projection_lookup_controls_preserve_their_existing_boundaries() -> Result {
    let cases = [
        ("free-inferred", None),
        ("core-default-method", None),
        ("unique-inherent", None),
        ("explicit-rigid", None),
        ("explicit-concrete", None),
        (
            "explicit-concrete-fail",
            Some("no applicable trait method for this prefix"),
        ),
        ("explicit-concrete-fallback", None),
        ("inferred-concrete", None),
        (
            "inferred-concrete-fail",
            Some("trait obligation requires an unambiguous matching assumption or implementation"),
        ),
        ("free-copy", None),
        ("free-sized", None),
        ("method-rigid", None),
        ("method-unresolved", None),
    ];
    for (name, expected) in cases {
        let text = source::fixture(&format!(
            "pa31/tests/spec/projection-lookup-conditions/{name}.rs"
        ))?;
        let report = source::source_provider(&text, "projection_lookup_control")?;
        // This frozen source packet qualifies analysis. Its separate full
        // lowering limits are unchanged on the immutable baseline (see receipts).
        let result = rustgm::pipeline::analyze_source(&report)
            .map(|_| ())
            .map_err(|error| error.to_string());
        match (expected, result) {
            (None, Ok(())) => {}
            (Some(expected), Err(actual)) => assert!(actual.contains(expected), "{name}: {actual}"),
            (expected, result) => {
                return Err(format!("{name}: expected {expected:?}, got {result:?}").into());
            }
        }
    }
    Ok(())
}
