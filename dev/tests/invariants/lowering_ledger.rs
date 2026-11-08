//! Lowering pays its own input-proportional account for originals-sized
//! storage and work, so many small bodies lower; demanded bodies pay the fixed
//! demand budget by their own output size, in work and bytes; superlinear
//! lowering retention is still rejected by the account, and superlinear
//! instantiation by the cap.
use crate::lower::instances::demand::{demand_costs, forge_instance_charges, preload_demand};
use crate::lower::instances::{BODIES_EXCEEDED, BODY_BYTES};
use crate::sema::header_attributes::{
    lowering_costs, source_contract_bound, source_contract_costs,
};
use std::fmt::Write;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
pub(super) type Item = fn(&mut String, usize) -> std::fmt::Result;

const WORK: &str = "pa31/tests/spec/needs-drop/work";
const BASE: &str = "pa31/tests/spec/needs-drop/work/width8-calls8.rs";
const ROOT: &str = "struct Root(Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf);\n";
const FEATURES: &str = "const_trait_impl)]";
const REJECTED: &str = "lowering work or retention exceeds its input-graph bound";

/// The `needs_drop` scaling base with `count` numbered items after its root
/// nominal, where the declared noise inputs place their unrelated nominals.
pub(super) fn inserted(count: usize, item: Item) -> Result<String> {
    let base = crate::test_source::fixture(BASE)?;
    let at = base.find(ROOT).ok_or("scaling root")? + ROOT.len();
    let mut source = base[..at].to_owned();
    for index in 0..count {
        item(&mut source, index)?;
    }
    source.push_str(&base[at..]);
    Ok(source)
}
pub(super) fn noise(source: &mut String, index: usize) -> std::fmt::Result {
    writeln!(source, "struct Noise{index}([u8;{}]);", index + 1)
}
fn trivial(source: &mut String, index: usize) -> std::fmt::Result {
    writeln!(source, "fn f{index}(){{}}")
}
fn constant(source: &mut String, index: usize) -> std::fmt::Result {
    writeln!(source, "const C{index}:u8=0;")
}
fn unnamed(source: &mut String, _: usize) -> std::fmt::Result {
    writeln!(source, "const _:u8=0;")
}
/// Statics need the `Sync` marker, added once with its feature.
fn shared(source: &mut String, index: usize) -> std::fmt::Result {
    if index == 0 {
        *source = source.replacen(FEATURES, "const_trait_impl,auto_traits)]", 1);
        source.push_str("#[lang=\"sync\"]pub unsafe auto trait Sync{}\n");
    }
    writeln!(source, "static S{index}:u8=0;")
}
fn dropped(source: &mut String, index: usize) -> std::fmt::Result {
    writeln!(source, "fn d{index}(){{let _l=Leaf;}}")
}
fn constructed(source: &mut String, index: usize) -> std::fmt::Result {
    writeln!(
        source,
        "struct D{index}(Leaf);#[unsafe(no_mangle)]pub extern \"C\" fn e{index}(){{let _d=D{index}(Leaf);}}"
    )
}
fn evaluated(source: &mut String, index: usize) -> std::fmt::Result {
    writeln!(source, "const fn k{index}()->u8{{0}}")
}
/// One shared generic function, instantiated once per item.
fn shared_instance(source: &mut String, index: usize) -> std::fmt::Result {
    if index == 0 {
        source.push_str("fn g<T>(_t:T){}\n");
    }
    writeln!(
        source,
        "struct S{index};#[unsafe(no_mangle)]pub extern \"C\" fn c{index}(){{g(S{index})}}"
    )
}
/// The same instances, each demanded twice.
fn revisited_instance(source: &mut String, index: usize) -> std::fmt::Result {
    if index == 0 {
        source.push_str("fn g<T>(_t:T){}\n");
    }
    writeln!(
        source,
        "struct S{index};#[unsafe(no_mangle)]pub extern \"C\" fn c{index}(){{g(S{index});g(S{index})}}"
    )
}
/// One instance after every item's trivial function.
fn lone_instance(source: &mut String, index: usize) -> std::fmt::Result {
    if index == 0 {
        source.push_str(
            "fn g<T>(_t:T){}struct S;#[unsafe(no_mangle)]pub extern \"C\" fn c(){g(S)}\n",
        );
    }
    trivial(source, index)
}
fn own_instance(source: &mut String, index: usize) -> std::fmt::Result {
    writeln!(
        source,
        "struct S{index};fn g{index}<T>(_t:T){{}}#[unsafe(no_mangle)]pub extern \"C\" fn c{index}(){{g{index}(S{index})}}"
    )
}
/// An instance whose body calls a function, so its emission asks a header
/// query.
fn calling_instance(source: &mut String, index: usize) -> std::fmt::Result {
    if index == 0 {
        source.push_str("fn h(){}\n");
    }
    writeln!(
        source,
        "struct S{index};fn g{index}<T>(_t:T){{h()}}#[unsafe(no_mangle)]pub extern \"C\" fn c{index}(){{g{index}(S{index})}}"
    )
}
fn drop_instance(source: &mut String, index: usize) -> std::fmt::Result {
    writeln!(
        source,
        "struct W{index}<T>(T);fn h{index}<T>(_t:T){{}}#[unsafe(no_mangle)]pub extern \"C\" fn q{index}(){{h{index}(W{index}(Leaf))}}"
    )
}
/// Generic function `k` forwards to `k - 1`, and root `k` instantiates it
/// with its own nominal: `n` items demand n(n+1)/2 instances.
fn chain(source: &mut String, index: usize) -> std::fmt::Result {
    if index == 0 {
        source.push_str("fn g0<T>(){}\n");
    } else {
        writeln!(source, "fn g{index}<T>(){{g{}::<T>()}}", index - 1)?;
    }
    writeln!(
        source,
        "struct S{index};#[unsafe(no_mangle)]pub extern \"C\" fn c{index}(){{g{index}::<S{index}>()}}"
    )
}

