//! Source coercion discovery cannot turn a pending associated input into a mismatch.
use crate::provider_traits::{TestResult, lower};
const FOLDER: &str = "pa31/tests/spec/coercion-discovery";

fn inputs() -> TestResult<Vec<(String, String, bool)>> {
    let mut rows = Vec::new();
    for line in crate::resolve::fixture(&format!("{FOLDER}/cases.tsv"))?.lines() {
        let (name, expected) = line.split_once('\t').ok_or("discovery columns")?;
        let accepted = match expected {
            "0" => true,
            "1" => false,
            _ => return Err("invalid discovery status".into()),
        };
        rows.push((
            name.trim_end_matches(".rs").to_owned(),
            crate::resolve::fixture(&format!("{FOLDER}/{name}"))?,
            accepted,
        ));
    }
    assert_eq!(rows.len(), 22);
    Ok(rows)
}

#[test]
fn pending_aggregate_arguments_retain_complete_equality_and_conversion_proofs() -> TestResult {
    let mut failures = Vec::new();
    for (name, source, accepted) in inputs()? {
        match (accepted, lower(&source)) {
            (true, Ok(_)) => {}
            (false, Err(error))
                if ![
                    "normalization",
                    "not implemented",
                    "budget",
                    "exhausted",
                    "invalid",
                ]
                .iter()
                .any(|part| error.to_string().contains(part)) => {}
            (_, result) => failures.push(format!(
                "{name}: {}",
                result.map_or_else(|error| error.to_string(), |_| "accepted".into())
            )),
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
fn coercion_discovery_scales_with_calls_without_unrelated_type_work() -> TestResult {
    let mut previous = None;
    for width in [8, 32, 128] {
        let mut ordinary = None;
        for noise in [0, 128] {
            let source =
                crate::resolve::fixture(&format!("{FOLDER}/width-{width}-noise-{noise}.rs"))?;
            let report = crate::resolve::source_provider(&source, "discovery_work")?;
            let facts = rustgm::pipeline::analyze_source(&report)?.facts;
            let work = [
                facts.body_work.inference_steps,
                facts.body_work.canonical_steps,
                facts.body_work.trait_selections,
                facts.body_work.projection_steps,
                facts.body_work.projection_selections,
                facts.methods.selection_steps,
                facts.methods.selection_probes,
            ];
            println!("coercion width={width} noise={noise} work={work:?}");
            if noise == 0 {
                if let Some(before) = previous {
                    for (old, actual) in std::iter::zip(before, work) {
                        assert!(actual <= 4 * old + 1024, "{before:?}->{work:?}");
                    }
                }
                previous = Some(work);
                ordinary = Some(work);
            } else {
                assert_eq!(ordinary, Some(work), "unrelated type call work");
            }
        }
    }
    Ok(())
}

#[test]
fn mismatch_discovery_preserves_raw_fences_variables_and_identity_errors() -> TestResult {
    use rustgm::types::{
        Kind, Primitive,
        infer::{Constructor, Context, Domain, Error, TermId},
    };
    let source = crate::resolve::fixture(&format!("{FOLDER}/raw-projection-control.rs"))?;
    let report = crate::resolve::source_provider(&source, "discovery_raw")?;
    let facts = rustgm::pipeline::analyze_source(&report)?.facts;
    let member = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find_map(|(i, def)| (report.package.names.get(def.name) == "Out").then_some(i))
        .ok_or("actual Give::Out declaration")?;
    let ty = report.resolution.associated_paths.keys().find_map(|key| {
        facts.node(*key).and_then(|node| node.ty).filter(|ty| {
            matches!(facts.types.kind(*ty), Some(Kind::Projection(p)) if p.item.0 as usize == member)
        })
    }).ok_or("actual Give::Out projection")?;
    let mut infer = Context::default();
    let projected = infer.known(&facts.types, ty)?;
    let byte = infer.known(
        &facts.types,
        rustgm::types::TypeId::primitive(Primitive::U8),
    )?;
    let word = infer.known(
        &facts.types,
        rustgm::types::TypeId::primitive(Primitive::U16),
    )?;
    let variable = infer.variable(Domain::Any)?;
    assert_eq!(
        infer.incompatible(&facts.types, projected, byte),
        Err(Error::Normalization)
    );
    assert!(!infer.unsizing_mismatch(&facts.types, projected, byte)?);
    assert_eq!(
        infer.equate_shape(&facts.types, projected, byte),
        Err(Error::Normalization)
    );
    assert!(!infer.unsizing_mismatch(&facts.types, variable, byte)?);
    assert_eq!(infer.head(&facts.types, variable)?, None);
    let left = infer.construct(Constructor::Tuple, &[byte, projected])?;
    let right = infer.construct(Constructor::Tuple, &[word, byte])?;
    assert!(infer.unsizing_mismatch(&facts.types, left, right)?);
    assert_eq!(
        infer.unsizing_mismatch(&facts.types, TermId(u32::MAX), byte),
        Err(Error::InvalidIdentity)
    );
    Ok(())
}

#[test]
#[ignore = "requires the pinned CPPGM object backend and host C linker"]
fn inferred_source_conversions_execute_actual_selected_values() -> TestResult {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root.join("obj/parallel/conversion-errors/native");
    std::fs::create_dir_all(&folder)?;
    let client = crate::resolve::fixture(&format!("{FOLDER}/observer.c"))?;
    let rows: Vec<_> = inputs()?
        .into_iter()
        .filter(|(_, _, accepted)| *accepted)
        .collect();
    assert_eq!(rows.len(), 16);
    for (name, source, _) in rows {
        let program = lower(&source)?;
        std::fs::write(folder.join(format!("{name}.rs")), source)?;
        crate::ffi_program::check(root, &folder, &name, &program, &client, "7\n")?;
    }
    Ok(())
}
