//! The analysis-only census reports every isolated failure and never passes a build;
//! ordinary checking of the same inputs keeps its single first error.
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

const HEADER: &str = "#![feature(lang_items,no_core)]\n#![no_core]\n\
#![allow(internal_features,dead_code,unused_variables)]\n\
#[lang=\"pointee_sized\"]pub trait PointeeSized{}\n\
#[lang=\"meta_sized\"]pub trait MetaSized:PointeeSized{}\n\
#[lang=\"sized\"]pub trait Sized:MetaSized{}\n";
const CORE: [&str; 6] = [
    "--source-core",
    "probe@census",
    "--crate-name",
    "probe",
    "--crate-disambiguator",
    "census",
];
const BLOCKED: &str = "blocked by an earlier census failure";

fn input(name: &str, items: &str) -> Result<PathBuf> {
    source(name, &format!("{HEADER}{items}"))
}
/// Write one complete input under this test process's folder.
fn source(name: &str, text: &str) -> Result<PathBuf> {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/census-tests")
        .join(std::process::id().to_string());
    fs::create_dir_all(&folder)?;
    let path = folder.join(format!("{name}.rs"));
    fs::write(&path, text)?;
    Ok(path)
}
fn rustgm(mode: &str, input: &Path, extra: &[&str]) -> Result<Output> {
    Ok(Command::new(env!("CARGO_BIN_EXE_rustgm"))
        .arg(mode)
        .args(CORE)
        .args(extra)
        .arg(input)
        .output()?)
}
/// A census run must fail without view output and leave one complete file.
fn census(mode: &str, input: &Path, tag: &str, extra: &[&str]) -> Result<String> {
    let path = input.with_extension(format!("{tag}.tsv"));
    let path_text = path.to_str().ok_or("census path")?;
    let mut args = extra.to_vec();
    args.extend(["--census-output", path_text]);
    let output = rustgm(mode, input, &args)?;
    let stderr = String::from_utf8(output.stderr)?;
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(output.stdout.is_empty(), "a census emits no view output");
    assert!(stderr.ends_with(&format!(
        "ERROR: analysis-only census written to {path_text}; a census is never a passing build\n"
    )));
    let text = fs::read_to_string(&path)?;
    assert!(text.starts_with("rustgm-census\t1\tanalysis-only\tnot-a-qualification\n"));
    let count = text
        .lines()
        .filter(|line| line.starts_with("record\t"))
        .count();
    assert!(text.ends_with(&format!("end\t{count}\n")), "{text}");
    Ok(text)
}
/// Phase, category, item, root-cause record and message of every record.
fn records(text: &str) -> Vec<[&str; 5]> {
    text.lines()
        .filter_map(|line| line.strip_prefix("record\t"))
        .map(|line| {
            let fields: Vec<_> = line.split('\t').collect();
            [fields[1], fields[2], fields[3], fields[5], fields[6]]
        })
        .collect()
}
fn line<'a>(text: &'a str, prefix: &str) -> Option<&'a str> {
    text.lines().find(|line| line.starts_with(prefix))
}
/// Ordinary checking reports exactly the census's first failure, at the same
/// projected position, and nothing else.
fn ordinary_matches_first_record(input: &Path, text: &str) -> Result {
    let first = line(text, "record\t1\t").ok_or("first record")?;
    let fields: Vec<_> = first.split('\t').collect();
    let (file, offset) = fields[5]
        .strip_prefix("file ")
        .and_then(|location| location.split_once(':'))
        .ok_or("located first record")?;
    for mode in ["--emit-types", "--emit-semantics"] {
        let output = rustgm(mode, input, &[])?;
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert_eq!(
            String::from_utf8(output.stderr)?,
            format!("ERROR: file {file}: {offset}: {}\n", fields[7])
        );
    }
    Ok(())
}

