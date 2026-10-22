//! Observe actual region visits, including failed demanded annotation preparation.
use super::*;
use std::{cell::Cell, fmt::Write};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Clone, Copy, Default, Debug)]
struct Counts {
    visits: usize,
    own: usize,
    produced: usize,
    live_regions: usize,
    peak_regions: usize,
    region_capacity: usize,
    peak_region_capacity: usize,
    owners: usize,
    owner_capacity: usize,
}

thread_local! {
    static VISITS: Cell<Counts> = const { Cell::new(Counts {
        visits: 0, own: 0, produced: 0, live_regions: 0, peak_regions: 0,
        region_capacity: 0, peak_region_capacity: 0, owners: 0, owner_capacity: 0,
    }) };
}

pub(super) fn visit(owned: bool) {
    VISITS.with(|counts| {
        let mut value = counts.get();
        value.visits += 1;
        value.own += usize::from(owned);
        counts.set(value);
    });
}

pub(super) fn opened(owners: usize, capacity: usize) {
    VISITS.with(|counts| {
        let mut value = counts.get();
        value.owners = value.owners.max(owners);
        value.owner_capacity = value.owner_capacity.max(capacity);
        counts.set(value);
    });
}

pub(super) fn produced(capacity: usize) {
    VISITS.with(|counts| {
        let mut value = counts.get();
        value.produced += 1;
        value.live_regions += 1;
        value.peak_regions = value.peak_regions.max(value.live_regions);
        value.region_capacity += capacity;
        value.peak_region_capacity = value.peak_region_capacity.max(value.region_capacity);
        counts.set(value);
    });
}

pub(super) fn released(regions: usize, capacity: usize) {
    VISITS.with(|counts| {
        let mut value = counts.get();
        value.live_regions -= regions;
        value.region_capacity -= capacity;
        counts.set(value);
    });
}

fn chain(depth: usize, foreign: usize, invalid: bool) -> Result<String> {
    let mut text = String::from("const N:usize=f0();\n");
    for i in 0..depth {
        let next = if i + 1 == depth {
            "1".into()
        } else {
            format!("f{}()", i + 1)
        };
        write!(text, "const fn f{i}()->usize{{")?;
        for j in 0..foreign {
            write!(text, "let _r{j}:&u8=&0;")?;
        }
        let bad = if invalid && i + 1 == depth {
            "let _bad:u8=true;"
        } else {
            ""
        };
        writeln!(text, "let _c=|_x:&[u8;{next}]|{{}};{bad}1}}")?;
    }
    if invalid {
        text += "fn main(){}\n";
    } else {
        text += "#[unsafe(no_mangle)]pub extern \"C\" fn result()->usize{N}fn main(){}\n";
    }
    Ok(text)
}

fn observe(text: &str) -> Result<(bool, usize, Counts)> {
    let report = crate::test_source::resolve(text, true)?;
    let mut checker = Checker::new(&report).map_err(|e| e.to_string())?;
    checker
        .facts
        .set_constant_executor(crate::pipeline::execution::Context::default().executor());
    VISITS.with(|counts| counts.set(Counts::default()));
    let accepted = checker.run().is_ok();
    let counts = VISITS.with(Cell::get);
    Ok((accepted, checker.facts.body_regions.log().len(), counts))
}

#[test]
fn deferred_annotation_scan_observations() -> Result {
    for depth in [8, 32, 128] {
        for foreign in [0, 3] {
            let (accepted, regions, counts) = observe(&chain(depth, foreign, false)?)?;
            assert!(accepted);
            assert_eq!(counts.visits, depth);
            assert_eq!(counts.own, depth);
            assert_eq!(counts.produced, depth);
            assert_eq!(counts.live_regions, 0);
            println!("retained depth={depth} foreign={foreign}: {counts:?}");
            println!(
                "depth={depth} foreign={foreign} accepted={accepted} regions={regions} visits={} own={}",
                counts.visits, counts.own
            );
        }
        let (accepted, regions, counts) = observe(&chain(depth, 0, true)?)?;
        assert!(!accepted);
        assert_eq!(counts.visits, 1);
        assert_eq!(counts.produced, depth);
        println!("retained invalid depth={depth}: {counts:?}");
        println!(
            "depth={depth} invalid accepted={accepted} regions={regions} visits={} own={}",
            counts.visits, counts.own
        );
        let mut text = String::from("const N:usize=f();\nconst fn f()->usize{");
        for i in 0..depth {
            write!(text, "let _c{i}=|_x:&[u8;one()]|{{}};")?;
        }
        text += "1}\nconst fn one()->usize{1}\n#[unsafe(no_mangle)]pub extern \"C\" fn result()->usize{N}fn main(){}\n";
        let (accepted, regions, counts) = observe(&text)?;
        assert!(accepted);
        assert_eq!(counts.visits, depth);
        assert_eq!(counts.own, depth);
        assert_eq!(counts.produced, depth);
        assert_eq!(counts.owners, 1);
        println!("retained width={depth}: {counts:?}");
        println!(
            "width={depth} accepted={accepted} regions={regions} visits={} own={}",
            counts.visits, counts.own
        );
    }
    Ok(())
}
