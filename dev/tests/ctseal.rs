//! Actual Destruct effects stay distinct after an ordinary nominal proof.
use crate::resolve as source;
use rustgm::pipeline;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn analyze(name: &str) -> Result<pipeline::Analysis> {
    let text = source::fixture(&format!("pa31/tests/spec/source-const-cleanup/{name}.rs"))?;
    let report = source::source_provider(&text, "const_destruct_effects")?;
    Ok(pipeline::analyze_source(&report)?)
}
#[test]
fn ordinary_and_runtime_conditional_destruct_accept_nonconst_drop() -> Result {
    for name in ["seord", "serun"] {
        analyze(name).map_err(|e| format!("{name}: {e}"))?;
    }
    Ok(())
}
#[test]
fn actual_const_drop_and_dropless_fields_pay_the_reached_const_bound() -> Result {
    for name in ["secnst", "senod"] {
        analyze(name).map_err(|e| format!("{name}: {e}"))?;
    }
    Ok(())
}
#[test]
fn ordinary_drop_never_pays_actual_stronger_destruct_effects() -> Result {
    for name in ["senom", "sestrg"] {
        let error = analyze(name)
            .err()
            .ok_or_else(|| format!("{name}: ordinary evidence paid const Destruct"))?;
        assert!(
            error.to_string().contains("can't drop")
                || error.to_string().contains("const Destruct"),
            "{name}: {error}"
        );
    }
    Ok(())
}
