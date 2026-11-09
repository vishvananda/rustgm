//! Re-proving an imported packet's same-head pairs costs a consumer linear
//! work. A packet of N traits, each with core's same-head pair
//! `impl<T: E + ?Sized> E for &T` and `impl !E for &str`, is consumed by an
//! empty client. Every pair is proved disjoint from the packet's perspective,
//! and the client's header-budget work and the pairs' coherence steps grow by
//! at most a constant per pair.
use crate::{
    lower::instances::SourcePublication,
    sema::{crates::Linkage, model::Declarations},
    support::{crates, hash::HashMap},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const SPEC: &str = "pa29/tests/spec/knowable-absence";
const COUNTS: [usize; 3] = [8, 16, 32];
const CLIENT: &str = "#![feature(no_core)]\n#![no_core]\nextern crate base;\n#[unsafe(no_mangle)]\npub extern \"C\" fn entry() -> i32 { 42 }\n";
/// The provider and its generated pairs. Zero-padded names give every row one width.
fn pairs(provider: &str, count: usize) -> Result<String> {
    use std::fmt::Write;
    let mut text = format!("{provider}// Generated scaling pairs.\n");
    for index in 0..count {
        writeln!(
            text,
            "pub trait E{index:03} {{}}\nimpl<T: E{index:03} + ?Sized> E{index:03} for &T {{}}\nimpl !E{index:03} for &str {{}}"
        )?;
    }
    Ok(text)
}
/// Every pair of the consumer's impls of one trait, re-proved as the
/// consumer proves it: each must be disjoint, and their steps are summed.
fn coherence_steps(facts: &mut Declarations) -> Result<usize> {
    let mut traits: HashMap<u32, Vec<_>> = HashMap::default();
    for header in facts
        .generics
        .implementations
        .as_ref()
        .ok_or("registry")?
        .headers()
    {
        let Some((application, _)) = header.predicate.bound.trait_parts() else {
            continue;
        };
        let item = facts
            .types
            .trait_application(application)
            .ok_or("trait application")?
            .item;
        traits
            .entry(item.0)
            .or_default()
            .push((header.predicate, header.owner));
    }
    let mut steps = 0;
    for headers in traits.into_values() {
        for (index, &left) in headers.iter().enumerate() {
            for &right in &headers[index + 1..] {
                let (possible, work) = super::super::possible(facts, left, right)?;
                assert!(!possible, "an imported pair stays disjoint");
                steps += work;
            }
        }
    }
    Ok(steps)
}
/// The packet's bytes, then the client's header-budget work and the pairs'
/// coherence steps.
fn measured(provider: &str, count: usize) -> Result<[usize; 3]> {
    let interface = crate::module_packets::exported_profile(
        &pairs(provider, count)?,
        SourcePublication::ModuleBodies,
        "absence_pairs",
    )?;
    let bytes = crates::encode(&interface)?;
    let (_, mut analysis, _) =
        crate::source_packets::client_analysis(crates::decode(&bytes)?, CLIENT)?;
    let (work, _, _, _) = analysis
        .facts
        .crates
        .as_deref()
        .and_then(Linkage::original_work)
        .ok_or("installed header budget")?;
    let steps = coherence_steps(&mut analysis.facts)?;
    Ok([bytes.len(), work, steps])
}
#[test]
fn imported_same_head_pairs_cost_linear_client_work() -> Result {
    let provider = crate::test_source::fixture(&format!("{SPEC}/provider.rs"))?;
    let mut observed = Vec::with_capacity(COUNTS.len());
    for count in COUNTS {
        observed.push(measured(&provider, count)?);
    }
    let [small, middle, large] = <[[usize; 3]; 3]>::try_from(observed).map_err(|_| "counts")?;
    for (metric, label) in [
        (0, "packet bytes"),
        (1, "header-budget work"),
        (2, "coherence steps"),
    ] {
        let (first, second) = (
            middle[metric] - small[metric],
            large[metric] - middle[metric],
        );
        assert!(first > 0, "{label} did not grow: {small:?} {middle:?}");
        assert!(
            second <= 2 * first + 64,
            "{label}: {small:?} {middle:?} {large:?}"
        );
    }
    Ok(())
}
