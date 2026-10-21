//! Conditional child proofs retain exact owner identity and all return obligations.
use super::{analyze, inspect, native};
use std::fmt::Write as _;
#[path = "../helpers/qualify.rs"]
mod rustc;
const CASES: &[(&str, bool)] = &[
    ("borrowed-outer", true),
    ("owned-outer", false),
    ("mutable-receiver-escape", false),
    ("borrowed-mutable-shared", true),
    ("once-reference", true),
    ("once-aggregate", true),
    ("ordinary-returned-field", true),
    ("joined-borrows", true),
    ("escape-outer", false),
    ("inferred-inline", true),
    ("inline-local-escape", false),
    ("explicit-return", true),
    ("branch-return", true),
    ("diverging-root", true),
    ("explicit-owned-return", false),
    ("mixed-instances", true),
    ("explicit-inline-return", true),
    ("repeated-slots", true),
    ("once-cleanup", true),
];
fn fixture(name: &str) -> Result<String, std::io::Error> {
    if name == "ordinary-returned-field" {
        return std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../pa16/tests/spec/thin-native/returned-lifetime-field.rs"),
        );
    }
    let folder = if name.contains("inline") {
        "pa25/tests/spec/inline-output"
    } else {
        "pa24/tests/spec/output-interfaces"
    };
    std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../{folder}/{name}.rs")),
    )
}
#[test]
fn independent_output_outcomes() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, accepted) in CASES {
        rustc::qualify(
            &format!("parallel/pa25-nested-output/qualification/{name}"),
            &fixture(name)?,
            accepted,
        )?;
    }
    Ok(())
}
#[test]
fn exact_output_and_escape_outcomes() -> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for &(name, accepted) in CASES {
        if inspect(&fixture(name)?).is_ok() != accepted {
            failures.push(name);
        }
    }
    assert!(failures.is_empty(), "{failures:?}");
    Ok(())
}
#[test]
#[ignore = "requires the qualified CPPGM native backend"]
fn output_native_values() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, accepted) in CASES {
        if !accepted {
            continue;
        }
        let source = fixture(name)?;
        let source = if matches!(name, "once-cleanup" | "ordinary-returned-field") {
            source
                .strip_prefix(native::CHECK)
                .unwrap_or(&source)
                .to_owned()
        } else {
            let check = match name {
                "branch-return" | "joined-borrows" => "run(true)==7&&run(false)==9",
                "mixed-instances" => "run()==9",
                "repeated-slots" => "run()==14",
                _ => "run()==7",
            };
            format!("{source}fn main(){{check({check});}}")
        };
        native::check_cases(
            "parallel/pa25-nested-output/native",
            &[(name, &source, false)],
        )?;
    }
    Ok(())
}
#[test]
fn output_ports_and_roots_are_unique_under_repeated_results() -> Result<(), String> {
    for count in [8, 32, 128] {
        let mut source = String::new();
        for index in 0..count {
            write!(source, "pub fn f{index}()->u8{{let x=7u8;let c=||&x;*c()}}")
                .map_err(|e| e.to_string())?;
        }
        let (_, analysis) = analyze(&source)?;
        let outputs = analysis
            .facts
            .nested_outputs
            .as_deref()
            .ok_or("missing output registry")?;
        assert_eq!(outputs.ports, count);
        assert!(outputs.bridges <= 3 * count);
        assert!(outputs.retained_bytes <= 1024 * count);
        inspect(&source)?;
    }
    Ok(())
}

#[test]
fn named_and_runtime_zero_repeats_have_distinct_operand_work() -> Result<(), String> {
    use rustgm::body::ir::{Action, Rvalue};
    let text = "const ITEM:u8=7;pub fn empty()->[u8;0]{[ITEM;0]}pub fn runtime(n:&mut u8)->[u8;0]{[{*n+=1;*n};0]}";
    let (report, analysis) = analyze(text)?;
    let program =
        rustgm::body::build_concrete(&report, &analysis.facts).map_err(|e| e.to_string())?;
    assert!(
        !program.bodies[0]
            .values
            .iter()
            .any(|value| matches!(value.kind, Rvalue::Constant(_)))
    );
    assert!(
        program.bodies[1]
            .statements
            .iter()
            .any(|statement| matches!(statement.action, Action::Store { .. }))
    );
    inspect(text)?;
    assert!(inspect("const ITEM:u8=1/0;pub fn empty()->[u8;0]{[ITEM;0]}").is_err());
    Ok(())
}
