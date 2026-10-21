//! Canonical imported traits use the ordinary dot receiver selection pipeline.
use super::{checked, native, rustc};
const CASES: &[(&str, bool)] = &[
    ("two-phase", true),
    ("shadow-block", true),
    ("block-scope", true),
    ("nominal", true),
    ("primitive", true),
    ("forwarding", true),
    ("generic", true),
    ("alias", true),
    ("anonymous", true),
    ("glob", true),
    ("nonimported", false),
    ("outer-scope", false),
    ("shadow", false),
    ("user-trait", true),
    ("ambiguous", false),
    ("inherent", true),
    ("index", true),
    ("index-mut", true),
    ("index-loan", false),
    ("borrowed-immutable", false),
    ("owned-move", false),
    ("owned-add", true),
    ("closure", true),
    ("order", true),
    ("unsupported-pair", false),
    ("unrelated-receiver", false),
];
fn fixture(name: &str) -> Result<String, std::io::Error> {
    std::fs::read_to_string(format!(
        "{}/../pa26/tests/spec/dot-methods/{name}.rs",
        env!("CARGO_MANIFEST_DIR")
    ))
}
#[test]
fn independent_dot_methods() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, accepted) in CASES {
        rustc::qualify(
            &format!("parallel/pa26/dot-methods/qualification/{name}"),
            &fixture(name)?,
            accepted,
        )?;
    }
    Ok(())
}
#[test]
fn selected_dot_methods() -> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for &(name, accepted) in CASES {
        let result = checked(&fixture(name)?);
        if result.is_ok() != accepted {
            failures.push(format!("{name}: {result:?}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn native_dot_methods() -> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for &(name, accepted) in CASES {
        if accepted
            && let Err(error) = native::check_cases(
                "parallel/pa26/dot-methods/native",
                &[(name, &fixture(name)?, false)],
            )
        {
            failures.push(format!("{name}: {error}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
#[test]
fn repeated_methods_share_visibility_index() -> Result<(), Box<dyn std::error::Error>> {
    use std::fmt::Write;
    let mut observations = Vec::new();
    for width in [8, 32, 128] {
        let mut source = "use core::ops::AddAssign;struct W(u8);impl AddAssign<u8> for W{fn add_assign(&mut self,rhs:u8){self.0+=rhs;}}".to_owned();
        for index in 0..width {
            write!(source, "trait Unused{index}{{fn unrelated{index}(&self);}}")?;
        }
        source.push_str("fn main(){let mut w=W(0);");
        for _ in 0..width {
            source.push_str("w.add_assign(1u8);");
        }
        source.push('}');
        rustc::qualify(
            &format!("parallel/pa26/dot-methods/scaling/{width}"),
            &source,
            true,
        )?;
        let (_, facts) = super::semantic::analyze(&source)?;
        let methods = &facts.methods;
        assert_eq!(methods.calls.len(), width);
        assert_eq!(methods.selection_probes, width);
        assert!(methods.selection_steps < width * 100);
        observations.push((width, methods.trait_visibility_steps));
    }
    let [
        (first_width, first_work),
        (middle_width, middle_work),
        (last_width, last_work),
    ] = observations.as_slice()
    else {
        return Err("missing scaling observations".into());
    };
    assert_eq!(
        (middle_work - first_work) * (last_width - middle_width),
        (last_work - middle_work) * (middle_width - first_width)
    );
    eprintln!("visibility work: {observations:?}");
    Ok(())
}