#[test]
fn independent_declaration_failures_and_their_dependents_are_each_reported_once() -> Result {
    let input = input(
        "declarations",
        "struct Bad<const M: usize, const N: M> { field: u8 }\n\
         fn uses(x: Bad<1, 2>) {}\n\
         fn twice(x: Bad<3, 4>, y: Bad<5, 6>) {}\n\
         fn bad_a<const N: usize>(x: N) {}\n\
         type Loop = Loop;\n\
         fn uses_loop(x: Loop) {}\n\
         struct Fine { x: u8 }\n\
         fn fine(x: Fine) {}\n",
    )?;
    let text = census("--emit-types", &input, "types", &[])?;
    let failed = "const parameter cannot be used as a type";
    assert_eq!(
        records(&text),
        [
            ["declarations", "failed", "struct `Bad`", "-", failed],
            ["declarations", "blocked", "function `uses`", "1", BLOCKED],
            ["declarations", "blocked", "function `twice`", "1", BLOCKED],
            ["declarations", "failed", "function `bad_a`", "-", failed],
            [
                "declarations",
                "failed",
                "alias `Loop`",
                "-",
                "recursive type alias"
            ],
            [
                "declarations",
                "blocked",
                "function `uses_loop`",
                "5",
                BLOCKED
            ],
            [
                "declarations",
                "stopped",
                "-",
                "-",
                "later passes were not run after 3 failed and 3 blocked items in declarations"
            ],
        ]
    );
    let summary = line(&text, "summary\tdeclarations\t").ok_or("summary")?;
    assert!(summary.ends_with("\tfailed\t3\tblocked\t3"), "{summary}");
    assert!(text.contains("\nsummary\tbodies\tchecked\t0\tfailed\t0\tblocked\t0\n"));
    assert!(text.contains("\nterminal\t7\tdeclarations\tstopped\n"));
    let path = input.to_str().ok_or("input path")?;
    assert!(text.contains(&format!("\nfile\t0\t{path}\n")));
    // Discovery order is deterministic across runs and both analysis views.
    assert_eq!(census("--emit-types", &input, "again", &[])?, text);
    assert_eq!(census("--emit-semantics", &input, "semantics", &[])?, text);
    ordinary_matches_first_record(&input, &text)
}

#[test]
fn failed_impl_headers_stay_item_local_and_block_their_members() -> Result {
    let input = input(
        "headers",
        "pub trait Tr { const C: u8; }\n\
         pub struct S;\n\
         impl<const N: usize> Tr for N { const C: u8 = 0; }\n\
         impl<const N: usize> N { const D: u8 = 1; }\n\
         impl Tr for S { const C: u8 = 2; }\n\
         pub const USE: u8 = <S as Tr>::C;\n\
         fn fine() {}\n",
    )?;
    let text = census("--emit-types", &input, "types", &[])?;
    let failed = "const parameter cannot be used as a type";
    // Neither the trait-impl inventory nor the inherent index turns the two
    // failed headers into a crate-level stop; their members are blocked.
    assert_eq!(
        records(&text),
        [
            ["declarations", "failed", "self-type `Self`", "-", failed],
            ["declarations", "failed", "self-type `Self`", "-", failed],
            ["declarations", "blocked", "const `C`", "1", BLOCKED],
            ["declarations", "blocked", "const `D`", "2", BLOCKED],
            [
                "declarations",
                "stopped",
                "-",
                "-",
                "later passes were not run after 2 failed and 2 blocked items in declarations"
            ],
        ]
    );
    ordinary_matches_first_record(&input, &text)
}

#[test]
fn an_opaque_failing_after_interning_leaves_later_storage_to_its_items() -> Result {
    let input = input(
        "interned-opaque",
        "pub trait Tr {}\n\
         impl Tr for u8 {}\n\
         pub fn declared() -> impl Tr { 0u8 }\n\
         pub fn failed() -> impl Tr + '_ { 0u8 }\n\
         pub const fn size<T>() -> usize { 4 }\n\
         pub struct S;\n\
         impl S { pub const fn bytes(self) -> [u8; size::<Self>()] { loop {} } }\n",
    )?;
    let text = census("--emit-types", &input, "types", &[])?;
    // The `size::<Self>()` length prepares storage after `failed` interned its
    // application; that application waits instead of failing `bytes`.
    assert_eq!(
        records(&text),
        [
            [
                "declarations",
                "failed",
                "function `failed`",
                "-",
                "cannot infer an output lifetime from these parameters"
            ],
            [
                "declarations",
                "stopped",
                "-",
                "-",
                "later passes were not run after 1 failed and 0 blocked items in declarations"
            ],
        ]
    );
    ordinary_matches_first_record(&input, &text)
}

