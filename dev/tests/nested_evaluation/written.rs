//! Written inline lifetimes belong to the exact anonymous owner, with ordinary escape proof.
use super::{analyze, inspect, native, rustc};
const CASES: &[(&str, bool)] = &[
    ("original", true),
    ("nested", true),
    ("unsafe", true),
    ("mutable", true),
    ("reborrow", true),
    ("closure", true),
    ("method", true),
    ("promotion", true),
    ("static", true),
    ("dangling", false),
    ("static-local", false),
    ("outer", false),
    ("non-const", false),
];
fn fixture(name: &str) -> Result<String, std::io::Error> {
    let path = if name == "original" {
        "inline/inline-local-annotation.rs".to_owned()
    } else {
        format!("inline-written/{name}.rs")
    };
    std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../pa25/tests/spec/{path}")),
    )
}
#[test]
fn independent_written_lifetime_outcomes() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, accepted) in CASES {
        rustc::qualify(
            &format!("parallel/pa25-inline-written/qualification/{name}"),
            &fixture(name)?,
            accepted,
        )?;
    }
    Ok(())
}
#[test]
fn written_local_regions_keep_exact_inline_ownership() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, accepted) in CASES {
        let source = fixture(name)?;
        assert_eq!(inspect(&source).is_ok(), accepted, "{name}");
        if !accepted {
            continue;
        }
        let (_, analysis) = analyze(&source)?;
        let registry = analysis
            .facts
            .inline_constants
            .as_deref()
            .ok_or("missing inline registry")?;
        let regions: Vec<_> = analysis
            .facts
            .body_regions
            .iter()
            .filter(|region| {
                matches!(
                    region.origin,
                    rustgm::sema::model::BodyRegionOrigin::Annotation
                )
            })
            .collect();
        if name == "original" {
            assert!(!regions.is_empty(), "missing written annotation region");
        }
        for region in regions {
            assert!(registry.get(region.body).is_some(), "{name}");
        }
    }
    Ok(())
}
#[test]
#[ignore = "requires the qualified CPPGM native backend"]
fn written_inline_values_execute() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, accepted) in CASES {
        if !accepted {
            continue;
        }
        let text = format!("{}fn main(){{check(run()==7);}}", fixture(name)?);
        native::check_cases(
            "parallel/pa25-inline-written/native",
            &[(name, &text, false)],
        )?;
    }
    Ok(())
}

#[test]
fn inline_local_context_preserves_enclosing_named_binders() -> Result<(), Box<dyn std::error::Error>>
{
    use rustgm::{sema, types::RegionId};
    let source = "pub fn run<'a>(_: &'a u8)->u8{const{let r:&'a u8=&7;*r}}";
    rustc::qualify(
        "parallel/pa25-inline-written/qualification/generic-binder",
        source,
        true,
    )?;
    let report = super::source::resolve(source, true)?;
    let facts = sema::declarations::analyze(&report)?;
    assert!(
        facts
            .body_regions
            .iter()
            .all(|region| !matches!(region.origin, sema::model::BodyRegionOrigin::Annotation))
    );
    let expected = RegionId::bound(0, 0)?;
    assert!(
        facts
            .lifetimes
            .named
            .values()
            .any(|&region| region == expected)
    );
    Ok(())
}
