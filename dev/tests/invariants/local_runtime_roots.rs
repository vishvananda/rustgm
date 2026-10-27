//! Explicit body roots reuse ordinary typing, originals and executable proof.
use super::*;
use crate::{resolve::model::DefKind, test_source};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn definition(report: &Report, name: &str) -> TestResult<DefId> {
    report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, row)| {
            row.kind == DefKind::Function && report.package.names.get(row.name) == name
        })
        .map(|(index, _)| u32::try_from(index).map(DefId))
        .transpose()?
        .ok_or_else(|| format!("missing actual function {name}").into())
}

#[test]
fn explicit_roots_leave_unused_bodies_pending_without_whole_source_or_artifact_success()
-> TestResult {
    use std::fmt::Write as _;
    for count in [8, 32, 128] {
        let mut source = String::from("fn helper(x:u8)->u8{x+1}fn main(){let _=helper(6);}");
        for index in 0..count {
            write!(source, "fn pending{index}()->u8{{true}}")?;
        }
        let report = test_source::resolve(&source, true)?;
        let entry = definition(&report, "main")?;
        let helper = definition(&report, "helper")?;
        let selected = analyze(&report, &[entry, helper, helper])?;
        assert_eq!(selected.facts.typed_function_bodies.len(), 2);
        assert!(
            !selected
                .facts
                .typed_function_bodies
                .contains(&definition(&report, "pending0")?)
        );
        let whole_error = crate::pipeline::analyze_source(&report)
            .err()
            .ok_or("invalid unused body gained whole-source success")?;
        assert!(whole_error.to_string().contains("type"), "{whole_error}");
        let identity = crate::support::crates::CrateIdentity::new("selected", "pending_body")?;
        let mut export_candidate = analyze(&report, &[entry, helper])?;
        assert!(
            crate::lower::instances::build_library_prepared(
                &report,
                &mut export_candidate.facts,
                identity,
                Prepared::default()
            )
            .is_err()
        );
        let program = selected.build(entry)?;
        println!(
            "local-roots pending={count} selected=2 lowered={}",
            program
                .symbols
                .iter()
                .filter(|symbol| symbol.body.is_some())
                .count()
        );
        assert_eq!(
            program
                .symbols
                .iter()
                .filter(|symbol| symbol.body.is_some())
                .count(),
            3
        );
    }
    assert_eq!(size_of::<crate::body::ir::Body>(), 448);
    println!(
        "local-roots Body={} Value={} Rvalue={} Code={} Request={}",
        size_of::<crate::body::ir::Body>(),
        size_of::<crate::body::ir::Value>(),
        size_of::<crate::body::ir::Rvalue>(),
        size_of::<crate::const_eval::code::Code>(),
        size_of::<crate::sema::values::execution::Requirement>()
    );
    Ok(())
}

#[test]
fn missing_direct_indirect_and_cleanup_originals_never_gain_execution_permission() -> TestResult {
    for source in [
        "fn helper(){}fn main(){helper();}",
        "fn helper(){}fn main(){let f:fn()=helper;f();}",
        "struct D;impl Drop for D{fn drop(&mut self){}}fn main(){let _d=D;}",
    ] {
        let report = test_source::resolve(source, true)?;
        let entry = definition(&report, "main")?;
        let selected = analyze(&report, &[entry])?;
        assert!(
            selected.build(entry).is_err(),
            "a demanded Pending original executed: {source}"
        );
    }
    for source in [
        "fn main(){}const BAD:u8=true;",
        "fn main(){}static BAD:u8=true;",
        "fn main(){}fn bad_signature(x:Missing){}",
    ] {
        match test_source::resolve(source, true) {
            Ok(report) => {
                let entry = definition(&report, "main")?;
                assert!(analyze(&report, &[entry]).is_err());
            }
            Err(error) => assert!(
                source.contains("Missing"),
                "unexpected resolver rejection {error}"
            ),
        }
    }
    let report = test_source::resolve("struct S;fn main(){}", true)?;
    let entry = definition(&report, "main")?;
    assert!(analyze(&report, &[]).is_err());
    assert!(analyze(&report, &[DefId(u32::MAX)]).is_err());
    let selected = analyze(&report, &[entry])?;
    assert!(selected.build(DefId(u32::MAX)).is_err());
    Ok(())
}

#[test]
fn source_abort_and_generic_helpers_use_the_same_actual_call_instance_and_body_proofs() -> TestResult
{
    let ordinary = test_source::resolve(
        "fn id<T>(x:T)->T{x}fn main(){let _=id::<u8>(7);}fn pending()->u8{true}",
        true,
    )?;
    let entry = definition(&ordinary, "main")?;
    let helper = definition(&ordinary, "id")?;
    analyze(&ordinary, &[entry, helper])?.build(entry)?;
    assert!(analyze(&ordinary, &[entry])?.build(entry).is_err());
    let source = r#"#![feature(no_core,lang_items,intrinsics,rustc_attrs)]#![no_core]
        #[lang="pointee_sized"]trait PointeeSized{}
        #[lang="meta_sized"]trait MetaSized:PointeeSized{}
        #[lang="sized"]trait Sized:MetaSized{}
        #[rustc_intrinsic]fn abort()->!;
        fn terminal()->!{abort()}
        fn main(){if false {terminal();}}
        fn pending()->u8{true}"#;
    let report = test_source::source_provider(source, "selected_abort")?;
    let entry = definition(&report, "main")?;
    let terminal = definition(&report, "terminal")?;
    let selected = analyze(&report, &[entry, terminal])?;
    assert_eq!(selected.facts.typed_function_bodies.len(), 2);
    selected.build(entry)?;
    Ok(())
}