#[test]
fn a_failed_header_blocks_later_uses_of_its_self_type() -> Result {
    let input = input(
        "header-self",
        "pub trait Tr { fn make() -> Self; }\n\
         impl<const N: usize> Tr for N { fn make() -> Self { loop {} } }\n",
    )?;
    let text = census("--emit-types", &input, "types", &[])?;
    // The failed header never assigns its self type a schema; the member's
    // `Self` is blocked by the header, not a later-assignment failure.
    assert_eq!(
        records(&text),
        [
            [
                "declarations",
                "failed",
                "self-type `Self`",
                "-",
                "const parameter cannot be used as a type"
            ],
            ["declarations", "blocked", "function `make`", "1", BLOCKED],
            [
                "declarations",
                "stopped",
                "-",
                "-",
                "later passes were not run after 1 failed and 1 blocked items in declarations"
            ],
        ]
    );
    ordinary_matches_first_record(&input, &text)
}

#[test]
fn a_const_body_left_active_by_a_failed_root_blocks_its_later_demands() -> Result {
    let input = input(
        "abandoned-body",
        "pub const fn f() -> usize { let x: u8 = 1u16; 1 }\n\
         pub fn a() -> [u8; f()] { loop {} }\n\
         pub fn b() -> [u8; f()] { loop {} }\n",
    )?;
    let text = census("--emit-types", &input, "types", &[])?;
    // `a`'s failure leaves `f`'s body active; `b` needs it and is blocked
    // instead of reporting a dependency cycle.
    assert_eq!(
        records(&text),
        [
            [
                "declarations",
                "failed",
                "function `a`",
                "-",
                "incompatible types"
            ],
            ["declarations", "blocked", "function `b`", "1", BLOCKED],
            [
                "declarations",
                "stopped",
                "-",
                "-",
                "later passes were not run after 1 failed and 1 blocked items in declarations"
            ],
        ]
    );
    ordinary_matches_first_record(&input, &text)
}

#[test]
fn body_failures_in_plain_and_hidden_type_schedules_are_all_reported() -> Result {
    let plain = input(
        "plain-bodies",
        "fn ok_a() -> u8 { 1 }\n\
         fn bad_a() -> u8 { true }\n\
         fn ok_b(x: u16) -> u16 { x }\n\
         fn bad_b() -> bool { 7u8 }\n\
         fn bad_c(x: u8) -> u16 { x }\n\
         const C: u8 = false;\n\
         const D: u8 = 3;\n",
    )?;
    let hidden = input(
        "hidden-bodies",
        "pub trait Tr {}\n\
         impl Tr for u8 {}\n\
         fn hidden() -> impl Tr { 1u8 }\n\
         fn bad_a() -> u8 { true }\n\
         fn ok_b(x: u16) -> u16 { x }\n\
         fn bad_b() -> bool { 7u8 }\n",
    )?;
    for (input, items, checked) in [
        (
            &plain,
            &[
                "function `bad_a`",
                "function `bad_b`",
                "function `bad_c`",
                "const `C`",
            ][..],
            3,
        ),
        (&hidden, &["function `bad_a`", "function `bad_b`"][..], 2),
    ] {
        let text = census("--emit-types", input, "types", &[])?;
        let mut expected: Vec<_> = items
            .iter()
            .map(|item| ["bodies", "failed", item, "-", "incompatible types"])
            .collect();
        let stopped = format!(
            "later passes were not run after {} failed and 0 blocked items in bodies",
            items.len()
        );
        expected.push(["bodies", "stopped", "-", "-", &stopped]);
        assert_eq!(records(&text), expected);
        assert!(text.contains(&format!(
            "\nsummary\tbodies\tchecked\t{checked}\tfailed\t{}\tblocked\t0\n",
            items.len()
        )));
        assert!(
            line(&text, "summary\tdeclarations\t")
                .is_some_and(|summary| summary.ends_with("\tfailed\t0\tblocked\t0"))
        );
        assert_eq!(census("--emit-semantics", input, "semantics", &[])?, text);
        ordinary_matches_first_record(input, &text)?;
    }
    Ok(())
}

