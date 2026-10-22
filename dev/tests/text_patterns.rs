use crate::semantic;
use rustgm::sema::{
    patterns::Kind,
    text::{Id, Texts},
};
use semantic::analyze;
use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
};

const ACCEPTED: &[(&str, &str)] = &[
    ("string", r#"fn f(x:&str){match x{"a"=>(),_=>()}}"#),
    (
        "raw",
        r##"fn f(x:&str){match x{r#"a"#=>(),"\x61"=>(),_=>()}}"##,
    ),
    (
        "unicode",
        r#"fn f(x:&str){match x{"é"=>(),"e\u{301}"=>(),"\0"=>(),""=>(),_=>()}}"#,
    ),
    (
        "constant",
        r#"const C:&str="a";fn f(x:&str){match x{C=>(),_=>()}}"#,
    ),
    (
        "forward",
        r#"const C:&str=D;const D:&str="a";fn f(x:&str){match x{C=>(),D=>(),_=>()}}"#,
    ),
    (
        "lazy",
        r#"const C:&str=if true{"a"}else{"b"};fn f(x:&str){match x{C=>(),_=>()}}"#,
    ),
    (
        "tuple",
        r#"const C:(&str,bool)=("a",true);fn f(x:(&str,bool)){match x{C=>(),("a",false)=>(),_=>()}}"#,
    ),
    (
        "repeat",
        r#"const C:[&str;16]=["a";16];fn f(x:[&str;16]){match x{C=>(),_=>()}}"#,
    ),
    (
        "struct",
        r#"#[derive(PartialEq)]struct S(&'static [u8;1]);const C:S=S(b"a");fn f(x:S){match x{C=>(),S(b"b")=>(),_=>()}}"#,
    ),
    (
        "enum",
        r#"#[derive(PartialEq)]enum E{A(&'static [u8;1]),B}const C:E=E::A(b"a");fn f(x:E){match x{C=>(),E::A(_)=>(),E::B=>()}}"#,
    ),
    (
        "guard",
        r#"fn f(x:&str,b:bool){match x{"a"|"a" if b=>(),"a"=>(),_=>()}}"#,
    ),
    (
        "conditional",
        r#"fn f(x:&str){if let "a"=x{}let "b"=x else{return;};}"#,
    ),
    (
        "macro",
        r#"macro_rules! m{($x:expr,$p:pat)=>{match $x{$p=>(),_=>()}}}fn f(x:&str){m!(x,"a");}"#,
    ),
    ("byte-array", r#"fn f(x:&[u8;2]){match x{b"ab"=>(),_=>()}}"#),
    ("byte-slice", r#"fn f(x:&[u8]){match x{b"ab"=>(),_=>()}}"#),
    (
        "byte-raw",
        r##"fn f(x:&[u8]){match x{br#"ab"#=>(),b"\x61b"=>(),_=>()}}"##,
    ),
    (
        "byte-constant",
        r#"const C:&[u8;2]=b"ab";fn f(x:&[u8;2]){match x{C=>(),_=>()}}"#,
    ),
    (
        "byte-unsized-constant",
        r#"const C:&[u8]=b"ab";fn f(x:&[u8]){match x{C=>(),_=>()}}"#,
    ),
    (
        "byte-mixed",
        r#"fn f(x:&[u8]){match x{b""=>(),[_,..]=>()}}"#,
    ),
    ("byte-empty-array", r#"fn f(x:&[u8;0]){let b""=x;}"#),
    (
        "byte-prefix",
        r#"fn f(x:&[u8]){match x{[97,..]=>(),b"ab"=>(),_=>()}}"#,
    ),
    (
        "byte-suffix",
        r#"fn f(x:&[u8]){match x{[..,98]=>(),b"ab"=>(),_=>()}}"#,
    ),
    (
        "byte-nul",
        r#"fn f(x:&[u8]){match x{b"\0\xff"=>(),[0,255]=>(),_=>()}}"#,
    ),
    (
        "byte-nested",
        r#"const C:(&[u8],bool)=(b"ab",true);fn f(x:(&[u8],bool)){match x{C=>(),(b"ab",false)=>(),_=>()}}"#,
    ),
    (
        "string-slice",
        r#"fn f(x:&[&str]){match x{["a",..,"b"]=>(),["a","b"]=>(),[]=>(),_=>()}}"#,
    ),
    ("static", r#"static C:&str="a";static B:&[u8]=b"ab";"#),
    (
        "function-elision",
        "fn id(x:&str)->&str{x}const F:fn(&str)->&str=id;",
    ),
];
const REJECTED: &[(&str, &str)] = &[
    ("finite-strings", r#"fn f(x:&str){match x{""|"a"=>()}}"#),
    ("refutable-string", r#"fn f(x:&str){let "a"=x;}"#),
    ("refutable-bytes", r#"fn f(x:&[u8]){let b""=x;}"#),
    (
        "guard-exhaustiveness",
        "fn f(x:&str){match x{_ if true=>()}}",
    ),
    ("string-mut", r#"fn f(x:&mut str){match x{"a"=>(),_=>()}}"#),
    (
        "string-nested-ref",
        r#"fn f(x:&&str){match x{"a"=>(),_=>()}}"#,
    ),
    (
        "string-constant-mut",
        r#"const C:&str="a";fn f(x:&mut str){match x{C=>(),_=>()}}"#,
    ),
    (
        "string-constant-nested-ref",
        r#"const C:&str="a";fn f(x:&&str){match x{C=>(),_=>()}}"#,
    ),
    (
        "byte-mut",
        r#"fn f(x:&mut [u8;2]){match x{b"ab"=>(),_=>()}}"#,
    ),
    (
        "byte-nested-ref",
        r#"fn f(x:&&[u8;2]){match x{b"ab"=>(),_=>()}}"#,
    ),
    ("byte-owned", r#"fn f(x:[u8;2]){match x{b"ab"=>(),_=>()}}"#),
    (
        "byte-length",
        r#"fn f(x:&[u8;3]){match x{b"ab"=>(),_=>()}}"#,
    ),
    (
        "constant-no-unsize",
        r#"const C:&[u8;2]=b"ab";fn f(x:&[u8]){match x{C=>(),_=>()}}"#,
    ),
    (
        "constant-no-resize",
        r#"const C:&[u8]=b"ab";fn f(x:&[u8;2]){match x{C=>(),_=>()}}"#,
    ),
    ("byte-finite", r#"fn f(x:&[u8]){match x{b""|b"a"=>()}}"#),
    (
        "static-pattern",
        r#"static C:&str="a";fn f(x:&str){match x{C=>(),_=>()}}"#,
    ),
    ("alias-no-elision", "type S=&str;"),
    (
        "function-no-output-elision",
        r#"const F:fn()->&str=f;fn f()->&'static str{"a"}"#,
    ),
];
fn folder() -> Result<PathBuf, std::io::Error> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../obj")
        .join(format!("text-patterns-{}", std::process::id()));
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
            "text_patterns",
            "--emit=metadata",
        ])
        .arg(path)
        .arg("-o")
        .arg(path.with_extension("rmeta"))
        .output()
}

#[test]
fn text_pattern_acceptance_and_rejection_agree_with_rustc() -> Result<(), Box<dyn std::error::Error>>
{
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
            std::fs::write(path.with_extension("patterns"), &emitted.stdout)?;
            std::fs::write(path.with_extension("patterns-stderr"), &emitted.stderr)?;
            let counted = frontend(&path, "--count-patterns")?;
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
                let view = std::str::from_utf8(&emitted.stdout)?;
                let footer = view
                    .lines()
                    .last()
                    .and_then(|s| s.strip_prefix("eof "))
                    .ok_or("missing footer")?;
                assert_eq!(format!("{footer}\n").as_bytes(), counted.stdout, "{name}");
                let repeated = frontend(&path, "--emit-patterns")?;
                assert!(repeated.status.success());
                assert_eq!(emitted.stdout, repeated.stdout, "{name}");
            } else {
                assert!(emitted.stdout.is_empty() && counted.stdout.is_empty());
                assert!(!emitted.stderr.is_empty());
            }
        }
    }
    Ok(())
}

#[test]
fn decoded_text_identity_and_coverage_preserve_exact_bytes() -> Result<(), String> {
    let (_, facts) = analyze(
        r##"fn f(x:&str){match x{r#"a"#=>(),"\x61"=>(),"é"=>(),"e\u{301}"=>(),""=>(),"\0"=>(),_=>()}}"##,
    )?;
    assert_eq!(facts.texts.len(), 5);
    let useful: Vec<_> = facts.patterns.arms.iter().map(|a| a.useful).collect();
    assert_eq!(useful, [true, false, true, true, true, true, true]);
    assert!(facts.region_equalities.is_empty());
    assert!(facts.body_regions.is_empty());
    let (_, facts) = analyze(r#"fn f(x:&[u8]){match x{[..,98]=>(),b"ab"=>(),_=>()}}"#)?;
    assert_eq!(
        facts
            .patterns
            .arms
            .iter()
            .map(|a| a.useful)
            .collect::<Vec<_>>(),
        [true, false, true]
    );
    Ok(())
}

#[test]
fn value_patterns_do_not_impose_storage_lifetimes() -> Result<(), String> {
    for source in [
        r#"fn f<'a>(x:&'a str){match x{"a"=>(),_=>()}}"#,
        r#"const C:&str="a";fn f<'a>(x:&'a str){match x{C=>(),_=>()}}"#,
        r#"const C:(&str,bool)=("a",true);fn f<'a>(x:(&'a str,bool)){match x{C=>(),_=>()}}"#,
        r#"fn f<'a>(x:&'a [u8]){match x{b"ab"=>(),_=>()}}"#,
    ] {
        let (_, facts) = analyze(source)?;
        assert!(facts.region_equalities.is_empty(), "{source}");
    }
    let (_, facts) = analyze(r#"fn f<'a,'b>(x:&'a str,y:&'b str)->&'a str{match x{"a"=>y,_=>x}}"#)?;
    assert!(!facts.coercions.is_empty());
    let (_, facts) = analyze(r#"fn f<'a>(_:&'a u8)->&'a [u8]{b"ab"}"#)?;
    assert!(!facts.coercions.is_empty());
    Ok(())
}

#[test]
fn repeated_texts_share_nodes_but_not_selected_places() -> Result<(), String> {
    let (_, facts) =
        analyze(r#"const C:[&str;64]=["a";64];fn f(x:[&str;64]){match x{C=>(),_=>()}}"#)?;
    assert_eq!(facts.texts.len(), 1);
    assert_eq!(facts.texts.byte_len(), 1);
    assert_eq!(facts.patterns.fields.len(), 64);
    let child = facts.patterns.fields[0].pattern;
    assert!(facts.patterns.fields.iter().all(|f| f.pattern == child));
    assert_eq!(
        facts
            .patterns
            .nodes
            .iter()
            .filter(|p| matches!(p.kind, Kind::Text(_)))
            .count(),
        1
    );
    assert!(facts.patterns.nodes.len() < 10);
    Ok(())
}

#[test]
fn text_pool_is_flat_and_reuses_equal_bytes() -> Result<(), Box<dyn std::error::Error>> {
    let mut texts = Texts::default();
    assert!(texts.is_empty());
    for bytes in [
        &b""[..],
        b"a",
        b"a\0",
        b"\xff",
        "é".as_bytes(),
        "e\u{301}".as_bytes(),
    ] {
        let id = texts.intern(bytes)?;
        assert_eq!(texts.intern(bytes)?, id);
        assert_eq!(texts.get(id)?, bytes);
    }
    assert_eq!(texts.len(), 6);
    assert_eq!(texts.byte_len(), 9);
    assert!(texts.get(Id(u32::MAX)).is_err());
    Ok(())
}

#[test]
fn text_values_and_pattern_decisions_pass_execution_proofs()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = folder()?;
    for (index, source) in [
        r#"fn main(){let _="a";}"#,
        r#"fn main(){let _=b"ab";}"#,
        r#"fn f(x:&str){match x{"a"=>(),_=>()}}fn main(){}"#,
        r#"fn f(x:&[u8]){match x{b"ab"=>(),_=>()}}fn main(){}"#,
    ]
    .iter()
    .enumerate()
    {
        let path = directory.join(format!("boundary{index}.rs"));
        std::fs::write(&path, source)?;
        assert!(frontend(&path, "--emit-patterns")?.status.success());
        let output = frontend(&path, "--emit-lowir")?;
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stderr.is_empty() && !output.stdout.is_empty());
    }
    Ok(())
}

#[test]
fn text_coverage_queries_visit_only_their_own_patterns() -> Result<(), String> {
    let source = r#"fn target(x:&str){match x{"a"=>(),_=>()}}"#;
    let (_, baseline) = analyze(source)?;
    let work = baseline.patterns.decisions[0].work;
    let mut combined = String::new();
    for index in 0..128 {
        use std::fmt::Write;
        write!(
            combined,
            "fn f{index}(x:&str){{match x{{\"{index}\"=>(),_=>()}}}}"
        )
        .map_err(|e| e.to_string())?;
    }
    combined.push_str(source);
    let (_, facts) = analyze(&combined)?;
    assert_eq!(facts.texts.len(), 129);
    assert!(facts.patterns.decisions.iter().all(|d| d.work == work));
    let (_, facts) = analyze(r#"fn f(){let _="never demanded";}"#)?;
    assert_eq!(facts.texts.len(), 1);
    assert_eq!(facts.text_literals.len(), 1);
    assert_eq!(facts.texts.get(Id(0))?, b"never demanded");
    assert!(facts.patterns.decisions.is_empty());
    Ok(())
}

#[test]
fn text_constants_keep_file_provenance_and_distinguish_string_from_bytes()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = folder()?.join("package");
    std::fs::create_dir_all(&directory)?;
    std::fs::write(
        directory.join("values.rs"),
        r#"pub const S:&str="a";pub const B:&[u8;1]=b"a";"#,
    )?;
    let path = directory.join("lib.rs");
    std::fs::write(
        &path,
        "mod values;use values::{S,B};fn f(x:(&str,&[u8;1])){match x{(S,B)=>(),_=>()}}",
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
    assert!(view.contains("text=\"a\""));
    assert!(view.contains("ranks=97:97"));
    assert!(
        view.lines()
            .filter(|s| s.starts_with("pattern "))
            .all(|s| s.contains("source=0:"))
    );
    let (_, facts) = analyze(
        r#"const S:&str="a";const B:&[u8;1]=b"a";fn f(x:(&str,&[u8;1])){match x{(S,B)=>(),_=>()}}"#,
    )?;
    assert_eq!(facts.texts.len(), 1);
    Ok(())
}

#[test]
fn text_witnesses_and_expansion_limits_are_atomic() -> Result<(), Box<dyn std::error::Error>> {
    let directory = folder()?;
    for (name, source, diagnostic) in [
        (
            "other",
            r#"fn f(x:&str){match x{""|"a"=>()}}"#.to_owned(),
            "other string",
        ),
        (
            "known",
            r#"fn f(x:(&str,bool)){match x{("a",true)|("a",false)=>(),(_,true)=>()}}"#.to_owned(),
            "other string",
        ),
        (
            "budget",
            format!(
                "fn f(x:&[u8]){{match x{{b\"{}\"=>(),_=>()}}}}",
                "a".repeat(1_000_001)
            ),
            "constant pattern expansion work budget exceeded",
        ),
    ] {
        let path = directory.join(format!("{name}.rs"));
        std::fs::write(&path, source)?;
        let emitted = frontend(&path, "--emit-patterns")?;
        let counted = frontend(&path, "--count-patterns")?;
        for output in [&emitted, &counted] {
            assert_eq!(output.status.code(), Some(1));
            assert!(output.stdout.is_empty());
            assert!(
                String::from_utf8_lossy(&output.stderr).contains(diagnostic),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        assert_eq!(emitted.stderr, counted.stderr);
    }
    Ok(())
}

#[test]
fn wide_pointer_fields_are_supported_while_c_strings_keep_their_later_boundary()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = folder()?;
    for (name, source, boundary) in [
        ("str-field", "struct S(&'static str);", None),
        ("slice-field", "struct S(&'static [u8]);", None),
        (
            "c-string",
            r#"fn f(){let _=c"a";}"#,
            Some("C string values require the library type assignment"),
        ),
    ] {
        let path = directory.join(format!("deferred-{name}.rs"));
        std::fs::write(&path, source)?;
        assert!(rustc(&path)?.status.success());
        let output = frontend(&path, "--emit-patterns")?;
        if let Some(boundary) = boundary {
            assert_eq!(output.status.code(), Some(1));
            assert!(output.stdout.is_empty());
            assert!(String::from_utf8_lossy(&output.stderr).contains(boundary));
        } else {
            assert!(output.status.success());
            assert!(output.stderr.is_empty() && !output.stdout.is_empty());
        }
    }
    Ok(())
}
