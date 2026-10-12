use std::{
    fmt::Write,
    path::{Path, PathBuf},
    process::{Command, Output},
};

const ACCEPTED: &[(&str, &str)] = &[
    (
        "tuple",
        "fn f(x:(bool,bool)){match x{(true,_)=>(),(_,true)=>(),(false,false)=>()}}",
    ),
    (
        "tuple-or",
        "fn f(x:(bool,u8)){match x{(true,_)|(false,0..=127)=>(),(false,128..)=>()}}",
    ),
    (
        "tuple-binding",
        "fn f(x:(bool,u8)){let((true,n)|(false,n))=x;let _=n;}",
    ),
    (
        "record-order",
        "struct S{a:bool,b:bool}fn f(x:S){match x{S{b:true,a:_}=>(),S{a:true,b:false}=>(),S{b:false,a:false}=>()}}",
    ),
    (
        "unit-enum",
        "enum E{A,B,C}fn f(x:E){match x{E::A|E::B=>(),E::C=>()}}",
    ),
    (
        "enum-payload",
        "enum E{A(bool),B{v:bool},C}fn f(x:E){match x{E::A(true)|E::B{v:true}=>(),E::A(false)|E::B{v:false}=>(),E::C=>()}}",
    ),
    (
        "enum-binding",
        "enum E{A(u8),B(u8)}fn f(x:E){let(E::A(n)|E::B(n))=x;let _=n;}",
    ),
    (
        "enum-nested",
        "enum I{A,B}enum E{X(I),Y}fn f(x:E){match x{E::X(I::A|I::B)=>(),E::Y=>()}}",
    ),
    (
        "guard-fallback",
        "enum E{A(bool),B}fn f(x:E){match x{E::A(true) if false=>(),E::A(_)=>(),E::B=>()}}",
    ),
    (
        "array",
        "fn f(x:[bool;2]){match x{[true,_]=>(),[false,true]=>(),[false,false]=>()}}",
    ),
    (
        "array-rest",
        "fn f(x:[bool;4]){match x{[true,..]=>(),[false,..,true]=>(),[false,..,false]=>()}}",
    ),
    (
        "array-huge",
        "fn f(x:[();18446744073709551615]){match x{[(),..]=>()}}",
    ),
    (
        "array-empty",
        "enum Void{}fn f(x:[Void;0]){match x{[]=>()}}",
    ),
    (
        "array-uninhabited",
        "enum Void{}fn f(x:[Void;1]){match x{}}",
    ),
    ("enum-empty", "enum Void{}fn f(x:Void){match x{}}"),
    ("tuple-empty", "enum Void{}fn f(x:(Void,bool)){match x{}}"),
    (
        "struct-empty",
        "enum Void{}struct S{v:Void}fn f(x:S){match x{}}",
    ),
    (
        "variant-empty",
        "enum Void{}enum E{A(Void),B}fn f(x:E){match x{E::B=>()}}",
    ),
    (
        "public-empty",
        "mod m{pub enum Void{}pub struct S{pub v:Void}}fn f(x:m::S){match x{}}",
    ),
    (
        "private-wildcard",
        "mod m{pub enum Void{}pub struct S{v:Void}}fn f(x:m::S){match x{m::S{..}=>()}}",
    ),
    (
        "private-owner",
        "enum Void{}mod m{pub struct S{v:super::Void}fn f(x:S){match x{}}}fn f(x:m::S){match x{m::S{..}=>()}}",
    ),
    (
        "reference-empty-wildcard",
        "enum Void{}fn f(x:&Void){match x{&_=>()}}",
    ),
    (
        "reference-tuple",
        "fn f(x:&(bool,bool)){match x{(true,_)=>(),(false,_)=>()}}",
    ),
    (
        "reference-enum",
        "enum E{A(bool),B}fn f(x:&E){match x{E::A(true|false)=>(),E::B=>()}}",
    ),
    (
        "reference-binding",
        "enum E{A(u8),B(u8)}fn f(x:&E){let(E::A(n)|E::B(n))=x;let _=n;}",
    ),
    ("slice-rest", "fn f(x:&[u8]){match x{[..]=>()}}"),
    ("opaque-wildcard", "fn f(x:f32){match x{_=>()}}"),
    (
        "macro",
        "macro_rules! cover{($x:expr,$p:pat)=>{match $x{$p=>()}}}fn f(x:(bool,bool)){cover!(x,(true|false,true|false));}",
    ),
    (
        "cfg-variant",
        "enum E{A,#[cfg(any())]B}fn f(x:E){match x{E::A=>()}}",
    ),
];
const REJECTED: &[(&str, &str)] = &[
    (
        "tuple-gap",
        "fn f(x:(bool,bool)){match x{(true,true)|(false,false)=>()}}",
    ),
    (
        "tuple-binding-gap",
        "fn f(x:(bool,u8)){let(true,n)=x;let _=n;}",
    ),
    ("unit-gap", "enum E{A,B,C}fn f(x:E){match x{E::A|E::B=>()}}"),
    (
        "payload-gap",
        "enum E{A(bool),B}fn f(x:E){match x{E::A(true)=>(),E::B=>()}}",
    ),
    (
        "enum-binding-gap",
        "enum E{A(u8),B(u8)}fn f(x:E){let E::A(n)=x;let _=n;}",
    ),
    (
        "guard-gap",
        "enum E{A,B}fn f(x:E){match x{E::A if true=>(),E::B=>()}}",
    ),
    (
        "array-gap",
        "fn f(x:[bool;2]){match x{[true,_]=>(),[false,true]=>()}}",
    ),
    (
        "array-correlation",
        "fn f(x:[bool;2]){match x{[true,true]|[false,false]=>()}}",
    ),
    ("array-zero", "enum Void{}fn f(x:[Void;0]){match x{}}"),
    (
        "private-empty",
        "mod m{pub enum Void{}pub struct S{v:Void}}fn f(x:m::S){match x{}}",
    ),
    (
        "visibility-cache",
        "enum Void{}mod m{pub struct S{v:super::Void}fn f(x:S){match x{}}}fn f(x:m::S){match x{}}",
    ),
    ("reference-empty", "enum Void{}fn f(x:&Void){match x{}}"),
    (
        "reference-variant",
        "enum Void{}enum E{A(Void),B}fn f(x:&E){match x{E::B=>()}}",
    ),
    (
        "reference-tuple-gap",
        "enum Void{}fn f(x:&(Void,bool)){match x{(_,true)=>()}}",
    ),
    ("unit-empty", "fn f(x:()){match x{}}"),
    (
        "validity-cache",
        "enum Void{}enum E{A(Void),B}fn owned(x:E){match x{E::B=>()}}fn borrowed(x:&E){match x{E::B=>()}}",
    ),
];

