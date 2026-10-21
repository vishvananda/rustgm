//! Inspection consumes the checked entry-free pipeline and preserves symbolic backing.
#[path = "harness/native_references.rs"]
mod native;
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "helpers/resolve.rs"]
mod source;
use rustgm::{
    lower::{
        constant_view::{self, Limits},
        instances,
    },
    pipeline,
};

use std::fmt::Write;

fn inspect(text: &str, limits: Limits) -> Result<(String, String), String> {
    let report = source::resolve(text, true)?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report).map_err(|e| e.to_string())?;
    let inventory = instances::inspect_prepared(&report, &mut facts, prepared, |facts, checked| {
        constant_view::inspect_with_limits(facts, checked, limits)
    })
    .map_err(|e| e.to_string())?
    .map_err(str::to_owned)?;
    let mut rendered = Vec::new();
    let mut counted = Vec::new();
    inventory.write(&mut rendered).map_err(|e| e.to_string())?;
    inventory.count(&mut counted).map_err(|e| e.to_string())?;
    Ok((
        String::from_utf8(rendered).map_err(|e| e.to_string())?,
        String::from_utf8(counted).map_err(|e| e.to_string())?,
    ))
}
fn fixture(name: &str) -> std::io::Result<String> {
    std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../pa25/tests/spec/inspection/{name}.rs")),
    )
}
#[test]
fn entry_free_roots_and_shared_static_identity() -> Result<(), Box<dyn std::error::Error>> {
    let library = fixture("library")?;
    rustc::qualify("parallel/pa25-constant-inspection/library", &library, true)?;
    let (text, counts) = inspect(&library, Limits::default())?;
    assert!(text.starts_with("constants 1\n"));
    assert!(text.ends_with(&format!("eof {counts}")));
    assert_eq!(text.matches("origin=static:").count(), 1);
    assert_eq!(text.matches("root initializer").count(), 4);
    assert!(text.contains("function type="));
    assert!(text.contains("const=true status=checked-body"));
    assert_eq!(inspect(&library, Limits::default())?.0, text);
    Ok(())
}
#[test]
fn unused_invalid_source_still_rejects_without_entry() -> Result<(), Box<dyn std::error::Error>> {
    for (name, text, reason) in [
        (
            "invalid-unused",
            fixture("unused-division")?,
            "divides by zero",
        ),
        (
            "unused-body",
            fixture("unused-uninitialized")?,
            "uninitialized",
        ),
    ] {
        rustc::qualify(
            &format!("parallel/pa25-constant-inspection/{name}"),
            &text,
            false,
        )?;
        let error = inspect(&text, Limits::default())
            .err()
            .ok_or("unexpected success")?;
        assert!(error.contains(reason), "{error}");
    }
    Ok(())
}
#[test]
fn repeats_keep_compression_and_discard_zero_fill() -> Result<(), Box<dyn std::error::Error>> {
    let text = fixture("repeats")?;
    rustc::qualify("parallel/pa25-constant-inspection/repeats", &text, true)?;
    let (view, _) = inspect(&text, Limits::default())?;
    assert!(view.contains("repeat length=0 element=none"));
    assert!(view.contains("repeat length=1000000 element=v"));
    assert!(!view.contains("allocation a"));
    assert!(view.len() < 2000);
    Ok(())
}
#[test]
fn generic_and_associated_initializers_remain_deferred() -> Result<(), Box<dyn std::error::Error>> {
    let text = "const fn identity<T>(x:T)->T{x}trait T{const C:u8=7;}const VALUE:u8=1;";
    rustc::qualify("parallel/pa25-constant-inspection/deferred", text, true)?;
    let (view, _) = inspect(text, Limits::default())?;
    assert!(view.contains("status=deferred-template"));
    assert!(view.contains("status=deferred-initializer"));
    assert_eq!(view.matches("root initializer").count(), 1);
    Ok(())
}
#[test]
fn each_graph_budget_is_an_error() {
    for (limits, reason) in [
        (
            Limits {
                nodes: 0,
                ..Limits::default()
            },
            "node budget",
        ),
        (
            Limits {
                work: 0,
                ..Limits::default()
            },
            "work budget",
        ),
        (
            Limits {
                bytes: 0,
                ..Limits::default()
            },
            "byte budget",
        ),
    ] {
        let error = inspect("const VALUE:u8=7;", limits).err();
        assert!(error.is_some_and(|error| error.contains(reason)));
    }
}

