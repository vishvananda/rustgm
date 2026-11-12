//! A closure's computed `if let`/`while let` scrutinee is consumed like a
//! computed match's, as core's `read_radix_max_digits` loops over digits.
use std::fmt::Write as _;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa24/tests/spec/let-condition-captures";
/// Each rejection's reason; rustc reports E0382 and E0594.
const REJECTED: &[(&str, &str)] = &[
    (
        "while-let-moved.rs",
        "local may be uninitialized or moved at this read",
    ),
    (
        "if-let-fn-mutation.rs",
        "trait obligation requires an unambiguous matching assumption or implementation",
    ),
];

fn lower(source: &str) -> std::result::Result<(), String> {
    let (mut report, mut facts) = crate::semantic::analyze(source)?;
    let entry = rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
    rustgm::lower::instances::build(&report, &mut facts, entry)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[test]
fn let_condition_captures_lower_like_rustc() -> Result {
    let mut failures = Vec::new();
    let mut count = 0;
    for line in crate::resolve::fixture(&format!("{PACKET}/cases.tsv"))?.lines() {
        let (path, expected) = line
            .split_once('\t')
            .ok_or("let-condition capture outcome columns")?;
        let accepted = match expected {
            "0" => true,
            "1" => false,
            _ => return Err("invalid let-condition capture outcome".into()),
        };
        let name = path.rsplit('/').next().ok_or("fixture name")?;
        let reason = REJECTED
            .iter()
            .find(|(case, _)| *case == name)
            .map(|(_, reason)| *reason);
        if accepted == reason.is_some() {
            return Err(format!("{name}: undeclared rejection reason").into());
        }
        let source = crate::resolve::fixture(path)?;
        crate::qualify::qualify(&format!("let-condition-{name}"), &source, accepted)?;
        let result = lower(&source);
        let matches = match (&result, reason) {
            (Ok(()), None) => true,
            (Err(error), Some(reason)) => error.contains(reason),
            _ => false,
        };
        if !matches {
            failures.push(format!("{path}: {:?}", result.err()));
        }
        count += 1;
    }
    assert_eq!(count, 7);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

/// A scrutinee that is itself a captured place is used as its pattern
/// requires (`docs/pa31-closure-capture-patterns.md`), so valid Rust lowers.
#[test]
fn captured_place_let_scrutinees_lower() -> Result {
    for (name, source) in [
        (
            "if-let-captured-place",
            "fn main(){let o=Some(1u8);let f=||if let Some(v)=o{v}else{0};let _=f();}",
        ),
        (
            "while-let-captured-place",
            "fn main(){let mut o=Some(1u8);let mut f=||{let mut t=0;while let Some(v)=o{t+=v;o=None;}t};let _=f();}",
        ),
    ] {
        crate::qualify::qualify(name, source, true)?;
        if let Err(error) = lower(source) {
            return Err(format!("{name}: {error}").into());
        }
    }
    Ok(())
}

const PREFIX: &str = "struct It{n:u32}\n\
impl It{fn next(&mut self)->Option<u32>{if self.n==0{None}else{self.n-=1;Some(self.n)}}}\n";

/// `uses` closures each consuming a computed `while let` scrutinee beside a
/// copied capture, then unrelated non-closure `if let` functions.
fn width(uses: usize, noise: bool) -> std::result::Result<String, std::fmt::Error> {
    let mut text = PREFIX.to_owned();
    for index in 0..uses {
        writeln!(
            text,
            "fn p{index}(it:&mut It,k:u32)->u32{{let f=|it:&mut It|{{let mut t=0;while let Some(d)=it.next(){{t+=d+k;}}t}};f(it)}}"
        )?;
    }
    for index in 0..if noise { 64 } else { 0 } {
        writeln!(
            text,
            "fn q{index}(it:&mut It)->u32{{let mut t=0;if let Some(d)=it.next(){{t+=d;}}t}}"
        )?;
    }
    text.push_str("fn main(){}\n");
    Ok(text)
}

#[test]
fn let_condition_captures_track_closures_without_scanning_unrelated_bodies() -> Result {
    let mut previous = None;
    let mut unrelated = None;
    for uses in [8, 32, 128] {
        let mut ordinary = [0; 3];
        for noise in [false, true] {
            let (_, facts) = crate::semantic::analyze(&width(uses, noise)?)?;
            let work = [
                facts.body_work.inference_steps,
                facts.body_work.canonical_steps,
                facts.body_work.trait_selections,
            ];
            assert_eq!(facts.closures.all().len(), uses);
            println!("let-condition captures uses={uses} noise={noise} work={work:?}");
            if noise {
                // The unrelated bodies check themselves once; no closure's
                // capture walk visits them.
                let added: Vec<_> = std::iter::zip(ordinary, work)
                    .map(|(old, actual)| actual.checked_sub(old))
                    .collect();
                assert_eq!(*unrelated.get_or_insert_with(|| added.clone()), added);
            } else {
                if let Some(before) = previous {
                    for (old, actual) in std::iter::zip(before, work) {
                        assert!(actual <= 4 * old + 512, "uses={uses}: {before:?}->{work:?}");
                    }
                }
                previous = Some(work);
                ordinary = work;
            }
        }
    }
    Ok(())
}
