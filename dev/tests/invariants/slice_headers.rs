//! Actual source methods retain their bodies; declaration tags never synthesize Length.
use super::*;
use crate::{pipeline, sema::header_attributes::ContractKnowledge, test_source};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn fixture(name: &str) -> Result<String> {
    Ok(test_source::fixture(&format!(
        "pa31/tests/spec/source-slice-methods/{name}.rs"
    ))?)
}
fn prefix() -> Result<String> {
    Ok(fixture("generic-tagged-inherent-complete")?
        .split_once("struct Widget<T>")
        .ok_or("actual source foundation")?
        .0
        .to_owned())
}
fn checked(input: &str) -> Result<(Report, pipeline::Analysis)> {
    let report = test_source::source_provider(input, "source_slice_methods")?;
    let analysis = pipeline::analyze_source(&report)?;
    Ok((report, analysis))
}
fn definition(report: &Report, name: &str) -> Result<DefId> {
    report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, row)| report.package.names.get(row.name) == name)
        .map(|(index, _)| u32::try_from(index).map(DefId))
        .transpose()?
        .ok_or_else(|| format!("missing actual {name}").into())
}
#[test]
fn actual_string_slice_bodies_keep_const_runtime_proof_and_no_bootstrap_methods() -> Result {
    for name in [
        "module-primitive-fallback",
        "imported-module-primitive-fallback",
    ] {
        let input = test_source::fixture(&format!("pa6/tests/spec/names/{name}.rs"))?;
        let ordinary = test_source::resolve(&input, true)?;
        pipeline::analyze_source(&ordinary)?;
    }
    let input = fixture("actual-methods")?;
    let (
        report,
        pipeline::Analysis {
            mut facts,
            prepared,
        },
    ) = checked(&input)?;
    assert!(report.resolution.library_types.slice_length().is_none());
    assert!(
        report
            .resolution
            .intrinsics
            .get(crate::support::intrinsics::Intrinsic::ConstPointerAdd)
            .is_none()
    );
    assert_eq!(LangItem::SliceLen.index(), 60);
    let actual = report
        .resolution
        .source_bindings
        .as_deref()
        .and_then(|source| source.language(LangItem::SliceLen))
        .ok_or("actual slice method")?;
    assert!(actual.parent.is_some());
    assert!(facts.typed_function_bodies.contains(&actual.definition));
    assert!(matches!(
        facts.no_implicit_autorefs(&report, actual.definition)?,
        ContractKnowledge::KnownPresent(())
    ));
    let program = crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?;
    for name in ["str_length", "slice_length", "const_length"] {
        assert_eq!(
            program
                .symbols
                .iter()
                .filter(|s| s.name == name && s.body.is_some())
                .count(),
            1
        );
    }
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/pa31/source-slice-methods-v1/native");
    std::fs::create_dir_all(&folder)?;
    std::fs::write(folder.join("provider.rs"), input)?;
    crate::lowir::writer::write(
        &program,
        &mut std::fs::File::create(folder.join("provider.lowir"))?,
    )?;
    Ok(())
}
#[test]
fn inherent_tag_is_unconstrained_and_named_methods_require_actual_owners() -> Result {
    let (report, analysis) = checked(&fixture("generic-tagged-inherent-complete")?)?;
    let method = report
        .resolution
        .source_bindings
        .as_deref()
        .and_then(|s| s.language(LangItem::SliceLen))
        .ok_or("generic inherent role")?;
    assert!(
        analysis
            .facts
            .typed_function_bodies
            .contains(&method.definition)
    );
    assert_eq!(
        report
            .package
            .names
            .get(report.resolution.definitions[method.definition.0 as usize].name),
        "named"
    );
    for (declaration, receiver) in [
        ("struct Item;impl Item{fn len(&self)->usize{99}}", "Item"),
        (
            "trait Length{fn len(&self)->usize;}struct Item;impl Length for Item{fn len(&self)->usize{99}}",
            "Item",
        ),
        (
            "struct Item;impl Item{#[lang=\"slice_len_fn\"]fn len(&self)->usize{99}}",
            "Item",
        ),
    ] {
        checked(&format!(
            "{}{declaration}pub fn use_len(x:&{receiver})->usize{{x.len()}}",
            prefix()?
        ))?;
    }
    for declaration in [
        "struct Item;",
        "struct Item;struct Other;impl Other{fn len(&self)->usize{99}}",
    ] {
        let input = format!(
            "{}{declaration}pub fn use_len(x:&Item)->usize{{x.len()}}",
            prefix()?
        );
        assert!(checked(&input).is_err(), "missing/wrong selected owner");
    }
    for name in ["tag-free-function", "tag-trait-method"] {
        assert!(
            test_source::source_provider(&fixture(name)?, "bad_slice_target").is_err(),
            "{name}"
        );
    }
    let ordinary = test_source::resolve("fn main(){let x=[1u8,2,3];let _=x.len();}", true)?;
    assert!(ordinary.resolution.library_types.slice_length().is_some());
    pipeline::analyze_source(&ordinary)?;
    let mut previous = None;
    for width in [8, 32, 128] {
        let calls = "let _=x.len();".repeat(width);
        let input = format!(
            "{}struct Item;impl Item{{fn len(&self)->usize{{99}}}}fn many(x:&Item){{{calls}}}",
            prefix()?
        );
        let (report, analysis) = checked(&input)?;
        assert!(report.resolution.library_types.slice_length().is_none());
        assert!(analysis.facts.methods.demanded_length().is_none());
        let steps = analysis.facts.body_work.inference_steps;
        if let Some((old_width, old_steps)) = previous {
            assert!(steps <= old_steps * (width / old_width) + 128);
        }
        previous = Some((width, steps));
        eprintln!(
            "source method width={width} inference={steps} definitions={}",
            report.resolution.definitions.len()
        );
    }
    Ok(())
}
#[test]
fn retained_marker_checks_forms_incarnation_and_raw_implicit_receiver_boundary() -> Result {
    let report =
        test_source::source_provider(&fixture("marker-allowed-targets")?, "marker_targets")?;
    let facts = crate::sema::declarations::analyze(&report)?;
    assert!(matches!(
        facts.no_implicit_autorefs(&report, definition(&report, "free")?)?,
        ContractKnowledge::KnownPresent(())
    ));
    for name in [
        "marker-duplicate",
        "marker-arguments",
        "marker-struct",
        "marker-foreign",
        "marker-missing-gate",
        "marker-unsafe-wrapper",
    ] {
        let result =
            test_source::source_provider(&fixture(name)?, "marker_opponent").and_then(|report| {
                crate::sema::declarations::analyze(&report).map_err(|error| error.to_string())
            });
        assert!(result.is_err(), "{name}");
    }
    let declaration =
        "struct Item;impl Item{#[rustc_no_implicit_autorefs]fn len(&self)->usize{99}}";
    checked(&format!(
        "{}{declaration}pub fn safe(x:&Item)->usize{{x.len()}}pub unsafe fn explicit(p:*const Item)->usize{{unsafe{{(&*p).len()}}}}",
        prefix()?
    ))?;
    let input = format!(
        "{}{declaration}pub unsafe fn implicit(p:*const Item)->usize{{unsafe{{(*p).len()}}}}",
        prefix()?
    );
    let error = checked(&input)
        .err()
        .ok_or("marked raw implicit receiver accepted")?;
    assert!(
        error
            .to_string()
            .contains("source marked raw-pointer implicit receiver use is not implemented"),
        "{error}"
    );
    let input = input.replace("#[rustc_no_implicit_autorefs]", "");
    let (report, analysis) = checked(&input)?;
    assert!(matches!(
        analysis
            .facts
            .no_implicit_autorefs(&report, definition(&report, "len")?)?,
        ContractKnowledge::KnownAbsent
    ));
    let changed = test_source::source_provider(&input.replace("99", "98"), "source_slice_methods")?;
    assert!(
        analysis
            .facts
            .no_implicit_autorefs(&changed, definition(&changed, "len")?)
            .is_err()
    );
    Ok(())
}