#[test]
fn budget_exhaustion_is_sticky_and_ends_the_census() -> Result {
    let input = input(
        "budget",
        "#[lang=\"legacy_receiver\"]pub trait LegacyReceiver:PointeeSized{}\n\
         impl<T:PointeeSized> LegacyReceiver for &T{}\n\
         #[lang=\"clone\"]pub trait Clone:Sized{#[lang=\"clone_fn\"]fn clone(&self)->Self;}\n\
         #[lang=\"copy\"]pub trait Copy:Clone{}\n\
         impl Clone for usize{fn clone(&self)->usize{*self}}\n\
         impl Copy for usize{}\n\
         fn bad_a<const N: usize>(x: N) {}\n\
         const fn deep(n: usize) -> usize { deep(n) }\n\
         type Deep = [u8; deep(0)];\n\
         fn after<const M: usize>(x: M) {}\n",
    )?;
    let text = census("--emit-types", &input, "types", &[])?;
    assert_eq!(
        records(&text),
        [
            [
                "declarations",
                "failed",
                "function `bad_a`",
                "-",
                "const parameter cannot be used as a type"
            ],
            [
                "declarations",
                "budget",
                "-",
                "-",
                "constant execution frame limit exceeded"
            ],
        ]
    );
    assert!(text.contains("\nterminal\t2\tdeclarations\tbudget\n"));
    // A budget stop before analysis still records the sticky frontend failure.
    let early = census(
        "--emit-types",
        &input,
        "frontend",
        &["--source-contract-work-limit=0"],
    )?;
    assert_eq!(
        records(&early),
        [[
            "frontend",
            "budget",
            "-",
            "-",
            "source macro contracts exceed their work budget"
        ]]
    );
    assert!(early.contains("\nterminal\t1\tfrontend\tbudget\n"));
    ordinary_matches_first_record(&input, &text)
}

#[test]
fn later_crate_wide_passes_still_run_after_a_failed_pass_and_follow_it() -> Result {
    let later = "struct Recursive(Recursive);\nstruct Fine(u8);\n";
    let both = input(
        "lookahead",
        &format!("pub struct S; impl S {{ fn f() {{}} }} impl S {{ fn f() {{}} }}\n{later}"),
    )?;
    let infinite = "recursive type has infinite size";
    for mode in ["--emit-types", "--emit-semantics"] {
        let text = census(mode, &both, mode.trim_start_matches("--emit-"), &[])?;
        assert_eq!(
            records(&text),
            [
                [
                    "declarations",
                    "crate",
                    "pass `validate_inherent_overlaps`",
                    "-",
                    "duplicate inherent method in overlapping implementations"
                ],
                [
                    "declarations",
                    "lookahead",
                    "pass `complete_layouts`",
                    "1",
                    infinite
                ],
            ]
        );
        assert!(text.contains("\nsummary\tdeclarations\tchecked\t26\tfailed\t0\tblocked\t0\n"));
        assert!(text.contains("\nsummary\tbodies\tchecked\t0\tfailed\t0\tblocked\t0\n"));
        assert!(text.ends_with("terminal\t1\tdeclarations\tcrate\nend\t2\n"));
        ordinary_matches_first_record(&both, &text)?;
    }
    // Without the failed pass, ordinary checking reports the lookahead error.
    let alone = rustgm("--emit-types", &input("lookahead-alone", later)?, &[])?;
    assert_eq!(alone.status.code(), Some(1));
    assert!(String::from_utf8(alone.stderr)?.ends_with(&format!("{infinite}\n")));
    Ok(())
}

#[test]
fn a_census_of_a_passing_crate_still_fails_without_view_output() -> Result {
    let input = input("passing", "fn ok() -> u8 { 1 }\n")?;
    for mode in ["--emit-types", "--emit-semantics"] {
        let ordinary = rustgm(mode, &input, &[])?;
        assert!(ordinary.status.success());
        assert!(!ordinary.stdout.is_empty() && ordinary.stderr.is_empty());
        let text = census(mode, &input, mode.trim_start_matches("--emit-"), &[])?;
        assert!(records(&text).is_empty());
        assert!(text.contains("\nsummary\tbodies\tchecked\t1\tfailed\t0\tblocked\t0\n"));
        assert!(text.ends_with("terminal\tnone\nend\t0\n"));
    }
    Ok(())
}

