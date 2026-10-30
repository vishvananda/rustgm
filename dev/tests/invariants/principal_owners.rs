//! Local object ownership does not borrow foreign or compiler-marker permission.
use super::*;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn unmarked_local_objects_keep_real_self_receiver_bounds_and_virtual_bodies() -> Result {
    for (name, accepted) in [
        ("local-nominal", true),
        ("forwarded-virtual", true),
        ("alias", true),
        ("generic-principal", true),
        ("associated-principal", true),
        ("auto-set", true),
        ("generic-object-lifetime", true),
        ("method-bound", true),
        ("missing-method-bound", false),
        ("wrong-principal-argument", false),
        ("object-lifetime-escape", false),
        ("auto-only", false),
        ("wrong-principal-owner", false),
        ("foreign-principal", false),
        ("overlap-unused", false),
        ("invalid-receiver", false),
    ] {
        let source = crate::test_source::fixture(&format!(
            "pa23/tests/spec/members/local-dyn-inherent/{name}.rs"
        ))?;
        let report = crate::test_source::resolve(&source, true)?;
        let result = crate::pipeline::analyze_source(&report);
        assert_eq!(result.is_ok(), accepted, "{name}: {:?}", result.err());
    }
    Ok(())
}

#[test]
fn local_principal_cannot_be_replaced_by_a_nominal_or_missing_source_owner() -> Result {
    let report = crate::test_source::resolve("struct Nominal;trait Value{}fn main(){}", true)?;
    let principal = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find_map(|(index, definition)| {
            (report.package.names.get(definition.name) == "Value"
                && definition.kind == DefKind::Trait)
                .then_some(index)
        })
        .ok_or("actual principal Value")?;
    let nominal = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find_map(|(index, definition)| {
            (report.package.names.get(definition.name) == "Nominal"
                && definition.kind == DefKind::Struct)
                .then_some(index)
        })
        .ok_or("actual nominal")?;
    let definition = DefId(u32::try_from(principal)?);
    let source = report.resolution.definitions[principal]
        .node
        .ok_or("principal source")?;
    let mut checker = Checker::new(&report)?;
    checker.require_local_principal(source, ItemId(definition.0))?;
    assert!(
        checker
            .require_local_principal(source, ItemId(u32::try_from(nominal)?))
            .is_err()
    );
    let id = checker
        .facts
        .associated
        .owner(definition)
        .ok_or("principal owner")?;
    let original = *checker.facts.associated.get(id).ok_or("actual owner")?;
    checker
        .facts
        .associated
        .owner_for_test(id)
        .ok_or("current owner")?
        .source = None;
    assert!(
        checker
            .require_local_principal(source, ItemId(definition.0))
            .is_err()
    );
    *checker
        .facts
        .associated
        .owner_for_test(id)
        .ok_or("current owner")? = original;
    let OwnerKind::Trait {
        definition: current,
        ..
    } = &mut checker
        .facts
        .associated
        .owner_for_test(id)
        .ok_or("current owner")?
        .kind
    else {
        return Err("trait owner".into());
    };
    *current = DefId(u32::try_from(nominal)?);
    assert!(
        checker
            .require_local_principal(source, ItemId(definition.0))
            .is_err()
    );
    Ok(())
}

fn method_work(width: usize, noise: usize) -> Result<[usize; 4]> {
    use std::fmt::Write as _;
    let mut source = String::new();
    for index in 0..width {
        write!(
            source,
            "trait V{index}{{fn value(&self)->u8;}}impl dyn V{index}{{fn forward(&self)->u8{{self.value()}}}}fn use{index}(v:&(dyn V{index}+'static))->u8{{v.forward()}}"
        )?;
    }
    for index in 0..noise {
        write!(source, "struct Noise{index};")?;
    }
    source.push_str("fn main(){}");
    let report = crate::test_source::resolve(&source, true)?;
    let facts = crate::pipeline::analyze_source(&report)?.facts;
    Ok([
        facts.methods.selection_probes,
        facts.methods.selection_steps,
        facts.methods.selection_shape_steps,
        facts.methods.overlap_pairs,
    ])
}

#[test]
fn local_principal_lookup_work_is_linear_and_ignores_unrelated_nominals() -> Result {
    let mut previous = None;
    for width in [16, 64, 256] {
        let ordinary = method_work(width, 0)?;
        let noise = method_work(width, 512)?;
        assert_eq!(ordinary, noise);
        assert_eq!(
            ordinary[3], 0,
            "distinct principals have disjoint owner buckets"
        );
        if let Some(previous) = previous {
            assert!(ordinary[0] <= previous * 5);
        }
        previous = Some(ordinary[0]);
        println!("local-dyn-inherent width={width} work={ordinary:?}");
    }
    Ok(())
}

#[test]
fn forwarded_virtual_and_reified_inherent_body_produce_fresh_native_input() -> Result {
    let source = crate::test_source::fixture(
        "pa23/tests/spec/members/local-dyn-inherent/forwarded-virtual.rs",
    )?;
    let report = crate::test_source::resolve(&source, true)?;
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(&report)?;
    let program = crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?;
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/local-dyn-inherent-v1/candidate");
    std::fs::create_dir_all(&folder)?;
    std::fs::write(folder.join("provider.rs"), source)?;
    crate::lowir::writer::write(
        &program,
        &mut std::fs::File::create(folder.join("provider.lowir"))?,
    )?;
    Ok(())
}
