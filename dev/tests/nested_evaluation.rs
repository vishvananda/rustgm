//! Inline constants retain independent bodies while their results refine caller types.
#[path = "harness/native_references.rs"]
mod native;
#[path = "nested_evaluation/output.rs"]
mod output;
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "helpers/resolve.rs"]
mod source;
#[path = "nested_evaluation/written.rs"]
mod written;
use rustgm::{
    lower::{constant_view, instances},
    pipeline,
    sema::inline_constants::Evaluation,
    types::{Primitive, TypeId},
};

fn analyze(text: &str) -> Result<(rustgm::resolve::expansion::Report, pipeline::Analysis), String> {
    let report = source::resolve(text, true)?;
    let analysis = pipeline::analyze_source(&report).map_err(|e| e.to_string())?;
    Ok((report, analysis))
}

fn inspect(text: &str) -> Result<(), String> {
    let (
        report,
        pipeline::Analysis {
            mut facts,
            prepared,
        },
    ) = analyze(text)?;
    instances::inspect_prepared(&report, &mut facts, prepared, |facts, checked| {
        constant_view::inspect(facts, checked)
    })
    .map_err(|e| e.to_string())?
    .map_err(str::to_owned)?;
    Ok(())
}

#[test]
fn inline_result_waits_for_surrounding_constraints() -> Result<(), String> {
    let (_, analysis) =
        analyze("fn take(_:u8){}pub fn run(){take(const{7});let x=const{7};let _:u16=x;}")?;
    let bodies = analysis
        .facts
        .inline_constants
        .as_deref()
        .ok_or("missing inline bodies")?;
    let results: Vec<_> = bodies.bodies().map(|body| body.result).collect();
    assert_eq!(
        results,
        [
            Some(TypeId::primitive(Primitive::U8)),
            Some(TypeId::primitive(Primitive::U16))
        ]
    );
    assert_eq!(bodies.publications, 2);
    Ok(())
}

#[test]
fn inline_locals_and_control_do_not_capture_the_enclosing_body() {
    for text in [
        "pub fn run(x:u8)->u8{const{x}}",
        "pub fn run()->u8{const{return 7}}",
        "pub fn run()->u8{loop{const{break 7};}}",
    ] {
        assert!(analyze(text).is_err(), "{text}");
    }
}

#[test]
fn closed_inline_sources_reach_the_complete_checked_pipeline() -> Result<(), String> {
    for text in [
        "pub fn run()->u8{const{7}}",
        "pub fn run()->u8{const{let value=7;let p=&value;*p}}",
        "const fn add(x:u8)->u8{x+1}pub fn run()->u8{const{add(6)}}",
        "pub struct Value(u8);pub fn run()->[Value;2]{[const{Value(7)};2]}",
        "const fn divide(n:u8)->u8{1/n}pub fn run()->[u8;0]{[const{const{divide(0)}};0]}",
        "const fn divide(n:u8)->u8{1/n}pub fn run(){return;let _=const{divide(0)};}",
        "pub fn run()->u8{let f=||{const{7}};f()}",
    ] {
        inspect(text).map_err(|error| format!("{text}: {error}"))?;
    }
    Ok(())
}

#[test]
fn direct_inline_closure_body_has_distinct_semantic_owner_and_execution_root() -> Result<(), String>
{
    let text = "pub fn run()->u8{let f=||const{7};f()}";
    let (_, analysis) = analyze(text)?;
    let closure = analysis
        .facts
        .closures
        .all()
        .first()
        .ok_or("missing closure")?;
    assert_ne!(closure.source, closure.body);
    assert_eq!(closure.semantic_source(), closure.source);
    let inline = analysis
        .facts
        .inline_constants
        .as_deref()
        .and_then(|registry| registry.get(closure.body))
        .ok_or("missing direct inline child")?;
    assert_eq!(inline.parent_body, closure.source);
    inspect(text)?;
    for text in [
        "fn value()->u8{7}pub fn run()->u8{let f=||const{value()};f()}",
        "pub fn run()->u8{let f=||const{return 7};f()}",
        "pub fn run(x:u8)->u8{let f=||const{x};f()}",
        "const fn divide(n:u8)->u8{1/n}pub fn run(){let _f=||const{divide(0)};}",
    ] {
        let error = inspect(text).err().unwrap_or_default();
        assert!(
            !error.is_empty() && !error.contains("distinct checked body ownership"),
            "{text}: {error}"
        );
    }
    Ok(())
}

