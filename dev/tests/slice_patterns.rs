use std::{
    fmt::Write,
    path::{Path, PathBuf},
    process::{Command, Output},
};

const CASES: &[(&str, i32, &str)] = &[
    (
        "sparse-alias-gap",
        0,
        "fn f(x:&[bool]){match x{[_,_,true]=>(),[_,..,true]=>(),[..,false]=>(),[true]=>(),[]=>()}}",
    ),
    (
        "empty-or-nonempty",
        0,
        "fn f(x:&[bool]){match x{[]=>(),[_,..]=>()}}",
    ),
    (
        "prefix-partition",
        0,
        "fn f(x:&[bool]){match x{[]=>(),[true,..]=>(),[false,..]=>()}}",
    ),
    (
        "suffix-partition",
        0,
        "fn f(x:&[bool]){match x{[]=>(),[..,true]=>(),[..,false]=>()}}",
    ),
    (
        "short-alias-exhaustive",
        0,
        "fn f(x:&[bool]){match x{[]=>(),[true,..]=>(),[..,false]=>(),[false,..,true]=>()}}",
    ),
    (
        "length-gap",
        1,
        "fn f(x:&[bool]){match x{[]=>(),[_,_]=>(),[_,_,_,..]=>()}}",
    ),
    (
        "value-gap",
        1,
        "fn f(x:&[bool]){match x{[]=>(),[true,..]=>(),[..,false]=>()}}",
    ),
    (
        "singleton-value-gap",
        1,
        "fn f(x:&[bool]){match x{[]=>(),[true]=>(),[_,_,..]=>()}}",
    ),
    ("or-irrefutable", 0, "fn f(x:&[bool]){let([]|[_,..])=x;}"),
    ("let-refutable", 1, "fn f(x:&[bool]){let[_,..]=x;}"),
    (
        "let-else",
        0,
        "fn f(x:&[bool]){let[a,rest@..]=x else{return;};let _=a;let _=rest;}",
    ),
    (
        "if-let-rest",
        0,
        "fn f(x:&[bool]){if let[a,rest@..,z]=x{let _=a;let _=z;let _=rest;}}",
    ),
    (
        "or-capture",
        0,
        "fn f(x:&[bool]){if let[a,..]|[..,a]=x{let _=a;}}",
    ),
    (
        "mutable-inherited",
        0,
        "fn f(x:&mut[bool]){if let[a,..]=x{let _=a;}}",
    ),
    (
        "written-ref-inherited",
        1,
        "fn f(x:&[bool]){if let[ref a,..]=x{}}",
    ),
    (
        "owned-unsized-rest",
        1,
        "fn f(x:&[bool]){if let &[a,rest@..]=x{}}",
    ),
    ("duplicate-rest", 1, "fn f(x:&[bool]){if let[..,..]=x{}}"),
    (
        "nested-slice",
        0,
        "fn f(x:&[&[bool]]){match x{[]=>(),[[],..]=>(),[[_,..],..]=>()}}",
    ),
    (
        "guard-does-not-cover",
        1,
        "fn f(x:&[bool]){match x{[]=>(),[_,..] if true=>()}}",
    ),
    (
        "empty-element-only",
        1,
        "enum Void{}fn f(x:&[Void]){match x{[]=>()}}",
    ),
    (
        "empty-element-wildcards",
        0,
        "enum Void{}fn f(x:&[Void]){match x{[]=>(),[_,..]=>()}}",
    ),
    (
        "integer-ranges",
        0,
        "fn f(x:&[u8]){match x{[]=>(),[0..=127,..]=>(),[128..=255,..]=>()}}",
    ),
    (
        "named-constant",
        0,
        "const YES:bool=true;fn f(x:&[bool]){match x{[]=>(),[..,YES]=>(),[..,false]=>()}}",
    ),
    (
        "float-elements",
        0,
        "fn f(x:&[f32]){match x{[]=>(),[0.0,..]=>(),[..,-1.0]=>(),_=>()}}",
    ),
    (
        "float-nan-gap",
        1,
        "fn f(x:&[f32]){match x{[]=>(),[0.0,..]=>()}}",
    ),
    (
        "tuple-slices",
        0,
        "fn f(x:(&[bool],&[bool])){match x{([]|[true,..],_)=>(),([false,..],[])=>(),([false,..],[_,..])=>()}}",
    ),
    (
        "mutable-rest",
        0,
        "fn f(x:&mut[u8]){if let[a,rest@..,z]=x{let _=a;let _=rest;let _=z;}}",
    ),
    (
        "explicit-reference",
        0,
        "fn f(x:&[u8]){match x{&[]=>(),&[n,..]=>{let _n:u8=n;}}}",
    ),
    (
        "while-let",
        0,
        "fn f(mut x:&[u8]){while let[_,rest@..]=x{x=rest;}}",
    ),
    (
        "condition-chain",
        0,
        "fn f(x:&[u8],yes:bool){if let[a,..]=x&&yes&&let[b,..]=x{let _=a;let _=b;}}",
    ),
    (
        "missing-or-binding",
        1,
        "fn f(x:&[u8]){if let[a,..]|[..,a,b]=x{}}",
    ),
    ("slice-parameter", 1, "fn f([_,..]:&[u8]){}"),
    ("slice-or-parameter", 0, "fn f(([]|[_,..]):&[u8]){}"),
    (
        "macro-slice",
        0,
        "macro_rules! m{($x:expr)=>{match $x{[]=>(),[_,..]=>()}}}fn f(x:&[u8]){m!(x);}",
    ),
    (
        "inactive-arm",
        0,
        "fn f(x:&[bool]){match x{#[cfg(any())][missing,..]=>(),[..]=>()}}",
    ),
];