fn folder(name: &str) -> Result<PathBuf, std::io::Error> {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/pa13-work")
        .join(name)
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    Ok(folder)
}
fn candidate(file: &Path, mode: &str) -> Result<Output, std::io::Error> {
    Command::new(env!("CARGO_BIN_EXE_rustgm"))
        .arg(mode)
        .arg(file)
        .output()
}
fn rustc(file: &Path) -> Result<Output, std::io::Error> {
    Command::new("rustc")
        .args([
            "--edition=2024",
            "--crate-type=lib",
            "--crate-name=coverage",
            "--cap-lints=allow",
            "--emit=metadata",
        ])
        .arg(file)
        .arg("-o")
        .arg(file.with_extension("rmeta"))
        .output()
}

#[test]
fn constructor_coverage_agrees_with_pinned_rustc() -> Result<(), Box<dyn std::error::Error>> {
    let folder = folder("constructor-vectors")?;
    for (cases, status) in [(ACCEPTED, 0), (REJECTED, 1)] {
        for (name, source) in cases {
            let file = folder.join(format!("{name}.rs"));
            std::fs::write(&file, source)?;
            let checked = candidate(&file, "--emit-patterns")?;
            let independent = rustc(&file)?;
            for (label, result) in [("candidate", &checked), ("rustc", &independent)] {
                std::fs::write(
                    file.with_extension(format!("{label}.stderr")),
                    &result.stderr,
                )?;
                assert_eq!(
                    result.status.code(),
                    Some(status),
                    "{name}/{label}: {}",
                    String::from_utf8_lossy(&result.stderr)
                );
            }
            std::fs::write(file.with_extension("patterns"), &checked.stdout)?;
            let count = candidate(&file, "--count-patterns")?;
            assert_eq!(count.status.code(), Some(status));
            if status == 0 {
                assert!(checked.stderr.is_empty() && count.stderr.is_empty());
                let again = candidate(&file, "--emit-patterns")?;
                assert_eq!(checked.stdout, again.stdout);
                let text = std::str::from_utf8(&checked.stdout)?;
                assert_eq!(
                    text.lines()
                        .last()
                        .ok_or("missing pattern footer")?
                        .strip_prefix("eof ")
                        .ok_or("invalid footer")?,
                    std::str::from_utf8(&count.stdout)?.trim_end()
                );
            } else {
                assert!(checked.stdout.is_empty() && count.stdout.is_empty());
            }
        }
    }
    Ok(())
}

