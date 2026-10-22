//! Enum values resume checked const bodies without publishing provisional tags.
#[path = "harness/checked_sources.rs"]
mod checked_sources;
#[path = "harness/native_references.rs"]
mod native;
#[path = "discriminant_execution/reached.rs"]
mod reached;
#[path = "helpers/resolve.rs"]
mod source;
use std::{path::Path, process::Command};

const CASES: &[(&str, &str, bool)] = &[
    (
        "call",
        "const fn f()->isize{3}enum E{A=f(),B}fn main(){check(E::A as isize==3&&E::B as isize==4);}",
        false,
    ),
    (
        "forward-call",
        "enum E{A=f(),B}const fn f()->isize{3}fn main(){check(E::A as isize==3&&E::B as isize==4);}",
        false,
    ),
    (
        "reused-function",
        "const fn f(n:isize)->isize{2*n+1}enum E{A=f(1),B=f(2),C}fn main(){check(E::A as isize==3&&E::B as isize==5&&E::C as isize==6);}",
        false,
    ),
    (
        "loop",
        "const fn f()->isize{let mut i=0;while i<3{i+=1;}i}enum E{A=f(),B}fn main(){check(E::A as isize==3&&E::B as isize==4);}",
        false,
    ),
    (
        "generic",
        "const fn f<const N:isize>()->isize{N}enum E{A=f::<3>(),B}fn main(){check(E::A as isize==3&&E::B as isize==4);}",
        false,
    ),
    (
        "callee-pattern",
        "const N:isize=7;const fn f()->isize{match 7isize{N=>3,_=>4}}enum E{A=f(),B}fn main(){check(E::A as isize==3&&E::B as isize==4);}",
        false,
    ),
    (
        "forward-callee-pattern",
        "enum E{A=f(),B}const fn f()->isize{match 7isize{N=>3,_=>4}}const N:isize=7;fn main(){check(E::A as isize==3&&E::B as isize==4);}",
        false,
    ),
    (
        "completed-cross-enum",
        "enum First{A=3,B}enum E{A=f(),B}const fn f()->isize{First::A as isize}fn main(){check(E::A as isize==3&&E::B as isize==4&&First::B as isize==4);}",
        false,
    ),
];
#[test]
fn const_discriminants_resume_the_same_checked_function() -> Result<(), String> {
    for &(name, text, _) in CASES {
        let report = source::resolve(&format!("{}{text}", native::CHECK), true)?;
        rustgm::pipeline::analyze_source(&report).map_err(|e| format!("{name}: {e}"))?;
    }
    for width in [8, 32, 128] {
        use std::fmt::Write;
        let mut text = String::from("const fn f(n:isize)->isize{n}enum E{");
        for index in 0..width {
            write!(text, "V{index}=f({index}),").map_err(|e| e.to_string())?;
        }
        text.push('}');
        let report = source::resolve(&text, true)?;
        let facts = rustgm::pipeline::analyze_source(&report)
            .map_err(|e| e.to_string())?
            .facts;
        assert_eq!(facts.aggregates.discriminant_work(), (width, width));
        assert_eq!(facts.body_work.cfg_functions.get(), 1);
        assert_eq!(facts.constant_work.expressions, width);
        assert_eq!(facts.constant_work.discriminant_visits, width + 1);
        let mut distinct = String::new();
        for index in 0..width {
            write!(distinct, "const fn f{index}()->isize{{{index}}}").map_err(|e| e.to_string())?;
        }
        distinct.push_str("enum E{");
        for index in 0..width {
            write!(distinct, "V{index}=f{index}(),").map_err(|e| e.to_string())?;
        }
        distinct.push('}');
        let report = source::resolve(&distinct, true)?;
        let facts = rustgm::pipeline::analyze_source(&report)
            .map_err(|e| e.to_string())?
            .facts;
        assert_eq!(facts.aggregates.discriminant_work(), (width, width));
        assert_eq!(facts.body_work.cfg_functions.get(), width);
        assert_eq!(facts.constant_work.expressions, width);
        assert_eq!(facts.constant_work.discriminant_visits, 2 * width);
    }
    Ok(())
}
#[test]
#[ignore = "requires the configured CPPGM native backend"]
fn completed_discriminants_match_independent_native_values()
-> Result<(), Box<dyn std::error::Error>> {
    native::check_cases("parallel/pa25-discriminant-values", CASES)
}
fn qualify(name: &str, text: &str, accepted: bool) -> Result<String, Box<dyn std::error::Error>> {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../obj/parallel/pa25-discriminant-outcomes/{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let input = folder.join(format!("{name}.rs"));
    std::fs::write(&input, text)?;
    let independent = Command::new("rustc")
        .args([
            "+1.98.0",
            "--edition=2024",
            "--crate-name=discriminant_outcome",
            "--emit=obj",
            "-o",
        ])
        .arg(folder.join(format!("{name}.o")))
        .arg(&input)
        .output()?;
    std::fs::write(
        folder.join(format!("{name}.rustc.stderr")),
        &independent.stderr,
    )?;
    assert_eq!(
        independent.status.code(),
        Some(i32::from(!accepted)),
        "{name}: {}",
        String::from_utf8_lossy(&independent.stderr)
    );
    let candidate = Command::new(env!("CARGO_BIN_EXE_rustgm"))
        .arg("--emit-lowir")
        .arg(&input)
        .output()?;
    std::fs::write(
        folder.join(format!("{name}.candidate.stderr")),
        &candidate.stderr,
    )?;
    assert_eq!(candidate.status.code(), Some(1), "{name}");
    Ok(String::from_utf8(candidate.stderr)?)
}
#[test]
fn true_cycles_and_duplicate_tags_are_rejected() -> Result<(), Box<dyn std::error::Error>> {
    for (name, text, accepted, message) in [
        (
            "constant-cycle",
            "const N:isize=E::A as isize;enum E{A=N}fn main(){}",
            false,
            "cycle in enum discriminant or constant dependencies",
        ),
        (
            "callee-cycle",
            "enum E{A=f()}const fn f()->isize{E::A as isize}fn main(){}",
            false,
            "cycle in enum discriminant or constant dependencies",
        ),
        (
            "duplicate",
            "const fn f()->isize{3}enum E{A=f(),B=f()}fn main(){}",
            false,
            "duplicate enum discriminant",
        ),
    ] {
        let error = qualify(name, text, accepted)?;
        assert!(error.contains(message), "{name}: {error}");
        assert!(
            !error.contains("constant execution body is not ready"),
            "{name}"
        );
    }
    Ok(())
}