/// Both ledger accounts and the demand budget after lowering every root.
#[derive(Clone, Copy, Debug)]
struct Costs {
    /// The header account's work and byte bound.
    header_bound: (usize, usize),
    /// The header account's steps and bytes after analysis.
    analysis: (usize, usize),
    /// The header account's steps and bytes after lowering.
    header: (usize, usize),
    /// Lowering's steps, bytes, work bound and byte bound.
    lowering: [usize; 4],
    /// The demand budget's bytes, when instantiation ran.
    demand: usize,
    /// The work demand scopes paid the demand budget.
    demand_work: usize,
    /// Emitted symbols.
    symbols: usize,
}
fn lowered(source: &str) -> Result<Costs> {
    lowered_from(source, 0)
}
/// Lower with the demand budget starting at `preload` bytes.
fn lowered_from(source: &str, preload: usize) -> Result<Costs> {
    let report = crate::test_source::source_provider(source, "needs_drop_work")?;
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(&report)?;
    let header_bound = source_contract_bound(&facts);
    let analysis = source_contract_costs(&facts);
    preload_demand(preload);
    let program = crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    );
    let [demand, demand_work] = demand_costs();
    preload_demand(0);
    Ok(Costs {
        header_bound,
        analysis,
        symbols: program?.symbols.len(),
        header: source_contract_costs(&facts),
        lowering: lowering_costs(&facts).ok_or("open lowering account")?,
        demand,
        demand_work,
    })
}
/// Costs at 256 and 1,024 items, after checking that lowering paid at most
/// half of the bytes and work its bound grew by, and that the header account
/// grew by less than its own bound did.
fn scaled(name: &str, item: Item) -> Result<[Costs; 2]> {
    let small = lowered(&inserted(256, item)?)?;
    let large = lowered(&inserted(1024, item)?)?;
    let grew = |index: usize| large.lowering[index] - small.lowering[index];
    eprintln!(
        "{name}: per item lowering {} B {} W of {} B {} W granted; demand {} B; header {} B {} W of {} B {} W",
        grew(1) / 768,
        grew(0) / 768,
        grew(3) / 768,
        grew(2) / 768,
        (large.demand - small.demand) / 768,
        (large.header.1 - small.header.1) / 768,
        (large.header.0 - small.header.0) / 768,
        (large.header_bound.1 - small.header_bound.1) / 768,
        (large.header_bound.0 - small.header_bound.0) / 768,
    );
    assert!(2 * grew(1) <= grew(3), "{name} bytes {small:?} {large:?}");
    assert!(2 * grew(0) <= grew(2), "{name} work {small:?} {large:?}");
    assert!(large.header.1 - small.header.1 < large.header_bound.1 - small.header_bound.1);
    assert!(large.header.0 - small.header.0 < large.header_bound.0 - small.header_bound.0);
    for costs in [small, large] {
        assert!(
            2 * costs.lowering[1] <= costs.lowering[3],
            "{name} {costs:?}"
        );
        assert!(costs.header.1 < costs.header_bound.1, "{name} {costs:?}");
        assert!(costs.header.0 < costs.header_bound.0, "{name} {costs:?}");
    }
    Ok([small, large])
}

