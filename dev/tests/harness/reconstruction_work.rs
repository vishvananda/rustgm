//! Paired ordinary/raw reference workloads enforce bounded work and storage overhead.
#[path = "../helpers/qualify.rs"]
pub(super) mod rustc;
#[path = "../helpers/resolve.rs"]
pub(super) mod source;
pub fn compare(
    suite: &str,
    input: impl Fn(bool, usize) -> String,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut ordinary = std::collections::HashMap::new();
    let mut overhead = None;
    for raw in [false, true] {
        let mut previous = None;
        for count in [8, 32, 128] {
            let text = input(raw, count);
            rustc::qualify(&format!("{suite}-{raw}-{count}"), &text, true)?;
            let mut report = source::resolve(&text, true)?;
            let rustgm::pipeline::Analysis {
                mut facts,
                prepared,
            } = rustgm::pipeline::analyze_source(&report)?;
            let entry = rustgm::sema::entry::select(&mut report, &facts)?;
            rustgm::lower::instances::build_prepared(&report, &mut facts, entry, prepared)?;
            let work = &facts.constant_work;
            println!(
                "raw={raw} n={count} steps={} slots={}",
                work.steps, work.peak_slots
            );
            assert!(work.peak_slots <= count + 96);
            if let Some(old) = previous {
                assert!(work.steps < old * 5);
            }
            previous = Some(work.steps);
            if raw {
                let baseline = ordinary.get(&count).copied().ok_or("ordinary baseline")?;
                let delta = i64::try_from(work.peak_slots)? - baseline;
                if let Some(old) = overhead {
                    assert_eq!(delta, old);
                }
                overhead = Some(delta);
            } else {
                ordinary.insert(count, i64::try_from(work.peak_slots)?);
            }
        }
    }
    Ok(())
}
