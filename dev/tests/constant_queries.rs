use rustgm::{
    expand::config::Config,
    resolve::{self, expansion::Report, load},
    sema::{
        bodies, declarations,
        model::NodeFacts,
        patterns::{Interval, scalar::Coverage},
    },
    support::source::SourceFile,
};
use std::{fmt::Write, path::Path};

struct InMemory;
impl load::Sources for InMemory {
    fn read(&mut self, _: &Path) -> Result<Option<load::InputFile>, String> {
        Ok(None)
    }
}
fn resolve(source: &str) -> Result<Report, String> {
    let input = load::InputFile {
        identity: "constants.rs".into(),
        path: "constants.rs".into(),
        source: SourceFile::new(source.as_bytes().to_vec())?,
    };
    resolve::expansion::analyze(input, &mut InMemory, &Config::default())
        .and_then(resolve::analyze_expanded)
        .map_err(|e| e.to_string())
}
#[test]
fn typed_dependencies_survive_discriminants_and_pattern_uses() -> Result<(), String> {
    let report = resolve(
        "const BASE:isize=3;const N:isize=BASE+1;const UNUSED:isize=9;\
         enum E{A=N,B}const V:E=E::B;const LAST:isize=V as isize;\
         fn f(x:isize){match x{N=>(),LAST=>(),N=>(),_=>()}}\
         fn g(x:isize){match x{N..=LAST=>(),_=>()}}",
    )?;
    let facts = declarations::analyze(&report).map_err(|e| e.to_string())?;
    assert_eq!(
        facts
            .initializers
            .iter()
            .filter(|i| facts.node(i.value).is_some_and(|n| n.ty.is_some()))
            .count(),
        2
    );
    let facts = bodies::analyze(&report, facts).map_err(|e| e.to_string())?;
    assert!(
        facts
            .initializers
            .iter()
            .all(|i| facts.node(i.value).is_some_and(|n| n.ty.is_some()))
    );
    assert_eq!(facts.patterns.decisions.len(), 2);
    assert_eq!(
        facts
            .patterns
            .arms
            .iter()
            .map(|a| a.useful)
            .collect::<Vec<_>>(),
        [true, true, false, true, true, true]
    );
    let sign = 1 << 63;
    assert_eq!(
        facts.patterns.ranges,
        [
            Interval {
                lower: sign + 4,
                upper: sign + 4
            },
            Interval {
                lower: sign + 5,
                upper: sign + 5
            },
            Interval {
                lower: sign + 4,
                upper: sign + 4
            },
            Interval {
                lower: sign + 4,
                upper: sign + 5
            },
        ]
    );
    assert_eq!(std::mem::size_of::<NodeFacts>(), 80);
    Ok(())
}
#[test]
fn repeated_and_distinct_demands_finish_with_linear_retained_facts()
-> Result<(), Box<dyn std::error::Error>> {
    for count in [16, 256, 4096] {
        let mut source = String::from("const ROOT:u64=0;");
        for i in 0..count {
            write!(source, "const N{i}:u64=")?;
            if i == 0 {
                source.push_str("ROOT+1;");
            } else {
                write!(source, "N{}+1;", i - 1)?;
            }
            write!(
                source,
                "fn f{i}(x:u64){{match x{{N{i}=>(),ROOT=>(),_=>()}}}}"
            )?;
        }
        let report = resolve(&source)?;
        let facts = declarations::analyze(&report)?;
        let facts = bodies::analyze(&report, facts)?;
        assert_eq!(facts.patterns.decisions.len(), count);
        assert_eq!(facts.patterns.ranges.len(), count * 2);
        assert_eq!(facts.patterns.arms.len(), count * 3);
        assert!(facts.patterns.arms.iter().all(|a| a.useful));
        assert!(facts.patterns.decisions.iter().all(|d| d.work <= 24));
    }
    Ok(())
}
#[test]
fn deep_forward_demands_use_worklists() -> Result<(), Box<dyn std::error::Error>> {
    let count = 8192_u32;
    let mut source = String::new();
    for i in 0..count {
        writeln!(source, "const N{i}:u64=N{}+1;", i + 1)?;
    }
    writeln!(
        source,
        "const N{count}:u64=0;fn f(x:u64){{match x{{N0=>(),_=>()}}}}"
    )?;
    let report = resolve(&source)?;
    let facts = declarations::analyze(&report)?;
    let facts = bodies::analyze(&report, facts)?;
    assert_eq!(
        facts.patterns.ranges,
        [Interval {
            lower: u128::from(count),
            upper: u128::from(count)
        }]
    );
    assert!(facts.patterns.decisions[0].work < Coverage::LIMIT);
    Ok(())
}