#[test]
fn suppressed_values_still_require_type_and_const_eligibility() {
    for text in [
        "fn value()->u8{7}pub fn run(){return;let _=const{value()};}",
        "fn value()->u8{7}pub fn run()->[u8;0]{[const{const{value()}};0]}",
    ] {
        let error = inspect(text).err().unwrap_or_default();
        assert!(error.contains("non-const"), "{text}: {error}");
    }
}

#[test]
fn retained_inline_values_are_required_in_unused_and_false_bodies() {
    for text in [
        "const fn divide(n:u8)->u8{1/n}fn unused(){let _=const{divide(0)};}",
        "const fn divide(n:u8)->u8{1/n}pub fn run(){if false{let _=const{divide(0)};}}",
        "const fn divide(n:u8)->u8{1/n}const C:u8=if false{const{divide(0)}}else{7};",
        "const fn divide(n:u8)->u8{1/n}pub fn run(){let _f=||{const{divide(0)}};}",
        "const fn divide(n:u8)->u8{1/n}pub fn run()->u8{let f=||{const{divide(0)}};if false{f()}else{7}}",
    ] {
        let error = inspect(text).err().unwrap_or_default();
        assert!(error.contains("divides by zero"), "{text}: {error}");
    }
}

#[test]
fn zero_repeat_and_unreachable_status_do_not_manufacture_values() -> Result<(), String> {
    let (_, analysis) = analyze(
        "pub fn run()->[u8;0]{[const{const{7}};0]}pub fn unused(){return;let _=const{9};}",
    )?;
    let bodies = analysis
        .facts
        .inline_constants
        .as_deref()
        .ok_or("missing inline bodies")?;
    let statuses: Vec<_> = bodies.bodies().map(|body| body.evaluation).collect();
    assert_eq!(
        statuses,
        [
            Evaluation::SuppressedZeroRepeat,
            Evaluation::SuppressedZeroRepeat,
            Evaluation::DeferredUnreachable
        ]
    );
    assert!(bodies.bodies().all(|body| body.value.is_none()));
    assert!(bodies.suppression_visits <= 8);
    Ok(())
}

#[test]
fn early_and_late_queries_reuse_each_inline_cfg_and_value() -> Result<(), String> {
    for text in [
        "const fn value()->usize{const{3}}type A=[u8;value()];const C:usize=value();pub fn run()->usize{value()}",
        "pub fn run()->u8{const{7}}",
    ] {
        let (
            report,
            pipeline::Analysis {
                mut facts,
                prepared,
            },
        ) = analyze(text)?;
        let count = facts
            .inline_constants
            .as_deref()
            .ok_or("missing inline registry")?
            .indexed;
        assert_eq!(facts.body_work.cfg_inline_expressions.get(), count);
        instances::inspect_prepared(&report, &mut facts, prepared, |facts, checked| {
            constant_view::inspect(facts, checked)
        })
        .map_err(|e| e.to_string())?
        .map_err(str::to_owned)?;
        let registry = facts
            .inline_constants
            .as_deref()
            .ok_or("lost inline registry")?;
        assert_eq!(registry.publications, count);
        assert_eq!(registry.preparation_visits, count);
        assert_eq!(registry.value_publications, count);
        assert_eq!(facts.body_work.cfg_inline_expressions.get(), count);
        assert!(facts.constant_work.expression_bytes > 0);
        assert!(facts.constant_work.template_bytes >= facts.constant_work.expression_bytes);
    }
    Ok(())
}

#[test]
fn distinct_inline_preparation_has_bounded_index_and_cfg_work() -> Result<(), String> {
    for count in [8_usize, 32, 128] {
        let mut text = String::from("pub fn run()->u8{");
        for _ in 0..count {
            text.push_str("let _=const{7u8};");
        }
        text.push_str("7}");
        let (
            report,
            pipeline::Analysis {
                mut facts,
                prepared,
            },
        ) = analyze(&text)?;
        instances::inspect_prepared(&report, &mut facts, prepared, |facts, checked| {
            constant_view::inspect(facts, checked)
        })
        .map_err(|e| e.to_string())?
        .map_err(str::to_owned)?;
        let registry = facts
            .inline_constants
            .as_deref()
            .ok_or("missing inline registry")?;
        assert_eq!(registry.indexed, count);
        assert_eq!(registry.publications, count);
        assert_eq!(registry.preparation_visits, count);
        assert_eq!(registry.value_publications, count);
        assert_eq!(facts.body_work.cfg_inline_expressions.get(), count);
        assert!(registry.lookups.get() < count * 128);
        assert!(registry.retained_bytes < count * 1024);
        assert!(facts.constant_work.expression_bytes < count * 32 * 1024);
    }
    Ok(())
}

