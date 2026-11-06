//! Associated-bound sugar retains real projected subjects and every written clause.
use crate::semantic;
#[test]
fn projected_constraints_preserve_all_real_bound_obligations()
-> Result<(), Box<dyn std::error::Error>> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let cases = [
        ("pa20/tests/spec/assoc-bounds/plain.rs", true),
        ("pa20/tests/spec/assoc-bounds/multi.rs", true),
        ("pa20/tests/spec/assoc-bounds/second.rs", false),
        ("pa20/tests/spec/assoc-bounds/miss.rs", false),
        ("pa20/tests/spec/assoc-bounds/owner.rs", false),
        ("pa20/tests/spec/assoc-bounds/sib.rs", false),
        ("pa20/tests/spec/assoc-bounds/nested.rs", true),
        ("pa20/tests/spec/assoc-bounds/nestno.rs", false),
        ("pa20/tests/spec/assoc-bounds/life.rs", true),
        ("pa20/tests/spec/assoc-bounds/lfmiss.rs", false),
        ("pa20/tests/spec/assoc-bounds/dup.rs", true),
        ("pa20/tests/spec/assoc-bounds/unsize.rs", false),
        ("pa20/tests/spec/assoc-bounds/value.rs", false),
        ("pa20/tests/spec/assoc-bounds/equal.rs", true),
        ("pa21/tests/spec/assoc-bounds/hrtb.rs", true),
        ("pa21/tests/spec/assoc-bounds/hrbad.rs", false),
        ("pa21/tests/spec/assoc-bounds/outer.rs", true),
        ("pa21/tests/spec/assoc-bounds/gat.rs", true),
        ("pa21/tests/spec/assoc-bounds/gwrong.rs", false),
        ("pa21/tests/spec/assoc-bounds/alias.rs", true),
        ("pa20/tests/spec/assoc-bounds/eqdup.rs", true),
        ("pa20/tests/spec/assoc-bounds/dirlf.rs", false),
        ("pa21/tests/spec/assoc-bounds/unused.rs", true),
        ("pa20/tests/spec/assoc-bounds/eqdiff.rs", true),
        ("pa20/tests/spec/assoc-bounds/eqcall.rs", false),
        ("pa20/tests/spec/assoc-bounds/late.rs", true),
        ("pa20/tests/spec/assoc-bounds/ldebt.rs", true),
        ("pa20/tests/spec/assoc-bounds/lnodeb.rs", false),
        ("pa20/tests/spec/assoc-bounds/self.rs", true),
        ("pa20/tests/spec/assoc-bounds/swhere.rs", true),
        ("pa20/tests/spec/assoc-bounds/sproj.rs", true),
        ("pa20/tests/spec/assoc-bounds/sbare.rs", true),
        ("pa20/tests/spec/assoc-bounds/sbad.rs", false),
    ];
    let mut failures = Vec::new();
    for (path, accepted) in cases {
        let source = std::fs::read_to_string(root.join(path))?;
        let result = semantic::analyze(&source).and_then(|(mut report, mut facts)| {
            let entry =
                rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
            rustgm::lower::instances::build(&report, &mut facts, entry)
                .map(|_| ())
                .map_err(|e| e.to_string())
        });
        let unsupported = result.as_ref().err().is_some_and(|e| {
            e.contains("not implemented")
                || e.contains("projected bound obligations")
                || e.contains("budget")
        });
        eprintln!("{path}: {result:?}");
        if result.is_ok() != accepted || unsupported {
            failures.push((path, accepted, result));
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
    Ok(())
}

#[test]
fn wide_and_nested_constraints_keep_finite_completed_work() -> Result<(), Box<dyn std::error::Error>>
{
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    for width in [4, 16, 64] {
        for kind in ["wide", "deep"] {
            let source = std::fs::read_to_string(
                root.join(format!("pa20/tests/spec/assoc-bounds/{kind}{width}.rs")),
            )?;
            let (mut report, mut facts) = semantic::analyze(&source)?;
            let entry = rustgm::sema::entry::select(&mut report, &facts)?;
            rustgm::lower::instances::build(&report, &mut facts, entry)?;
            eprintln!(
                "{kind}{width}: canonical={} inference={} selections={}",
                facts.body_work.canonical_steps,
                facts.body_work.inference_steps,
                facts.body_work.trait_selections
            );
            assert!(facts.body_work.canonical_steps < 1_000_000);
            assert!(facts.body_work.inference_steps < 1_000_000);
        }
    }
    Ok(())
}