fn folder() -> Result<PathBuf, std::io::Error> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/pa13-work/slice-vectors")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&path)?;
    Ok(path)
}
fn frontend(path: &Path, mode: &str) -> Result<Output, std::io::Error> {
    Command::new(env!("CARGO_BIN_EXE_rustgm"))
        .arg(mode)
        .arg(path)
        .output()
}
fn check(path: &Path, expected: i32) -> Result<String, Box<dyn std::error::Error>> {
    let independent = Command::new("rustc")
        .args([
            "--edition=2024",
            "--crate-type=lib",
            "--crate-name",
            "slice_patterns",
            "--emit=metadata",
        ])
        .arg(path)
        .arg("-o")
        .arg(path.with_extension("rmeta"))
        .output()?;
    std::fs::write(path.with_extension("rustc-stderr"), &independent.stderr)?;
    assert_eq!(
        independent.status.code(),
        Some(expected),
        "{}: {}",
        path.display(),
        String::from_utf8_lossy(&independent.stderr)
    );
    let output = frontend(path, "--emit-patterns")?;
    std::fs::write(path.with_extension("patterns"), &output.stdout)?;
    std::fs::write(path.with_extension("patterns-stderr"), &output.stderr)?;
    assert_eq!(
        output.status.code(),
        Some(expected),
        "{}: {}",
        path.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    let counted = frontend(path, "--count-patterns")?;
    assert_eq!(counted.status.code(), Some(expected));
    assert_eq!(counted.stderr, output.stderr);
    if expected != 0 {
        assert!(output.stdout.is_empty() && counted.stdout.is_empty() && !output.stderr.is_empty());
        return Ok(String::new());
    }
    assert!(output.stderr.is_empty());
    let repeated = frontend(path, "--emit-patterns")?;
    assert!(repeated.status.success() && repeated.stderr.is_empty());
    assert_eq!(repeated.stdout, output.stdout);
    let text = String::from_utf8(output.stdout)?;
    let footer = text
        .lines()
        .last()
        .and_then(|line| line.strip_prefix("eof "))
        .ok_or("missing footer")?;
    assert_eq!(counted.stdout, format!("{footer}\n").as_bytes());
    Ok(text)
}

#[test]
fn dynamic_slice_selection_matches_rustc_and_keeps_proofs_pending()
-> Result<(), Box<dyn std::error::Error>> {
    let folder = folder()?;
    for &(name, expected, source) in CASES {
        let path = folder.join(format!("{name}.rs"));
        std::fs::write(&path, source)?;
        let view = check(&path, expected)?;
        if expected == 0 {
            assert!(view.contains("slice minimum="), "{name}");
            // Selecting borrowed patterns cannot bypass the outstanding loan/region proof.
            let executable = frontend(&path, "--emit-lowir")?;
            assert_eq!(executable.status.code(), Some(1), "{name}");
            assert!(executable.stdout.is_empty() && !executable.stderr.is_empty());
            let diagnostic = String::from_utf8(executable.stderr)?;
            assert!(
                diagnostic.contains("region and loan proofs")
                    || diagnostic.contains("requires owned selected patterns"),
                "{name}: {diagnostic}"
            );
        }
        if name == "mutable-rest" {
            assert!(view.contains("from-end=1"));
            assert!(view.contains("subslice=1:1"));
            assert_eq!(
                view.lines()
                    .filter(|line| line.contains("capture=ref-mut"))
                    .count(),
                3
            );
        }
    }
    Ok(())
}

#[test]
fn slice_partition_work_is_bounded_and_errors_are_atomic() -> Result<(), Box<dyn std::error::Error>>
{
    let folder = folder()?;
    let wide = vec!["_"; 5000].join(",");
    // These stress cases check our work budget, not rustc's large-pattern throughput.
    for (name, first) in [
        ("wildcard", wide.clone()),
        ("sparse", format!("{wide},true")),
    ] {
        let path = folder.join(format!("wide-{name}.rs"));
        std::fs::write(
            &path,
            format!(
                "fn f(x:&[bool]){{match x{{[{first}]=>(),[..,true]=>(),[..,false]=>(),[]=>()}}}}"
            ),
        )?;
        let output = frontend(&path, "--emit-patterns")?;
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stderr.is_empty());
        let view = std::str::from_utf8(&output.stdout)?;
        let work = view
            .lines()
            .find(|line| line.starts_with("match "))
            .and_then(|line| line.rsplit_once("work="))
            .ok_or("missing slice work counter")?
            .1
            .parse::<u64>()?;
        assert!(work < 100_000, "{name}: {work}");
    }
    let complex = folder.join("dense-budget.rs");
    let dense = vec!["true"; 5000].join(",");
    std::fs::write(
        &complex,
        format!("fn f(x:&[bool]){{match x{{[{dense}]=>(),[..,true]=>(),_=>()}}}}"),
    )?;
    let emitted = frontend(&complex, "--emit-patterns")?;
    let counted = frontend(&complex, "--count-patterns")?;
    for output in [&emitted, &counted] {
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert!(
            String::from_utf8_lossy(&output.stderr)
                .contains("pattern coverage work budget exceeded")
        );
    }
    assert_eq!(emitted.stderr, counted.stderr);
    Ok(())
}

