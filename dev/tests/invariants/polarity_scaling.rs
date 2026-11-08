//! Imported auto impls cost a source client linear work. A chain of nominals,
//! each holding the previous one and a `Unique` of itself, grows its packet by
//! the same rows per link, and the client's `Send` proof through every
//! `Unique` self-cycle grows its header-budget work and its inference steps
//! by at most a constant per link.
use crate::{lower::instances::SourcePublication, sema::crates::Linkage, support::crates};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const SPEC: &str = "pa29/tests/spec/auto-impls";
const LENGTHS: [usize; 3] = [8, 16, 32];
/// The provider and its chain. Zero-padded names give every row one width.
fn chain(provider: &str, length: usize) -> Result<String> {
    use std::fmt::Write;
    let mut text = format!(
        "{provider}// Generated scaling chain.\npub struct S000 {{ pub next: Unique<S000> }}\n"
    );
    for index in 1..length {
        writeln!(
            text,
            "pub struct S{index:03} {{ pub inner: S{:03}, pub next: Unique<S{index:03}> }}",
            index - 1
        )?;
    }
    Ok(text)
}
/// The packet's bytes, then the client's header-budget work and inference
/// steps when it requires the chain's last link to be `Send`. The body's
/// solved trait goals report their search steps as inference steps.
fn measured(provider: &str, length: usize) -> Result<[usize; 3]> {
    let interface = crate::module_packets::exported_profile(
        &chain(provider, length)?,
        SourcePublication::ModuleBodies,
        "unique_chain",
    )?;
    let bytes = crates::encode(&interface)?;
    let client = format!(
        "#![feature(no_core)]\n#![no_core]\nextern crate base;\nfn need<T: base::Send>() {{}}\n#[unsafe(no_mangle)]\npub extern \"C\" fn entry() -> i32 {{ need::<base::S{:03}>(); 42 }}\n",
        length - 1
    );
    let (_, analysis, _) =
        crate::source_packets::client_analysis(crates::decode(&bytes)?, &client)?;
    let (work, _, _, _) = analysis
        .facts
        .crates
        .as_deref()
        .and_then(Linkage::original_work)
        .ok_or("installed header budget")?;
    Ok([bytes.len(), work, analysis.facts.body_work.inference_steps])
}
#[test]
fn imported_auto_impl_chains_cost_linear_packets_and_client_work() -> Result {
    let provider = crate::test_source::fixture(&format!("{SPEC}/provider.rs"))?;
    let mut observed = Vec::with_capacity(LENGTHS.len());
    for length in LENGTHS {
        observed.push(measured(&provider, length)?);
    }
    let [small, middle, large] = <[[usize; 3]; 3]>::try_from(observed).map_err(|_| "lengths")?;
    let increments = |metric: usize| {
        (
            middle[metric] - small[metric],
            large[metric] - middle[metric],
        )
    };
    let (first, second) = increments(0);
    assert_eq!(
        second,
        2 * first,
        "packet bytes {small:?} {middle:?} {large:?}"
    );
    for (metric, label) in [(1, "header-budget work"), (2, "inference steps")] {
        let (first, second) = increments(metric);
        assert!(first > 0, "{label} did not grow: {small:?} {middle:?}");
        assert!(
            second <= 2 * first + 64,
            "{label}: {small:?} {middle:?} {large:?}"
        );
    }
    Ok(())
}