#[test]
fn inline_inspection_preserves_status_and_root_order() -> Result<(), String> {
    let text = "const C:u8=7;pub fn run(){let _=const{7u8};let _=[const{9u8};0];return;let _=const{11u8};}";
    let (
        report,
        pipeline::Analysis {
            mut facts,
            prepared,
        },
    ) = analyze(text)?;
    let inventory = instances::inspect_prepared(&report, &mut facts, prepared, |facts, checked| {
        constant_view::inspect(facts, checked)
    })
    .map_err(|e| e.to_string())?
    .map_err(str::to_owned)?;
    let mut bytes = Vec::new();
    inventory.write(&mut bytes).map_err(|e| e.to_string())?;
    let view = String::from_utf8(bytes).map_err(|e| e.to_string())?;
    assert_eq!(view.matches("kind=inline").count(), 3);
    assert_eq!(view.matches("root inline").count(), 1);
    assert!(view.contains("status=suppressed-zero-repeat"));
    assert!(view.contains("status=deferred-unreachable"));
    let initializer = view
        .find("root initializer")
        .ok_or("missing initializer root")?;
    let inline = view.find("root inline").ok_or("missing inline root")?;
    assert!(initializer < inline);
    assert!(
        view.lines()
            .filter(|line| line.starts_with("root "))
            .all(|line| line.contains("value=v0"))
    );
    Ok(())
}

#[test]
fn zero_atomic_repeat_uses_the_owning_pa28_source() -> Result<(), String> {
    let text = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pa28/tests/spec/atomic-storage/zero-array-repeat-return.rs"),
    )
    .map_err(|e| e.to_string())?;
    inspect(&text)
}

#[test]
#[ignore = "requires the qualified CPPGM native backend"]
fn inline_values_execute_through_checked_native_lowering() -> Result<(), Box<dyn std::error::Error>>
{
    native::check_cases(
        "parallel/pa25-inline-const/native",
        &[
            (
                "context",
                "fn main(){let x=const{7};let _:u16=x;check(x==7);}",
                false,
            ),
            (
                "product-repeat",
                "struct Value(u8);fn main(){let a=[const{Value(7)};2];check(a[0].0==7);check(a[1].0==7);}",
                false,
            ),
            (
                "early-late",
                "const fn value()->usize{const{3}}type A=[u8;value()];fn main(){let _:A=[0;3];check(value()==3);check(value()==3);}",
                false,
            ),
            (
                "zero",
                "const fn divide(n:u8)->u8{1/n}fn main(){let a:[u8;0]=[const{const{divide(0)}};0];let _:[u8;0]=a;check(true);}",
                false,
            ),
            (
                "local-state",
                "fn main(){let value=const{let mut n=0u8;while n<3{n=n+1;}let p=&n;*p};check(value==3);}",
                false,
            ),
            (
                "closure-pointer",
                "fn main(){let f:fn()->u8=const{||7};check(f()==7);}",
                false,
            ),
            (
                "static-reference",
                "fn f()->&'static u8{const{&7}}fn main(){check(*f()==7);}",
                false,
            ),
            (
                "direct-root-context",
                "fn main(){let f=||const{let mut n=0;while n<7{n+=1;}n};let x:u16=f();check(x==7);}",
                false,
            ),
            (
                "direct-root-reference",
                "fn main(){let f:fn()->&'static u8=||const{&7u8};check(*f()==7);}",
                false,
            ),
            (
                "atomic-zero",
                "use core::sync::atomic::AtomicU32;fn f()->&'static [AtomicU32;0]{&[const{AtomicU32::new(0)};0]}fn main(){let _:&'static [AtomicU32;0]=f();check(true);}",
                false,
            ),
        ],
    )
}