struct Shape {
    prefix: Vec<Option<bool>>,
    suffix: Vec<Option<bool>>,
    rest: bool,
}
impl Shape {
    fn accepts(&self, value: &[bool]) -> bool {
        let width = self.prefix.len() + self.suffix.len();
        if value.len() < width || !self.rest && value.len() != width {
            return false;
        }
        self.prefix
            .iter()
            .zip(value)
            .all(|(p, v)| p.is_none_or(|p| p == *v))
            && self
                .suffix
                .iter()
                .rev()
                .zip(value.iter().rev())
                .all(|(p, v)| p.is_none_or(|p| p == *v))
    }
    fn source(&self) -> String {
        let spelling = |v: &Option<bool>| match v {
            None => "_",
            Some(true) => "true",
            Some(false) => "false",
        };
        let mut fields: Vec<_> = self.prefix.iter().map(spelling).collect();
        if self.rest {
            fields.push("..");
        }
        fields.extend(self.suffix.iter().map(spelling));
        format!("[{}]", fields.join(","))
    }
}
fn next(seed: &mut u64) -> u64 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    *seed
}
fn shape(seed: &mut u64) -> Shape {
    let rest = next(seed) & 1 == 0;
    let prefix = next(seed) % 4;
    let suffix = if rest { next(seed) % 3 } else { 0 };
    let mut fields = |n| {
        (0..n)
            .map(|_| match next(seed) % 3 {
                0 => None,
                1 => Some(true),
                _ => Some(false),
            })
            .collect()
    };
    Shape {
        prefix: fields(prefix),
        suffix: fields(suffix),
        rest,
    }
}

#[test]
fn boolean_slice_coverage_matches_exhaustive_sequences() -> Result<(), Box<dyn std::error::Error>> {
    let folder = folder()?;
    let mut seed = 0xa875_0964_eef1_7239_u64;
    // Prefix <= 3, suffix <= 2. Lengths 0..=6 cover every aliasing partition,
    // exact-length boundary and one representative of the independent long tail.
    let values: Vec<Vec<bool>> = (0..=6)
        .flat_map(|length| {
            (0..1_u64 << length)
                .map(move |bits| (0..length).map(|i| bits & (1 << i) != 0).collect())
        })
        .collect();
    for case in 0..64 {
        let mut covered = vec![false; values.len()];
        let mut useful = Vec::new();
        let mut arms = String::new();
        for _ in 0..6 {
            let alternatives: Vec<_> = (0..=next(&mut seed) % 2)
                .map(|_| shape(&mut seed))
                .collect();
            let guarded = next(&mut seed).is_multiple_of(5);
            let matches: Vec<_> = values
                .iter()
                .map(|v| alternatives.iter().any(|p| p.accepts(v)))
                .collect();
            useful.push(
                matches
                    .iter()
                    .zip(&covered)
                    .any(|(matched, old)| *matched && !old),
            );
            if !guarded {
                for (old, matched) in covered.iter_mut().zip(matches) {
                    *old |= matched;
                }
            }
            let patterns = alternatives
                .iter()
                .map(Shape::source)
                .collect::<Vec<_>>()
                .join("|");
            write!(
                arms,
                "{patterns}{}=>(),",
                if guarded { " if true" } else { "" }
            )?;
        }
        let exhaustive = covered.iter().all(|v| *v);
        let path = folder.join(format!("boolean-{case}.rs"));
        std::fs::write(&path, format!("fn f(x:&[bool]){{match x{{{arms}}}}}"))?;
        check(&path, i32::from(!exhaustive))?;
        useful.push(!exhaustive);
        let path = folder.join(format!("boolean-{case}-complete.rs"));
        std::fs::write(
            &path,
            format!("fn f(x:&[bool]){{match x{{{arms}[..]=>()}}}}"),
        )?;
        let view = check(&path, 0)?;
        let actual: Vec<_> = view
            .lines()
            .filter(|line| line.starts_with("arm "))
            .map(|line| line.contains("useful=true"))
            .collect();
        assert_eq!(actual, useful, "case {case}");
    }
    Ok(())
}