#[test]
fn declared_scaling_inputs_lower_inside_both_accounts() -> Result {
    // The native scaling test lowers these same inputs; this private control
    // keeps both accounts' margins visible without the backend.
    for count in [64, 1024] {
        let declared =
            crate::test_source::fixture(&format!("{WORK}/width8-calls8-noise{count}.rs"))?;
        assert_eq!(inserted(count, noise)?, declared, "{count}");
    }
    let mut names = Vec::new();
    for width in [8, 32, 128] {
        for calls in [8, 32, 128] {
            names.push(format!("width{width}-calls{calls}"));
        }
    }
    names.extend(
        [
            "width8-calls8-noise64",
            "width8-calls8-noise1024",
            "ordinary-unused-intrinsic",
            "primitive-calls128",
        ]
        .map(str::to_owned),
    );
    for name in names {
        let costs = lowered(&crate::test_source::fixture(&format!("{WORK}/{name}.rs"))?)?;
        eprintln!(
            "declared {name}: header {}% lowering {}% of bytes",
            costs.header.1 * 100 / costs.header_bound.1,
            costs.lowering[1] * 100 / costs.lowering[3]
        );
        // Lowering charges its own account only.
        assert_eq!(costs.header, costs.analysis, "{name}");
        assert!(costs.header.1 < costs.header_bound.1, "{name}: {costs:?}");
        assert!(
            2 * costs.lowering[1] <= costs.lowering[3],
            "{name}: {costs:?}"
        );
        assert!(
            2 * costs.lowering[0] <= costs.lowering[2],
            "{name}: {costs:?}"
        );
    }
    Ok(())
}

#[test]
fn ordinary_shapes_pay_at_most_half_of_what_their_inputs_grant() -> Result {
    for (name, item) in [
        ("trivial fn", trivial as Item),
        ("literal const", constant),
        ("unnamed const", unnamed),
        ("static", shared),
        ("anonymous-constant noise", noise),
        ("Drop local", dropped),
        ("const fn", evaluated),
    ] {
        let [small, large] = scaled(name, item)?;
        // None of these demands a body.
        assert_eq!(small.demand, large.demand, "{name}");
    }
    // Each constructed nominal demands its own cleanup, which pays the
    // demand budget about the same per item at either size.
    let base = lowered(&inserted(0, constructed)?)?;
    let [small, large] = scaled("constructor body", constructed)?;
    assert_eq!(
        demanded_per_item("constructor body", [base, small, large]),
        0
    );
    Ok(())
}
/// Demand bytes per item from 0 to 256 and from 256 to 1,024 items agree
/// within the factor of two that geometric vector growth allows.
fn demanded_per_item(name: &str, [base, small, large]: [Costs; 3]) -> usize {
    let first = (small.demand - base.demand) / 256;
    let added = (large.demand - small.demand) / 768;
    assert!(
        first > 0 && added <= 2 * first && first <= 2 * added,
        "{name} {base:?} {small:?} {large:?}"
    );
    // Their scoped work has no growth term: each item pays the same.
    let first = (small.demand_work - base.demand_work) / 256;
    let added = (large.demand_work - small.demand_work) / 768;
    eprintln!("{name}: demand work per item {first} then {added}");
    assert_eq!(first, added, "{name} {base:?} {small:?} {large:?}");
    first
}

#[test]
fn trivial_functions_with_drop_rows_lower_beyond_the_shared_header_bound() -> Result {
    // 256 such functions failed when lowering paid the header account. At
    // lowering's 768 bytes per body, 1,024 of them need more than the header
    // bound grants header analysis and lowering together.
    let few = lowered(&inserted(256, trivial)?)?;
    let many = lowered(&inserted(1024, trivial)?)?;
    assert!(
        many.analysis.1 + many.lowering[1] > many.header_bound.1,
        "{many:?}"
    );
    for costs in [few, many] {
        assert!(2 * costs.lowering[1] <= costs.lowering[3], "{costs:?}");
        assert!(costs.header.1 < costs.header_bound.1, "{costs:?}");
    }
    Ok(())
}

