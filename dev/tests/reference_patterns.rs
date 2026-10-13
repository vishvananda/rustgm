#[path = "support/semantic.rs"]
mod semantic;
use rustgm::sema::{self, patterns::Kind};
use semantic::analyze;
use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
};
const ACCEPTED: &[(&str, &str)] = &[
    ("scalar", "const C:&u8=&7;fn f(x:&u8){match x{C=>(),_=>()}}"),
    ("unit", "const C:&()=&();fn f(C:&()){}"),
    (
        "tuple",
        "const C:&(u8,bool)=&(7,true);fn f(x:&(u8,bool)){match x{C=>(),_=>()}}",
    ),
    (
        "array",
        "const C:&[u8;2]=&[2,7];fn f(x:&[u8;2]){match x{C=>(),_=>()}}",
    ),
    (
        "slice",
        "const C:&[u8]=&[2,7];fn f(x:&[u8]){match x{C=>(),_=>()}}",
    ),
    (
        "nested",
        "const C:&&u8=&&7;fn f(x:&&u8){match x{C=>(),_=>()}}",
    ),
    (
        "nominal",
        "#[derive(PartialEq)]struct S(u8);const C:&S=&S(7);fn f(x:&S){match x{C=>(),_=>()}}",
    ),
    (
        "static-scalar",
        "static S:u8=7;const C:&u8=&S;fn f(x:&u8){match x{C=>(),_=>()}}",
    ),
    (
        "static-noncopy",
        "#[derive(PartialEq)]struct S(u8);static V:S=S(7);const C:&S=&V;fn f(x:&S){match x{C=>(),_=>()}}",
    ),
    (
        "deref",
        "const P:&u8=&7;const C:u8=*P;fn f(x:u8){match x{C=>(),_=>()}}",
    ),
    (
        "index",
        "const P:&[u8]=&[2,7];const C:u8=P[1];fn f(x:u8){match x{C=>(),_=>()}}",
    ),
    (
        "projection",
        "const P:&(u8,bool)=&(7,true);const C:u8=P.0;fn f(x:u8){match x{C=>(),_=>()}}",
    ),
    (
        "static-field",
        "struct S(u8);static V:S=S(7);const C:&u8=&V.0;fn f(x:&u8){match x{C=>(),_=>()}}",
    ),
    (
        "static-read-field",
        "struct S(u8);static V:S=S(7);const C:u8=V.0;fn f(x:u8){match x{C=>(),_=>()}}",
    ),
    (
        "nested-tuple",
        "const C:(&u8,bool)=(&7,true);fn f(x:(&u8,bool)){match x{C=>(),_=>()}}",
    ),
    (
        "reference-repeat",
        "const C:[&u8;64]=[&7;64];fn f(x:[&u8;64]){match x{C=>(),_=>()}}",
    ),
    (
        "borrow-index",
        "const C:&u8=&(&[2u8,7])[1];fn f(x:&u8){match x{C=>(),_=>()}}",
    ),
    (
        "signed-slice",
        "const C:&[i16]=&[-2,7];fn f(x:&[i16]){match x{C=>(),_=>()}}",
    ),
    (
        "slice-empty",
        "const C:&[()]=&[];fn f(x:&[()]){match x{C=>(),[_,..]=>()}}",
    ),
    (
        "double-string",
        "const C:&&str=&\"a\";fn f(x:&&str){match x{C=>(),_=>()}}",
    ),
    (
        "double-bytes",
        "const C:&&[u8;2]=&b\"ab\";fn f(x:&&[u8;2]){match x{C=>(),_=>()}}",
    ),
    (
        "byte-copy",
        "const C:[u8;2]=*b\"ab\";fn f(x:[u8;2]){match x{C=>(),_=>()}}",
    ),
    (
        "byte-index",
        "const C:u8=b\"ab\"[1];fn f(x:u8){match x{C=>(),_=>()}}",
    ),
    (
        "byte-borrow-index",
        "const C:&u8=&b\"ab\"[1];fn f(x:&u8){match x{C=>(),_=>()}}",
    ),
    (
        "copy-reborrow",
        "const P:&[u8;2]=&[2,7];const A:[u8;2]=*P;const C:&[u8]=&A;fn f(x:&[u8]){match x{C=>(),_=>()}}",
    ),
    (
        "packed-byte",
        "#[repr(packed)]struct S(u8,u64);const C:&u8=&S(2,7).0;fn f(x:&u8){match x{C=>(),_=>()}}",
    ),
    (
        "packed-two",
        "#[repr(packed(2))]struct S(u8,u16);const C:&u16=&S(2,7).1;fn f(x:&u16){match x{C=>(),_=>()}}",
    ),
    (
        "packed-read",
        "#[repr(packed)]struct S(u8,u64);const C:u64=S(2,7).1;fn f(x:u64){match x{C=>(),_=>()}}",
    ),
    (
        "autoderef",
        "struct S(u8);const P:&&S=&&S(7);const C:u8=P.0;fn f(x:u8){match x{C=>(),_=>()}}",
    ),
    (
        "reborrow",
        "const P:&u8=&7;const C:&u8=&*P;fn f(x:&u8){match x{C=>(),_=>()}}",
    ),
    (
        "struct-field-ref",
        "#[derive(PartialEq)]struct N(u8);struct S(N);const C:&N=&S(N(7)).0;fn f(x:&N){match x{C=>(),_=>()}}",
    ),
    (
        "nominal-shared-static",
        "#[derive(PartialEq)]struct S(&'static u8);static V:S=S(&7);const C:&S=&V;fn f(x:&S){match x{C=>(),_=>()}}",
    ),
    (
        "lazy",
        "const C:&u8=if true{&7}else{&(1/0)};fn f(x:&u8){match x{C=>(),_=>()}}",
    ),
    (
        "macro",
        "macro_rules! val{($e:expr)=>{&$e}}const C:&u8=val!(7);fn f(x:&u8){match x{C=>(),_=>()}}",
    ),
];
const REJECTED: &[(&str, &str)] = &[
    (
        "mut-input",
        "const C:&u8=&7;fn f(x:&mut u8){match x{C=>(),_=>()}}",
    ),
    (
        "wrong-depth",
        "const C:&&u8=&&7;fn f(x:&u8){match x{C=>(),_=>()}}",
    ),
    (
        "named-no-unsize",
        "const C:&[u8;2]=&[2,7];fn f(x:&[u8]){match x{C=>(),_=>()}}",
    ),
    (
        "no-equality",
        "struct S(u8);const C:&S=&S(7);fn f(x:&S){match x{C=>(),_=>()}}",
    ),
    (
        "nan",
        "const C:&f32=&(0.0/0.0);fn f(x:&f32){match x{C=>(),_=>()}}",
    ),
    (
        "static-direct",
        "static C:u8=7;fn f(x:u8){match x{C=>(),_=>()}}",
    ),
    ("refutable", "const C:&u8=&7;fn f(C:&u8){}"),
    (
        "finite",
        "const C:&bool=&true;fn f(x:&bool){match x{C=>()}}",
    ),
    (
        "bounds",
        "const C:&u8=&[2u8,7][2];fn f(x:&u8){match x{C=>(),_=>()}}",
    ),
    (
        "byte-bounds",
        "const C:&u8=&b\"ab\"[2];fn f(x:&u8){match x{C=>(),_=>()}}",
    ),
    (
        "byte-read-bounds",
        "const C:u8=b\"ab\"[2];fn f(x:u8){match x{C=>(),_=>()}}",
    ),
    (
        "packed-field",
        "#[repr(packed)]struct S(u8,u64);const C:&u64=&S(2,7).1;fn f(x:&u64){match x{C=>(),_=>()}}",
    ),
    (
        "packed-nested",
        "struct N(u64);#[repr(packed)]struct S(u8,N);const C:&u64=&S(2,N(7)).1.0;fn f(x:&u64){match x{C=>(),_=>()}}",
    ),
    (
        "move-shared",
        "#[derive(PartialEq)]struct S(u8);const P:&S=&S(7);const C:S=*P;fn f(x:S){match x{C=>(),_=>()}}",
    ),
    (
        "move-shared-field",
        "#[derive(PartialEq)]struct N(u8);struct S(N);const P:&S=&S(N(7));const C:N=P.0;fn f(x:N){match x{C=>(),_=>()}}",
    ),
    (
        "static-cache-move",
        "#[derive(PartialEq)]struct S(u8);static V:S=S(7);const A:&S=&V;const C:S=V;fn f(x:&S,y:S){match x{A=>(),_=>()}match y{C=>(),_=>()}}",
    ),
    (
        "mut-unit",
        "const C:&mut ()=&mut ();fn f(x:&mut ()){match x{C=>()}}",
    ),
    (
        "mut-empty",
        "const C:&mut [u8;0]=&mut [];fn f(x:&mut [u8;0]){match x{C=>()}}",
    ),
    (
        "mut-scalar",
        "const C:&mut u8=&mut 7;fn f(x:&mut u8){match x{C=>(),_=>()}}",
    ),
];
fn folder() -> Result<PathBuf, std::io::Error> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../obj")
        .join(format!("reference-patterns-{}", std::process::id()));
    std::fs::create_dir_all(&path)?;
    Ok(path)
}
fn frontend(path: &Path, mode: &str) -> Result<Output, std::io::Error> {
    Command::new(env!("CARGO_BIN_EXE_rustgm"))
        .arg(mode)
        .arg(path)
        .output()
}
fn rustc(path: &Path) -> Result<Output, std::io::Error> {
    Command::new("rustc")
        .args([
            "--edition=2024",
            "--crate-type=lib",
            "--crate-name",
            "reference_patterns",
            "--emit=metadata",
        ])
        .arg(path)
        .arg("-o")
        .arg(path.with_extension("rmeta"))
        .output()
}
#[test]
fn declared_reference_patterns_agree_with_rustc() -> Result<(), Box<dyn std::error::Error>> {
    let directory = folder()?;
    for (cases, expected) in [(ACCEPTED, 0), (REJECTED, 1)] {
        for &(name, source) in cases {
            let path = directory.join(format!("{expected}-{name}.rs"));
            std::fs::write(&path, source)?;
            let independent = rustc(&path)?;
            std::fs::write(path.with_extension("rustc-stderr"), &independent.stderr)?;
            assert_eq!(
                independent.status.code(),
                Some(expected),
                "{name}: {}",
                String::from_utf8_lossy(&independent.stderr)
            );
            let emitted = frontend(&path, "--emit-patterns")?;
            let counted = frontend(&path, "--count-patterns")?;
            std::fs::write(path.with_extension("patterns"), &emitted.stdout)?;
            std::fs::write(path.with_extension("patterns-stderr"), &emitted.stderr)?;
            for output in [&emitted, &counted] {
                assert_eq!(
                    output.status.code(),
                    Some(expected),
                    "{name}: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            assert_eq!(emitted.stderr, counted.stderr, "{name}");
            if expected == 0 {
                assert!(emitted.stderr.is_empty());
                let text = std::str::from_utf8(&emitted.stdout)?;
                let footer = text
                    .lines()
                    .last()
                    .and_then(|s| s.strip_prefix("eof "))
                    .ok_or("missing footer")?;
                assert_eq!(format!("{footer}\n").as_bytes(), counted.stdout);
                let repeated = frontend(&path, "--emit-patterns")?;
                assert!(repeated.status.success());
                assert_eq!(repeated.stdout, emitted.stdout);
            } else {
                assert!(
                    emitted.stdout.is_empty()
                        && counted.stdout.is_empty()
                        && !emitted.stderr.is_empty()
                );
            }
        }
    }
    Ok(())
}
#[test]
fn references_keep_sparse_projection_and_shared_pattern_facts() -> Result<(), String> {
    let (_, facts) =
        analyze("struct S(u8);const P:&&S=&&S(7);const C:u8=P.0;fn f(x:u8){match x{C=>(),_=>()}}")?;
    assert_eq!(facts.autoderefs.len(), 1);
    assert_eq!(facts.autoderefs.values().copied().collect::<Vec<_>>(), [2]);
    let (_, facts) = analyze("const C:[&u8;64]=[&7;64];fn f(x:[&u8;64]){match x{C=>(),_=>()}}")?;
    assert_eq!(facts.patterns.fields.len(), 64);
    assert!(
        facts
            .patterns
            .fields
            .iter()
            .all(|f| f.pattern == facts.patterns.fields[0].pattern)
    );
    assert_eq!(
        facts
            .patterns
            .nodes
            .iter()
            .filter(|p| matches!(p.kind, Kind::Dereference { .. }))
            .count(),
        1
    );
    assert!(facts.patterns.nodes.len() < 10);
    assert_eq!(std::mem::size_of::<sema::model::NodeFacts>(), 80);
    Ok(())
}
#[test]
fn static_value_cache_cannot_bypass_noncopy_reads() -> Result<(), String> {
    let source = "#[derive(PartialEq)]struct S(u8);static V:S=S(7);const A:&S=&V;const C:S=V;fn f(x:&S,y:S){match x{A=>(),_=>()}match y{C=>(),_=>()}}";
    let Err(error) = analyze(source) else {
        return Err("moved a non-Copy cached static".into());
    };
    assert!(
        error.contains("cannot move a non-Copy constant value out of static storage"),
        "{error}"
    );
    Ok(())
}
#[test]
fn cyclic_static_pattern_queries_diagnose_instead_of_recursing()
-> Result<(), Box<dyn std::error::Error>> {
    // Pinned rustc 1.98.0 SIGSEGVs for this input. This is a RustGM bounded-query
    // control, not a claim of language-rejection equivalence or a rustc test.
    let directory = folder()?;
    let path = directory.join("cycle.rs");
    std::fs::write(
        &path,
        "#[derive(PartialEq)]struct N{next:&'static N}static S:N=N{next:&S};const C:&N=&S;fn f(x:&N){match x{C=>(),_=>()}}",
    )?;
    for mode in ["--emit-patterns", "--count-patterns"] {
        let output = frontend(&path, mode)?;
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert!(
            String::from_utf8_lossy(&output.stderr)
                .contains("cycle in enum discriminant or constant dependencies")
        );
    }
    Ok(())
}
#[test]
fn reference_operations_keep_executable_proofs_pending() -> Result<(), Box<dyn std::error::Error>> {
    let directory = folder()?;
    for (index, source) in [
        "fn f(x:u8){let _=&x;}",
        "fn f(x:&u8)->u8{*x}",
        "struct S(u8);fn f(x:&S)->u8{x.0}",
        "fn f(x:&[u8])->u8{x[0]}",
        "const C:&u8=&7;fn f(x:&u8){match x{C=>(),_=>()}}",
    ]
    .into_iter()
    .enumerate()
    {
        let path = directory.join(format!("proof{index}.rs"));
        std::fs::write(&path, source)?;
        let selected = frontend(&path, "--emit-patterns")?;
        assert!(
            selected.status.success(),
            "{}",
            String::from_utf8_lossy(&selected.stderr)
        );
        let output = frontend(&path, "--emit-lowir")?;
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        let text = String::from_utf8_lossy(&output.stderr);
        assert!(
            text.contains("region and loan proofs") || text.contains("owned selected patterns"),
            "{text}"
        );
    }
    Ok(())
}
#[test]
fn reference_projection_values_are_selected_from_the_right_place() -> Result<(), String> {
    for (source, expected) in [
        (
            "const C:&u8=&(&[2u8,7])[1];fn f(x:&u8){match x{C=>(),_=>()}}",
            7,
        ),
        (
            "struct N(u8);struct S(N,u8);static V:S=S(N(2),7);const C:u8=V.1;fn f(x:u8){match x{C=>(),_=>()}}",
            7,
        ),
        ("const C:u8=b\"ab\"[1];fn f(x:u8){match x{C=>(),_=>()}}", 98),
        (
            "const C:&u8=if false{&2}else{&7};fn f(x:&u8){match x{C=>(),_=>()}}",
            7,
        ),
        (
            "#[repr(packed)]struct S(&'static u64);const C:&u64=&*S(&7).0;fn f(x:&u64){match x{C=>(),_=>()}}",
            7,
        ),
    ] {
        let (_, facts) = analyze(source)?;
        assert_eq!(facts.patterns.ranges.len(), 1, "{source}");
        assert_eq!(facts.patterns.ranges[0].lower, expected, "{source}");
        assert_eq!(facts.patterns.ranges[0].upper, expected, "{source}");
    }
    Ok(())
}
#[test]
fn reference_values_preserve_cross_file_use_sites() -> Result<(), Box<dyn std::error::Error>> {
    let directory = folder()?.join("package");
    std::fs::create_dir_all(&directory)?;
    std::fs::write(
        directory.join("values.rs"),
        "#[derive(PartialEq)]pub struct S{value:u8}static V:S=S{value:7};pub const C:&S=&V;",
    )?;
    let path = directory.join("lib.rs");
    std::fs::write(
        &path,
        "mod values;use values::C;fn f(x:&values::S){match x{C=>(),_=>()}}",
    )?;
    let independent = rustc(&path)?;
    assert!(
        independent.status.success(),
        "{}",
        String::from_utf8_lossy(&independent.stderr)
    );
    let output = frontend(&path, "--emit-patterns")?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let view = std::str::from_utf8(&output.stdout)?;
    assert!(view.contains("ranks=7:7"));
    assert!(
        view.lines()
            .filter(|s| s.starts_with("pattern "))
            .all(|s| s.contains("source=0:"))
    );
    Ok(())
}
#[test]
fn reference_queries_and_destruction_use_flat_worklists() -> Result<(), Box<dyn std::error::Error>>
{
    use std::fmt::Write;
    let mut source = String::from("type R0=u8;const C0:R0=7;");
    for depth in 1..=2048 {
        let previous = depth - 1;
        write!(
            source,
            "type R{depth}=&'static R{previous};const C{depth}:R{depth}=&C{previous};"
        )?;
    }
    source.push_str("fn f(x:R2048){match x{C2048=>(),_=>()}}");
    let (_, facts) = analyze(&source)?;
    assert_eq!(
        facts
            .patterns
            .nodes
            .iter()
            .filter(|p| matches!(p.kind, Kind::Dereference { .. }))
            .count(),
        2048
    );
    assert_eq!(facts.patterns.ranges.len(), 1);
    assert_eq!(facts.patterns.ranges[0].lower, 7);
    assert!(facts.patterns.nodes.len() < 2060);
    Ok(())
}

#[test]
fn root_place_validity_distinguishes_borrowed_fields_from_owned_values()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = folder()?;
    let prefix = "enum V{}enum E{A(V),B}struct S(E);";
    for (index, (source, accepted)) in [
        ("fn f(x:&V){match *x{}}", true),
        ("fn f(x:&V){match *x{_=>()}}", true),
        ("fn f(x:V){match x{_=>()}}", true),
        ("fn f(x:&E){match *x{E::A(_)=>(),E::B=>()}}", true),
        ("fn f(x:S){match x.0{E::B=>()}}", true),
        ("fn f(x:&E){match *x{E::B=>()}}", false),
        ("fn f(x:&E){match (*x){E::B=>()}}", false),
        ("fn f(x:&S){match x.0{E::B=>()}}", false),
        ("fn f(x:&[E;2]){match x[0]{E::B=>()}}", false),
        ("fn f(x:&(V,)){match *x{}}", false),
        ("struct T(V);fn f(x:&T){match *x{}}", false),
        ("fn f(x:&E){let E::B=*x;}", false),
        ("fn f(x:&E){let E::B=(*x);}", false),
        ("fn f(x:&E){let &E::B=x;}", false),
        (
            "const C:u8=7;fn f(x:&E,y:u8){let E::B=*x;match y{C=>(),_=>()}}",
            false,
        ),
        (
            "const C:u8=7;fn f(x:&E,y:u8){match y{C=>(),_=>()}match *x{E::B=>()}}",
            false,
        ),
        (
            "macro_rules! m{($x:expr)=>{match $x{E::B=>()}}}fn f(x:&E){m!(*x)}",
            false,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let source = format!("{prefix}{source}");
        let path = directory.join(format!("root{index}.rs"));
        std::fs::write(&path, &source)?;
        for (label, result) in [
            ("rustc", rustc(&path)?),
            ("rustgm", frontend(&path, "--emit-patterns")?),
        ] {
            assert_eq!(
                result.status.code(),
                Some(i32::from(!accepted)),
                "{label}: {source}: {}",
                String::from_utf8_lossy(&result.stderr)
            );
            if label == "rustgm" && !accepted {
                assert!(result.stdout.is_empty());
                let message = String::from_utf8_lossy(&result.stderr);
                assert!(message.contains("non-exhaustive") || message.contains("refutable"));
            }
        }
    }
    for (parameter, value, useful) in [("&E", "*x", true), ("E", "x", false)] {
        let (_, facts) = analyze(&format!(
            "{prefix}fn f(x:{parameter}){{match {value}{{E::A(_)=>(),E::B=>()}}}}"
        ))?;
        assert_eq!(facts.patterns.arms[0].useful, useful);
    }
    for (parameter, value, useful) in [("&V", "*x", true), ("V", "x", false)] {
        let (_, facts) = analyze(&format!(
            "{prefix}fn f(x:{parameter}){{match {value}{{_=>()}}}}"
        ))?;
        assert_eq!(facts.patterns.arms[0].useful, useful);
        assert!(facts.patterns.decisions[0].uninhabited);
    }
    // A block produces an owned value. Inspection does not prove that its
    // underlying shared-place read is legal; executable lowering remains gated.
    let (_, facts) = analyze(&format!("{prefix}fn f(x:&E){{match {{*x}}{{E::B=>()}}}}"))?;
    assert!(!facts.patterns.decisions[0].uninhabited);
    Ok(())
}