#[test]
fn census_output_is_limited_to_source_core_analysis_views() -> Result {
    let input = input("usage", "fn ok() -> u8 { 1 }\n")?;
    let path = input.with_extension("usage.tsv");
    let path = path.to_str().ok_or("census path")?;
    let scoped = "--census-output requires --source-core with --emit-types or --emit-semantics";
    for (mode, args, message) in [
        ("--emit-lowir", &["--census-output", path][..], scoped),
        (
            "--emit-types",
            &["--census-output", path, "-o", path][..],
            scoped,
        ),
        (
            "--emit-types",
            &["--census-output", path, "--census-output", path][..],
            "duplicate --census-output",
        ),
    ] {
        let output = rustgm(mode, &input, args)?;
        assert_eq!(output.status.code(), Some(2));
        assert!(String::from_utf8(output.stderr)?.contains(message));
    }
    let unscoped = Command::new(env!("CARGO_BIN_EXE_rustgm"))
        .args(["--emit-types", "--census-output", path])
        .arg(&input)
        .output()?;
    assert_eq!(unscoped.status.code(), Some(2));
    assert!(String::from_utf8(unscoped.stderr)?.contains(scoped));
    assert!(
        !Path::new(path).exists(),
        "rejected options write no census"
    );
    Ok(())
}

/// Core's sized-trait prefix and lang `shl` trait, as census 20's reduced
/// record 6 repro carries them. This text is from Rust's `library/core`
/// (MIT/Apache-2.0; see the root NOTICE).
const CORE_SIZED: &str = r#"#![feature(no_core,lang_items,rustc_attrs,fundamental,pattern_types)]
#![no_core]
#![allow(internal_features,dead_code,unused_variables)]
#[lang = "pointee_sized"]
#[diagnostic::on_unimplemented(
    message = "values of type `{Self}` may or may not have a size",
    label = "may or may not have a known size"
)]
#[fundamental]
#[rustc_specialization_trait]
#[rustc_deny_explicit_impl]
#[rustc_coinductive]
pub trait PointeeSized {
    // Empty
}
#[lang = "meta_sized"]
#[diagnostic::on_unimplemented(
    message = "the size for values of type `{Self}` cannot be known",
    label = "doesn't have a known size"
)]
#[fundamental]
#[rustc_specialization_trait]
#[rustc_deny_explicit_impl]
// `MetaSized` being coinductive, despite having supertraits, is okay for the same reasons as
// `Sized` above.
#[rustc_coinductive]
pub trait MetaSized: PointeeSized {
    // Empty
}
#[lang = "sized"]
#[diagnostic::on_unimplemented(
    message = "the size for values of type `{Self}` cannot be known at compilation time",
    label = "doesn't have a size known at compile-time"
)]
#[fundamental] // for Default, for example, which requires that `[T]: !Default` be evaluatable
#[rustc_specialization_trait]
#[rustc_deny_explicit_impl]
#[rustc_dyn_incompatible_trait]
// `Sized` being coinductive, despite having supertraits, is okay as there are no user-written impls,
// and we know that the supertraits are always implemented if the subtrait is just by looking at
// the builtin impls.
#[rustc_coinductive]
pub trait Sized: MetaSized {
    // Empty.
}
"#;
const SHL: &str = "#[lang = \"shl\"]\npub trait Shl<Rhs = Self> {\n    type Output;\n    \
fn shl(self, rhs: Rhs) -> Self::Output;\n}\n";
/// A crate failure in impl validation that rustc also rejects (E0277).
const STOP: &str =
    "pub trait Super {}\npub trait Sub: Super {}\npub struct Stop;\nimpl Sub for Stop {}\n";
const ALIGNMENT: &str = "#[repr(usize)]\nenum AlignmentEnum {\n    _Align1Shl0 = 1 << 0,\n}\n";
const INTEGERS: [&str; 12] = [
    "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize",
];
const EDITION: &[&str] = &["--edition=2024"];
const PARTIAL: &str = "trait selection requires the completed immutable impl epoch";
const REQUIREMENTS: &str = "trait implementation does not satisfy the trait's requirements";

fn shl(subject: &str, rhs: &str) -> String {
    format!(
        "impl Shl<{rhs}> for {subject} {{ type Output = {subject}; \
         fn shl(self, other: {rhs}) -> {subject} {{ loop {{}} }} }}\n"
    )
}
/// The Shl impls, the stop and the 64-bit alignment discriminant in `items` order.
fn ordered(name: &str, items: &[&str]) -> Result<PathBuf> {
    source(
        name,
        &format!("{CORE_SIZED}{SHL}{}{ALIGNMENT}", items.concat()),
    )
}
fn every_shl_but_u8() -> String {
    INTEGERS
        .iter()
        .flat_map(|subject| INTEGERS.iter().map(move |rhs| (*subject, *rhs)))
        .filter(|&pair| pair != ("u8", "u8"))
        .map(|(subject, rhs)| shl(subject, rhs))
        .collect()
}
fn location(text: &str, record: usize) -> Option<&str> {
    line(text, &format!("record\t{record}\t")).and_then(|line| line.split('\t').nth(5))
}
fn declarations_summary(text: &str) -> Option<&str> {
    line(text, "summary\tdeclarations\t")
}