#[test]
fn exact_bits_literal_storage_and_callable_permissions() -> Result<(), Box<dyn std::error::Error>> {
    let text = "const NEG:i8=-1;const ZERO:f64=-0.0;const NAN:f32=0.0/0.0;const TEXT:&str=\"hi\";const BYTES:&[u8]=b\"ok\";fn add(n:u8)->u8{n+1}const FN:fn(u8)->u8=add;const RAW:*const u8=0 as *const u8;";
    rustc::qualify("parallel/pa25-constant-inspection/bits", text, true)?;
    let (view, _) = inspect(text, Limits::default())?;
    assert!(view.contains("integer bits=000000000000000000000000000000ff"));
    assert!(view.contains("f64 bits=8000000000000000"));
    assert!(view.contains("f32 bits="));
    assert!(view.contains("origin=literal bytes=6869"));
    assert!(view.contains("origin=literal bytes=6f6b"));
    assert!(view.contains("const=false status=checked-body"));
    assert!(view.contains("address bits=0000000000000000"));
    Ok(())
}
#[test]
fn projected_static_aliases_base_and_equal_values_keep_distinct_storage()
-> Result<(), Box<dyn std::error::Error>> {
    let text =
        "static PAIR:(u8,u8)=(7,9);const ITEM:&u8=&PAIR.1;const FIRST:&u8=&7;const SECOND:&u8=&7;";
    rustc::qualify("parallel/pa25-constant-inspection/projection", text, true)?;
    let (view, _) = inspect(text, Limits::default())?;
    assert!(view.contains("origin=projection:"));
    assert!(view.contains(":1 contents=alias"));
    assert_eq!(view.matches("origin=static:").count(), 1);
    assert_eq!(view.matches("origin=allocation contents=").count(), 2);
    Ok(())
}
#[test]
fn inspector_requires_source_initializer_validation() -> Result<(), Box<dyn std::error::Error>> {
    let report = source::resolve("const VALUE:u8=7;", true)?;
    let mut analysis = pipeline::analyze_source(&report)?;
    let error = instances::inspect_prepared(
        &report,
        &mut analysis.facts,
        pipeline::Prepared::default(),
        |_, _| (),
    )
    .err()
    .ok_or("unprepared inspection succeeded")?;
    assert!(error.to_string().contains("requires source preparation"));
    Ok(())
}

#[test]
fn foreign_static_contents_are_external_and_promotions_are_roots()
-> Result<(), Box<dyn std::error::Error>> {
    let text = "unsafe extern \"C\"{static EXTERNAL:u8;}const POINTER:*const u8=unsafe{&raw const EXTERNAL};fn borrow()->&'static u8{&7}";
    rustc::qualify("parallel/pa25-constant-inspection/external", text, true)?;
    let (view, _) = inspect(text, Limits::default())?;
    assert!(view.contains("contents=external"));
    assert!(view.contains("root promotion owner=def:"));
    Ok(())
}
#[test]
fn preparation_and_rendering_do_not_repeat_initializer_execution()
-> Result<(), Box<dyn std::error::Error>> {
    let report = source::resolve(&fixture("library")?, true)?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    let validations = facts.constant_work.initializer_validations;
    let expressions = facts.constant_work.expressions;
    instances::inspect_prepared(&report, &mut facts, prepared, |facts, checked| {
        assert_eq!(facts.constant_work.initializer_validations, validations);
        assert_eq!(facts.constant_work.expressions, expressions);
        let lookups = facts.constant_work.initializer_lookups.get();
        for _ in 0..4 {
            constant_view::inspect(facts, checked).map_err(str::to_owned)?;
        }
        assert_eq!(facts.constant_work.initializer_lookups.get(), lookups);
        Ok::<_, String>(())
    })??;
    Ok(())
}
#[test]
fn cli_emit_and_count_share_the_inventory() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../pa25/tests/spec/inspection/library.rs");
    let emit = std::process::Command::new(env!("CARGO_BIN_EXE_rustgm"))
        .arg("--emit-constants")
        .arg(&path)
        .output()?;
    let count = std::process::Command::new(env!("CARGO_BIN_EXE_rustgm"))
        .arg("--count-constants")
        .arg(&path)
        .output()?;
    assert_eq!(
        emit.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&emit.stderr)
    );
    assert_eq!(
        count.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&count.stderr)
    );
    let emit = String::from_utf8(emit.stdout)?;
    assert!(emit.ends_with(&format!("eof {}", String::from_utf8(count.stdout)?)));
    Ok(())
}