#[test]
fn demanded_instances_pay_the_demand_budget_once_by_their_own_size() -> Result {
    for (name, item, queries) in [
        ("shared generic", shared_instance as Item, false),
        ("own generic", own_instance, false),
        ("generic Drop instance", drop_instance, false),
        ("calling generic", calling_instance, true),
    ] {
        // Every item demands the same instance shape, so each pays the same
        // demand bytes up to the geometric growth of the proof and row
        // vectors, and the same demand work; `scaled` checks that lowering's
        // account still follows only the originals.
        let base = lowered(&inserted(0, item)?)?;
        let [small, large] = scaled(name, item)?;
        let work = demanded_per_item(name, [base, small, large]);
        assert_eq!(work > 0, queries, "{name}");
    }
    // The instance's growth is its own: one instance after 256 or 1,024
    // originals pays the same demand bytes, and never the doubling of the
    // originals' exact proof reservation.
    let few = lowered(&inserted(256, lone_instance)?)?;
    let many = lowered(&inserted(1024, lone_instance)?)?;
    eprintln!("lone instance: {few:?} {many:?}");
    assert_eq!(few.demand, many.demand, "{few:?} {many:?}");
    assert_eq!(few.demand_work, many.demand_work, "{few:?} {many:?}");
    assert!(few.demand < 256 * size_of::<crate::borrow::incremental::Proved>());
    // A revisited instance is reused: it adds no proof, symbol or row, only
    // its demand's work entry.
    let base = lowered(&inserted(0, shared_instance)?)?;
    let once = lowered(&inserted(256, shared_instance)?)?;
    let twice = lowered(&inserted(256, revisited_instance)?)?;
    eprintln!("revisit: once {once:?} twice {twice:?}");
    assert_eq!(twice.symbols, once.symbols);
    assert!(
        twice.demand.abs_diff(once.demand) < (once.demand - base.demand) / 256,
        "{once:?} {twice:?}"
    );
    assert_eq!(twice.demand_work, once.demand_work, "{once:?} {twice:?}");
    Ok(())
}

/// How much one cost grew between two lowerings.
fn grew(from: Costs, to: Costs, cost: impl Fn(Costs) -> usize) -> usize {
    cost(to) - cost(from)
}
#[test]
fn superlinear_instantiation_lowers_until_the_fixed_demand_cap() -> Result {
    let base = lowered(&inserted(0, chain)?)?;
    let small = lowered(&inserted(32, chain)?)?;
    let large = lowered(&inserted(64, chain)?)?;
    let largest = lowered(&inserted(128, chain)?)?;
    eprintln!("chain 0 {base:?} 32 {small:?} 64 {large:?} 128 {largest:?}");
    // n roots and n(n+1)/2 instances: the output is quadratic in the input.
    for (count, costs) in [(32, small), (64, large), (128, largest)] {
        assert_eq!(
            costs.symbols - base.symbols,
            count + count * (count + 1) / 2
        );
        // Lowering's account stays inside half of its bound in work and bytes.
        assert!(2 * costs.lowering[1] <= costs.lowering[3], "{costs:?}");
        assert!(2 * costs.lowering[0] <= costs.lowering[2], "{costs:?}");
    }
    // The demand budget grows with the instances: about four times as much
    // from 64 to 128 items as from 32 to 64, in bytes and in work.
    let demands: [fn(Costs) -> usize; 2] = [|c| c.demand, |c| c.demand_work];
    for cost in demands {
        assert!(
            grew(large, largest, cost) > 3 * grew(small, large, cost),
            "{small:?} {large:?} {largest:?}"
        );
    }
    // Lowering's account grows only with the originals, in work and bytes:
    // about twice as much from 64 to 128 items as from 32 to 64.
    for index in [0, 1] {
        let cost = |costs: Costs| costs.lowering[index];
        assert!(
            grew(base, small, cost) <= grew(small, large, cost)
                && grew(large, largest, cost) < 3 * grew(small, large, cost),
            "{index} {base:?} {small:?} {large:?} {largest:?}"
        );
    }
    eprintln!(
        "chain work share 32 {}% 64 {}% 128 {}%",
        100 * small.lowering[0] / small.lowering[2],
        100 * large.lowering[0] / large.lowering[2],
        100 * largest.lowering[0] / largest.lowering[2]
    );
    // With the fixed cap moved to twice the 32-item chain's demand, that
    // chain still lowers and the 64-item chain is rejected by the demand
    // budget's own message.
    let preload = BODY_BYTES - 2 * small.demand;
    lowered_from(&inserted(32, chain)?, preload)?;
    let rejected = lowered_from(&inserted(64, chain)?, preload)
        .err()
        .ok_or("chain lowered past the demand cap")?
        .to_string();
    eprintln!("chain cap: {rejected}");
    assert!(rejected.contains(BODIES_EXCEEDED), "{rejected}");
    assert!(!rejected.contains(REJECTED), "{rejected}");
    Ok(())
}