#[test]
fn nested_alternatives_stay_shared_and_arrays_stay_sparse() -> Result<(), Box<dyn std::error::Error>>
{
    let folder = folder("coverage-sharing")?;
    let file = folder.join("product.rs");
    let types = vec!["bool"; 64].join(",");
    let patterns = vec!["true|false"; 64].join(",");
    std::fs::write(
        &file,
        format!("fn f(x:({types})){{match x{{({patterns})=>()}}}}"),
    )?;
    let output = candidate(&file, "--emit-patterns")?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    // The 64-column case deliberately checks our sharing bound. The smaller
    // independent control has the same product/or semantics without requiring
    // rustc to finish the much larger stress input.
    let control = folder.join("small.rs");
    std::fs::write(
        &control,
        "fn f(x:(bool,bool,bool,bool)){match x{(true|false,true|false,true|false,true|false)=>()}}",
    )?;
    assert!(rustc(&control)?.status.success());
    let text = std::str::from_utf8(&output.stdout)?;
    let record = text
        .lines()
        .find(|l| l.starts_with("match "))
        .ok_or("missing decision")?;
    let work: u64 = record
        .split("work=")
        .nth(1)
        .ok_or("missing coverage work")?
        .parse()?;
    assert!(work < 10_000, "shared alternatives used {work} operations");
    let sparse = folder.join("sparse.rs");
    std::fs::write(
        &sparse,
        "fn f(x:[();1000000000000]){match x{[(),..,()]=>()}}",
    )?;
    let output = candidate(&sparse, "--count-patterns")?;
    assert!(output.status.success());
    let counts: Vec<_> = std::str::from_utf8(&output.stdout)?
        .split_whitespace()
        .collect();
    assert!(counts.contains(&"patterns=3") && counts.contains(&"fields=2"));
    Ok(())
}

#[test]
fn empty_domain_proofs_do_not_bypass_place_initialization() -> Result<(), Box<dyn std::error::Error>>
{
    let folder = folder("coverage-validity")?;
    let file = folder.join("empty.rs");
    std::fs::write(&file, "enum Void{}fn main(){let x:Void;match x{}}")?;
    assert!(candidate(&file, "--emit-patterns")?.status.success());
    assert_eq!(rustc(&file)?.status.code(), Some(1));
    let body = candidate(&file, "--emit-body-ir")?;
    assert!(body.status.success() && body.stderr.is_empty());
    assert!(String::from_utf8_lossy(&body.stdout).contains("validate l"));
    for mode in ["--emit-ownership", "--emit-lowir", "--count-lowir"] {
        let result = candidate(&file, mode)?;
        assert_eq!(result.status.code(), Some(1));
        assert!(result.stdout.is_empty());
        assert!(String::from_utf8_lossy(&result.stderr).contains("uninitialized or moved"));
    }
    Ok(())
}