#[test]
fn active_products_and_closed_associated_values() -> Result<(), Box<dyn std::error::Error>> {
    let text = "enum E{A(u8,u8),B{value:u8}}const VALUE:E=E::B{value:7};const UNIT:()=();trait T{const X:u8=7;}impl T for u8{}const SELECTED:u8=<u8 as T>::X;";
    rustc::qualify("parallel/pa25-constant-inspection/products", text, true)?;
    let (view, _) = inspect(text, Limits::default())?;
    assert!(view.lines().any(|row| row.contains("product variant=")
        && !row.contains("variant=none")
        && row.contains("fields=[v")
        && !row.contains(',')));
    assert!(view.contains("product variant=none fields=[]"));
    assert!(view.contains("status=deferred-initializer"));
    assert_eq!(view.matches("root initializer").count(), 3);
    Ok(())
}
#[test]
fn distinct_roots_fit_linear_graph_budgets() -> Result<(), Box<dyn std::error::Error>> {
    for width in [16, 64, 256] {
        let mut source = String::new();
        for i in 0..width {
            write!(source, "const C{i}:u32={i};")?;
        }
        rustc::qualify(
            &format!("parallel/pa25-constant-inspection/scaling-{width}"),
            &source,
            true,
        )?;
        let (_, count) = inspect(
            &source,
            Limits {
                nodes: width,
                work: 16 * width,
                bytes: 2048 * width,
            },
        )?;
        assert_eq!(
            count,
            format!("declarations={width} roots={width} values={width} allocations=0\n")
        );
    }
    Ok(())
}
#[test]
#[ignore = "requires the pinned external LowIR backend"]
fn inspected_values_execute_independently() -> Result<(), Box<dyn std::error::Error>> {
    let values = format!(
        "{}fn main(){{check(NUMBER==7&&*LEFT==7&&*RIGHT==7);}}",
        fixture("library")?
    );
    let pointer = "fn add(x:u8)->u8{x+1}const FN:fn(u8)->u8=add;const ZERO:f64=-0.0;fn inverse(x:f64)->f64{1.0/x}fn main(){check(FN(6)==7&&inverse(ZERO)<0.0);}";
    native::check_cases(
        "parallel/pa25-constant-inspection-values",
        &[
            ("library", &values, false),
            ("bits-and-function", pointer, false),
        ],
    )
}

#[test]
fn owned_literal_bytes_do_not_invent_backing_storage() -> Result<(), Box<dyn std::error::Error>> {
    let text = "const COPY:[u8;2]=*b\"ok\";static STORAGE:[u8;2]=COPY;";
    rustc::qualify("parallel/pa25-constant-inspection/owned-bytes", text, true)?;
    let (view, _) = inspect(text, Limits::default())?;
    assert!(
        view.lines()
            .any(|row| row.starts_with("value v0 ") && row.contains("bytes-inline data=6f6b"))
    );
    assert!(view.contains("origin=static:"));
    Ok(())
}

#[test]
fn reified_closure_constants_keep_code_identity_and_aliases()
-> Result<(), Box<dyn std::error::Error>> {
    let text = fixture("closures")?;
    rustc::qualify("parallel/pa25-constant-inspection/closures", &text, true)?;
    let (view, counts) = inspect(&text, Limits::default())?;
    let callables: Vec<_> = view
        .lines()
        .filter(|line| line.contains("function closure="))
        .collect();
    assert_eq!(callables.len(), 2, "{view}");
    for callable in &callables {
        assert!(callable.contains(" instance="), "{callable}");
        assert!(callable.contains(" signature="), "{callable}");
        assert!(callable.contains(" const=false"), "{callable}");
    }
    assert_eq!(view.matches("root initializer").count(), 3);
    assert!(counts.contains("values=2 allocations=0"), "{counts}");
    assert_eq!(inspect(&text, Limits::default())?.0, view);
    Ok(())
}