#[test]
fn a_lookahead_selection_over_a_partial_impl_epoch_is_blocked_and_counted() -> Result {
    let (u8, usize, rest) = (shl("u8", "u8"), shl("usize", "i32"), every_shl_but_u8());
    for (name, items) in [
        ("p-u8-first", [u8.as_str(), STOP, rest.as_str()]),
        ("p-u8-min", [u8.as_str(), STOP, usize.as_str()]),
        ("p-usize-first", [usize.as_str(), STOP, u8.as_str()]),
        ("p-none-first", [STOP, u8.as_str(), usize.as_str()]),
    ] {
        let input = ordered(name, &items)?;
        let source = fs::read_to_string(&input)?;
        let at = |needle: &str| source.find(needle).map(|offset| format!("file 0:{offset}"));
        for mode in ["--emit-types", "--emit-semantics"] {
            let text = census(mode, &input, mode.trim_start_matches("--emit-"), EDITION)?;
            assert_eq!(
                records(&text),
                [
                    [
                        "declarations",
                        "crate",
                        "pass `validate_trait_implementations`",
                        "-",
                        REQUIREMENTS
                    ],
                    [
                        "declarations",
                        "lookahead",
                        "pass `validate_trait_impl_owners`",
                        "1",
                        "trait implementation requires conformance and selection semantics"
                    ],
                    [
                        "declarations",
                        "blocked",
                        "pass `complete_discriminants`",
                        "1",
                        PARTIAL
                    ],
                    [
                        "declarations",
                        "lookahead",
                        "pass `complete_layouts`",
                        "1",
                        "enum layout requires completed discriminants"
                    ],
                ],
                "{name}"
            );
            let stop = at("impl Sub for Stop");
            assert_eq!(location(&text, 1).map(String::from), stop, "{name}");
            assert_eq!(location(&text, 2).map(String::from), stop, "{name}");
            assert_eq!(location(&text, 3).map(String::from), at("1 << 0"), "{name}");
            assert_eq!(
                location(&text, 4).map(String::from),
                at("enum AlignmentEnum"),
                "{name}"
            );
            let summary = declarations_summary(&text).ok_or("declarations summary")?;
            assert!(
                summary.ends_with("\tfailed\t0\tblocked\t1"),
                "{name}: {summary}"
            );
            ordinary_matches_first_record(&input, &text)?;
        }
    }
    Ok(())
}

#[test]
fn a_lookahead_selection_over_a_complete_matching_set_selects_as_ordinary_checking() -> Result {
    let (u8, usize, rest) = (shl("u8", "u8"), shl("usize", "i32"), every_shl_but_u8());
    let before = ordered("p-all-before", &[u8.as_str(), usize.as_str(), STOP])?;
    for mode in ["--emit-types", "--emit-semantics"] {
        let text = census(mode, &before, mode.trim_start_matches("--emit-"), EDITION)?;
        assert_eq!(
            records(&text),
            [
                [
                    "declarations",
                    "crate",
                    "pass `validate_trait_implementations`",
                    "-",
                    REQUIREMENTS
                ],
                [
                    "declarations",
                    "lookahead",
                    "pass `validate_trait_impl_owners`",
                    "1",
                    "trait implementation requires conformance and selection semantics"
                ],
            ]
        );
        let summary = declarations_summary(&text).ok_or("declarations summary")?;
        assert!(summary.ends_with("\tfailed\t0\tblocked\t0"), "{summary}");
        ordinary_matches_first_record(&before, &text)?;
    }
    for (name, items) in [
        ("ord-min", [u8.as_str(), usize.as_str()]),
        ("ord-full", [u8.as_str(), rest.as_str()]),
    ] {
        let input = ordered(name, &items)?;
        for mode in ["--emit-types", "--emit-semantics"] {
            let ordinary = rustgm(mode, &input, EDITION)?;
            assert!(ordinary.status.success(), "{name}");
            let text = census(mode, &input, mode.trim_start_matches("--emit-"), EDITION)?;
            assert!(records(&text).is_empty(), "{name}");
        }
    }
    Ok(())
}
