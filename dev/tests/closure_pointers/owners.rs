//! Initializer declarations and root closures retain independent checked contracts.
use super::{native, semantic};
#[path = "../helpers/objects.rs"]
mod object;
const CASES: &[(&str, bool, bool)] = &[
    ("ordinary", true, false),
    ("root", true, true),
    ("grouped", true, true),
    ("attributed", true, true),
    ("block", true, false),
    ("inline", true, true),
    ("static", true, true),
    ("dangling", false, true),
    ("escaping-input", false, true),
];
fn fixture(name: &str) -> Result<String, std::io::Error> {
    std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../pa24/tests/spec/initializer-owners/{name}.rs")),
    )
}
#[test]
fn independent_initializer_owners_preserve_source_outcomes()
-> Result<(), Box<dyn std::error::Error>> {
    for &(name, accepted, _) in CASES {
        let text = fixture(name)?;
        object::qualify(
            &format!("parallel/borrowed-result/qualification/{name}"),
            &text,
            accepted,
            "bin",
        )?;
        let mut report = semantic::resolve(&text)?;
        let result = rustgm::pipeline::analyze_source(&report).and_then(|analysis| {
            let mut facts = analysis.facts;
            let entry = rustgm::sema::entry::select(&mut report, &facts)?;
            rustgm::lower::instances::build_prepared(&report, &mut facts, entry, analysis.prepared)
        });
        assert_eq!(result.is_ok(), accepted, "{name}: {:?}", result.err());
    }
    Ok(())
}
#[test]
fn semantic_owner_and_executable_root_remain_distinct() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, accepted, separate) in CASES {
        if !accepted {
            continue;
        }
        let (report, facts) = semantic::analyze(&fixture(name)?)?;
        let initializer = facts.initializers.first().ok_or("missing initializer")?;
        assert_eq!(
            initializer.semantic_source() != initializer.value,
            separate,
            "{name}"
        );
        let contracts = rustgm::body::regions::collect(&facts)?;
        let owner = contracts
            .bodies
            .iter()
            .find(|body| body.key == rustgm::body::ir::BodyKey::Declaration(initializer.definition))
            .ok_or("missing initializer contract")?;
        assert_eq!(owner.source, initializer.semantic_source().into());
        for closure in facts.closures.all() {
            assert_ne!(owner.source, closure.semantic_source().into(), "{name}");
            assert!(contracts.bodies.iter().any(|body| body.key
                == rustgm::body::ir::BodyKey::Closure(closure.identity)
                && body.source == closure.semantic_source().into()));
        }
        let program = rustgm::body::build(&report, &facts)?;
        rustgm::borrow::regions::check_bodies(
            &facts.types,
            &facts.lifetimes.variances,
            &contracts,
            &program,
        )?;
    }
    Ok(())
}
#[test]
#[ignore = "requires the qualified CPPGM native backend"]
fn initializer_closure_values_execute() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, accepted, _) in CASES {
        if accepted {
            let text = fixture(name)?;
            let body = text
                .strip_prefix(native::CHECK)
                .ok_or("fixture lost its exact check prefix")?;
            native::check_cases("parallel/borrowed-result/native", &[(name, body, false)])?;
        }
    }
    Ok(())
}