#[test]
fn forged_instance_charges_pay_only_the_demand_budget() -> Result {
    // Every specialized instance of the 64-item chain also charges 64 work
    // units and 1,000 bytes through its caller-input payer. Unscoped, that
    // work alone would exceed lowering's work bound.
    let instances = 64 * 65 / 2;
    let (work, bytes) = (64, 1000);
    let plain = lowered(&inserted(64, chain)?)?;
    assert!(work * instances > plain.lowering[2], "{plain:?}");
    let source = inserted(64, chain)?;
    forge_instance_charges(work, bytes);
    let forged = lowered(&source);
    forge_instance_charges(0, 0);
    let forged = forged?;
    eprintln!("forged {plain:?} {forged:?}");
    // Lowering's account is exactly what it was; the demand budget pays the
    // forged work and bytes once per instance.
    assert_eq!(forged.lowering, plain.lowering);
    assert_eq!(forged.header, plain.header);
    // Each forged charge also pays one unit to authenticate its payer.
    assert_eq!(
        forged.demand_work - plain.demand_work,
        (work + 1) * instances
    );
    assert_eq!(forged.demand - plain.demand, bytes * instances);
    // With the cap moved to between the plain and forged demand, the plain
    // chain lowers and the forged one is rejected by the demand budget only.
    let preload = BODY_BYTES - plain.demand - bytes * instances / 2;
    lowered_from(&source, preload)?;
    forge_instance_charges(work, bytes);
    let rejected = lowered_from(&source, preload);
    forge_instance_charges(0, 0);
    let rejected = rejected
        .err()
        .ok_or("forged chain lowered past the demand cap")?
        .to_string();
    assert!(rejected.contains(BODIES_EXCEEDED), "{rejected}");
    assert!(!rejected.contains(REJECTED), "{rejected}");
    Ok(())
}

#[test]
fn superlinear_lowering_charges_are_rejected_once_inputs_grow() -> Result {
    // A forged payer retaining work and bytes for every pair of n bodies, a
    // quarter of the 256-function account at 256 through each entry, fits
    // there and fails at 1,024: the account grows linearly with the input.
    let open = |count: usize| -> Result<(crate::sema::model::Declarations, [usize; 4])> {
        let report =
            crate::test_source::source_provider(&inserted(count, trivial)?, "needs_drop_work")?;
        let mut facts = crate::pipeline::analyze_source(&report)?.facts;
        facts.open_lowering_account(&report)?;
        let costs = lowering_costs(&facts).ok_or("open lowering account")?;
        Ok((facts, costs))
    };
    let (small, granted) = open(256)?;
    let forged = |granted: usize, n: usize| granted / 4 * n / 256 * n / 256;
    let header = source_contract_costs(&small);
    small.charge_caller_inputs(forged(granted[2], 256), forged(granted[3], 256))?;
    small.pay_lowering_growth(forged(granted[2], 256), forged(granted[3], 256))?;
    assert_eq!(source_contract_costs(&small), header);
    for (work, bytes) in [(granted[2], 0), (0, granted[3])] {
        let (large, _) = open(1024)?;
        let header = source_contract_costs(&large);
        assert_eq!(
            large.pay_lowering_growth(forged(work, 1024), forged(bytes, 1024)),
            Err(REJECTED)
        );
        assert_eq!(source_contract_costs(&large), header);
        let (large, _) = open(1024)?;
        assert_eq!(
            large.charge_caller_inputs(forged(work, 1024), forged(bytes, 1024)),
            Err("caller inputs exceed source work/storage budget")
        );
        assert_eq!(source_contract_costs(&large), header);
    }
    Ok(())
}