#[test]
fn product_usefulness_matches_every_subset_of_boolean_pairs()
-> Result<(), Box<dyn std::error::Error>> {
    let folder = folder("coverage-truth-tables")?;
    let pairs = [
        "(false,false)",
        "(true,false)",
        "(false,true)",
        "(true,true)",
    ];
    for mask in 0_u32..16 {
        let mut source = String::from("fn f(x:(bool,bool)){match x{");
        let mut expected = Vec::new();
        for (point, pattern) in pairs.iter().enumerate() {
            if mask & (1 << point) != 0 {
                write!(source, "{pattern}=>(),")?;
                expected.push(true);
            }
        }
        for (point, pattern) in pairs.iter().enumerate() {
            write!(source, "{pattern} if false=>(),")?;
            expected.push(mask & (1 << point) == 0);
        }
        source.push_str("_=>()}}");
        expected.push(mask != 15);
        let file = folder.join(format!("mask-{mask}.rs"));
        std::fs::write(&file, source)?;
        let output = candidate(&file, "--emit-patterns")?;
        assert!(
            output.status.success(),
            "{mask}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let text = std::str::from_utf8(&output.stdout)?;
        let actual: Vec<_> = text
            .lines()
            .filter(|line| line.starts_with("arm "))
            .map(|line| line.contains("useful=true"))
            .collect();
        assert_eq!(actual, expected, "mask {mask}");
    }
    Ok(())
}

#[test]
fn missing_witnesses_name_selected_fields_and_generated_variants()
-> Result<(), Box<dyn std::error::Error>> {
    let folder = folder("coverage-witnesses")?;
    for (index, (source, witness)) in [
        (
            "fn f(x:(bool,bool)){match x{(true,true)|(false,false)=>()}}",
            "{ $.0 = false, $.1 = true }",
        ),
        (
            "enum E{A,B,C}fn f(x:E){match x{E::A|E::B=>()}}",
            "{ $ = E::C }",
        ),
        (
            "macro_rules! decl{()=>{enum E{A,B}}}decl!();fn f(x:E){match x{E::A=>()}}",
            "{ $ = E::B }",
        ),
        (
            "struct S{δ:bool}fn f(x:S){match x{S{δ:true}=>()}}",
            "{ $.δ = false }",
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let file = folder.join(format!("witness-{index}.rs"));
        std::fs::write(&file, source)?;
        assert_eq!(rustc(&file)?.status.code(), Some(1));
        let result = candidate(&file, "--emit-patterns")?;
        assert_eq!(result.status.code(), Some(1));
        assert!(result.stdout.is_empty());
        assert!(
            String::from_utf8_lossy(&result.stderr).contains(witness),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
    Ok(())
}

#[test]
fn scattered_enum_arms_share_domain_updates() -> Result<(), Box<dyn std::error::Error>> {
    let folder = folder("coverage-enum-scaling")?;
    let mut previous = 0;
    for size in [256_usize, 1024, 4096] {
        let mut source = String::from("enum E{");
        for i in 0..size {
            write!(source, "V{i},")?;
        }
        source.push_str("}fn f(x:E){match x{");
        for i in 0..size {
            let index = i.reverse_bits() >> (usize::BITS - size.trailing_zeros());
            write!(source, "E::V{index}=>(),")?;
        }
        source.push_str("}}");
        let file = folder.join(format!("scattered-{size}.rs"));
        std::fs::write(&file, source)?;
        let output = candidate(&file, "--count-patterns")?;
        assert!(
            output.status.success(),
            "{size}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let counts = std::str::from_utf8(&output.stdout)?;
        let work: u64 = counts
            .split_whitespace()
            .find_map(|part| part.strip_prefix("coverage-work="))
            .ok_or("missing coverage work")?
            .parse()?;
        assert!(work < 100 * u64::try_from(size)?);
        if previous != 0 {
            assert!(work < 6 * previous);
        }
        previous = work;
        if size == 256 {
            assert!(rustc(&file)?.status.success());
        }
    }
    Ok(())
}
