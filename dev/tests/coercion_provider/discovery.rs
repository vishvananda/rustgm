//! Region-only opaque input shapes do not grant equality or conversion evidence.
use crate::provider_traits::{TestResult, lower};
const PACKET: &str = "pa31/tests/spec/projection-argument-discovery";

fn cases() -> TestResult<Vec<(String, String, bool)>> {
    let mut rows = Vec::new();
    for line in crate::resolve::fixture(&format!("{PACKET}/cases.tsv"))?.lines() {
        let (name, status) = line.split_once('\t').ok_or("projection region columns")?;
        let accepted = match status {
            "0" => true,
            "1" => false,
            _ => return Err("invalid projection region outcome".into()),
        };
        rows.push((
            name.trim_end_matches(".rs").to_owned(),
            crate::resolve::fixture(&format!("{PACKET}/{name}"))?,
            accepted,
        ));
    }
    assert_eq!(rows.len(), 26);
    Ok(rows)
}

#[test]
fn opaque_projection_input_regions_keep_complete_source_and_lifetime_proofs() -> TestResult {
    let mut failures = Vec::new();
    for (name, source, accepted) in cases()? {
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
fn opaque_projection_input_discovery_scales_with_calls_without_type_inventory_scans() -> TestResult
{
    let mut previous = None;
    for width in [8, 32, 128] {
        let mut ordinary = None;
        for noise in [0, 128] {
            let source =
                crate::resolve::fixture(&format!("{PACKET}/width-{width}-noise-{noise}.rs"))?;
            let report = crate::resolve::source_provider(&source, "projection_region_work")?;
            let facts = rustgm::pipeline::analyze_source(&report)?.facts;
            let work = [
                facts.body_work.inference_steps,
                facts.body_work.canonical_steps,
                facts.body_work.trait_selections,
                facts.body_work.projection_steps,
                facts.body_work.projection_selections,
                facts.body_work.unsizing_inputs,
                facts.methods.selection_steps,
                facts.methods.selection_probes,
            ];
            println!("projected input regions width={width} noise={noise} work={work:?}");
            if noise == 0 {
                if let Some(before) = previous {
                    for (old, actual) in std::iter::zip(before, work) {
                        assert!(actual <= 4 * old + 1024, "{before:?}->{work:?}");
                    }
                }
                previous = Some(work);
                ordinary = Some(work);
            } else {
                assert_eq!(ordinary, Some(work), "unrelated type query work");
            }
        }
    }
    Ok(())
}

#[test]
#[ignore = "requires the pinned CPPGM object backend and host C linker"]
fn opaque_projection_input_regions_execute_actual_payloads_and_borrows() -> TestResult {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root.join("obj/parallel/tagged-request/native");
    std::fs::create_dir_all(&folder)?;
    let client = crate::resolve::fixture(&format!("{PACKET}/observer.c"))?;
    let rows: Vec<_> = cases()?
        .into_iter()
        .filter(|(_, _, accepted)| *accepted)
        .collect();
    assert_eq!(rows.len(), 19);
    for (name, source, _) in rows {
        let program = lower(&source)?;
        std::fs::write(folder.join(format!("{name}.rs")), source)?;
        crate::ffi_program::check(root, &folder, &name, &program, &client, "9 23\n")?;
    }
    Ok(())
}

fn opaque_inputs() -> TestResult<(rustgm::types::Store, Vec<rustgm::types::TypeId>)> {
    use rustgm::types::{
        ItemId, Kind, Primitive, RegionId, Store, TypeId,
        arguments::{Argument, ConstKind, Constant},
        projection::{Application, Schema},
    };
    let mut store = Store::default();
    for member in [11, 12] {
        store.declare_projection(
            ItemId(member),
            Schema {
                trait_item: ItemId(10),
                sized: true,
                copies: rustgm::types::copy::Properties::default(),
            },
        )?;
    }
    let byte = TypeId::primitive(Primitive::U8);
    let word = TypeId::primitive(Primitive::U16);
    let mut forms = Vec::new();
    // Baseline, regions only, Self, type, const, member and group boundary.
    for (index, (subject, ty, bits, member, split)) in [
        (byte, byte, 7, 11, false),
        (byte, byte, 7, 11, false),
        (word, byte, 7, 11, false),
        (byte, word, 7, 11, false),
        (byte, byte, 8, 11, false),
        (byte, byte, 7, 12, false),
        (byte, byte, 7, 11, true),
    ]
    .into_iter()
    .enumerate()
    {
        let region = RegionId::bound(0, u16::from(index != 0))?;
        let constant = store.intern_const(ConstKind::Value(Constant {
            ty: TypeId::primitive(Primitive::Usize),
            bits,
        }))?;
        let arguments = [
            Argument::Lifetime(region),
            Argument::Type(ty),
            Argument::Const(constant),
            Argument::Lifetime(region),
        ];
        let boundary = if split { 1 } else { 3 };
        let trait_arguments = store.intern_arguments(&arguments[..boundary])?;
        let trait_application =
            store.intern_trait_application(rustgm::types::traits::Application {
                item: ItemId(10),
                arguments: trait_arguments,
            })?;
        let arguments = store.intern_arguments(&arguments[boundary..])?;
        forms.push(store.intern(Kind::Projection(Application {
            item: ItemId(member),
            subject,
            trait_application,
            arguments,
        }))?);
    }
    Ok((store, forms))
}

#[test]
fn completed_opaque_discovery_preserves_raw_fences_and_complete_input_shapes() -> TestResult {
    use rustgm::types::{
        Primitive, TypeId,
        infer::{Constructor, Context, Error, TermId},
    };
    let (mut store, forms) = opaque_inputs()?;
    let mut infer = Context::default();
    let mut terms = Vec::new();
    for &ty in &forms {
        let term = infer.known(&store, ty)?;
        assert_eq!(infer.head(&store, term), Err(Error::Normalization));
        // Simulate independently completed normal-form producers. This grants
        // no semantic trait/formation permission to the shape-only test.
        infer.apply_normalization(&mut store, term, ty)?;
        terms.push(term);
    }
    let before = infer.region_equalities.len();
    assert!(!infer.unsizing_mismatch(&store, terms[0], terms[1])?);
    assert_eq!(infer.region_equalities.len(), before);
    assert!(infer.incompatible(&store, terms[0], terms[1])?);
    assert_eq!(
        infer.equate(&store, terms[0], terms[1]),
        Err(Error::Mismatch)
    );
    for &changed in &terms[2..] {
        assert!(infer.unsizing_mismatch(&store, terms[0], changed)?);
    }
    let byte = infer.known(&store, TypeId::primitive(Primitive::U8))?;
    let word = infer.known(&store, TypeId::primitive(Primitive::U16))?;
    let left = infer.construct(Constructor::Tuple, &[byte, terms[0]])?;
    let right = infer.construct(Constructor::Tuple, &[word, terms[1]])?;
    assert!(infer.unsizing_mismatch(&store, left, right)?);
    assert_eq!(
        infer.unsizing_mismatch(&store, TermId(u32::MAX), byte),
        Err(Error::InvalidIdentity)
    );
    infer.account_work(1_000_000 - infer.steps - 1)?;
    assert_eq!(
        infer.unsizing_mismatch(&store, terms[0], terms[1]),
        Err(Error::Budget)
    );
    Ok(())
}

#[test]
fn completed_nested_opaque_inputs_are_compared_without_rewalking_their_prefixes() -> TestResult {
    use rustgm::types::{
        ItemId, Kind, Primitive, RegionId, Store, TypeId,
        arguments::Argument,
        infer::Context,
        projection::{Application, Schema},
    };
    let mut previous = None;
    for depth in [8, 32, 128] {
        let mut store = Store::default();
        store.declare_projection(
            ItemId(11),
            Schema {
                trait_item: ItemId(10),
                sized: true,
                copies: rustgm::types::copy::Properties::default(),
            },
        )?;
        let mut infer = Context::default();
        let mut roots = Vec::new();
        for index in [0, 1] {
            let arguments =
                store.intern_arguments(&[Argument::Lifetime(RegionId::bound(0, index)?)])?;
            let trait_application =
                store.intern_trait_application(rustgm::types::traits::Application {
                    item: ItemId(10),
                    arguments,
                })?;
            let mut ty = TypeId::primitive(Primitive::U8);
            for _ in 0..depth {
                ty = store.intern(Kind::Projection(Application {
                    item: ItemId(11),
                    subject: ty,
                    trait_application,
                    arguments: rustgm::types::GenericArgs::EMPTY,
                }))?;
                let term = infer.known(&store, ty)?;
                // Each distinct operation has its own completed producer.
                infer.apply_normalization(&mut store, term, ty)?;
            }
            roots.push(infer.known(&store, ty)?);
        }
        let before = infer.steps;
        assert!(!infer.unsizing_mismatch(&store, roots[0], roots[1])?);
        let steps = infer.steps - before;
        println!("opaque input depth={depth} steps={steps}");
        assert!(steps <= 64 * depth + 128, "depth={depth} steps={steps}");
        if let Some(old) = previous {
            assert!(steps <= 4 * old + 128, "{old}->{steps}");
        }
        previous = Some(steps);
    }
    Ok(())
}
