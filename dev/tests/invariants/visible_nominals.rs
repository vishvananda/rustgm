//! Original independently observed owners and normalized shared type interfaces.
use super::*;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const ROOT: &str = "#![feature(staged_api)]#![stable(feature=\"root\",since=\"1.0.0\")]";
const STABLE: &str = "#[stable(feature=\"api\",since=\"1.0.0\")]";

fn checked(source: &str) -> Result<(Report, Declarations)> {
    let report = crate::test_source::resolve(source, true)?;
    let facts = crate::sema::declarations::analyze(&report)?;
    Ok((report, facts))
}
#[test]
fn normalized_public_interfaces_preserve_private_marked_and_hidden_owner_rules() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../pa31/tests/spec/library-stability/owners");
    for (name, accepted) in [
        ("signature-exposes-unmarked-type", true),
        ("signature-exposes-marked-type", true),
        ("public-type-through-signature", false),
        ("marked-public-type-through-signature", true),
        ("marked-reexported-public-type", true),
    ] {
        let source = std::fs::read_to_string(root.join(name).join("lib.rs"))?;
        let result = checked(&source);
        if accepted {
            result.map_err(|error| format!("{name}: {error}"))?;
        } else {
            let error = result
                .err()
                .ok_or("unannotated exposed public type was accepted")?;
            assert!(
                error.to_string().contains("missing library stability"),
                "{name}: {error}"
            );
        }
    }
    for (name, declaration, api, accepted) in [
        ("unexposed", "mod hidden{pub struct Hidden;}", "", true),
        (
            "private alias",
            "mod hidden{pub struct Hidden;}type Alias=hidden::Hidden;",
            "pub fn answer()->Alias{hidden::Hidden}",
            false,
        ),
        (
            "marked alias target",
            "mod hidden{#[stable(feature=\"api\",since=\"1.0.0\")]pub struct Hidden;}type Alias=hidden::Hidden;",
            "pub fn answer()->Alias{hidden::Hidden}",
            true,
        ),
        (
            "input reference",
            "mod hidden{pub struct Hidden;}",
            "pub fn answer(_: &hidden::Hidden){}",
            false,
        ),
        (
            "function pointer",
            "mod hidden{pub struct Hidden;}",
            "pub fn answer(_:fn()->hidden::Hidden){}",
            false,
        ),
        (
            "body only",
            "mod hidden{pub struct Hidden;}",
            "pub fn answer(){let _ = hidden::Hidden;}",
            true,
        ),
        (
            "generic bound",
            "mod hidden{pub trait Hidden{}}",
            "pub fn answer<T:hidden::Hidden>(){}",
            false,
        ),
    ] {
        let source = format!(
            "{ROOT}{declaration}{}{api}",
            if api.is_empty() { "" } else { STABLE }
        );
        let result = checked(&source);
        if accepted {
            result.map_err(|error| format!("{name}: {error}"))?;
        } else {
            let error = result
                .err()
                .ok_or_else(|| format!("{name}: exposed public API was accepted"))?;
            assert!(
                error.to_string().contains("missing library stability"),
                "{name}: {error}"
            );
        }
    }
    Ok(())
}
fn shared(width: usize, hidden: usize) -> String {
    use std::fmt::Write;
    let mut source = format!(
        "{ROOT}mod hidden{{{STABLE}pub struct Visible;type Shared=(*const Visible,(*const Visible,*const Visible));{STABLE}pub type Alias=Shared;"
    );
    for index in 0..hidden {
        write!(source, "pub struct Unused{index};").ok();
    }
    source.push('}');
    for index in 0..width {
        write!(source, "{STABLE}pub fn f{index}(_: &hidden::Alias){{}}").ok();
    }
    source
}
#[test]
fn shared_interfaces_keep_bounded_visitation_and_prepaid_failure_paths() -> Result {
    let mut observations = Vec::new();
    for width in [8, 32, 128] {
        let (report, facts) = checked(&shared(width, 1))?;
        let cost = check(&report, &facts)?;
        assert!(cost.steps < 512 * width + 4096, "{width}: {cost:?}");
        assert!(cost.bytes < 4096 * width + 65536, "{width}: {cost:?}");
        observations.push((width, cost));
        let (other_report, other_facts) = checked(&shared(width, 128))?;
        let other = check(&other_report, &other_facts)?;
        assert_eq!(
            other.types, cost.types,
            "hidden unused declarations add no type walks"
        );
        assert_eq!(other.declarations, cost.declarations);
        // The existing inventory is larger, but there is no new all-declaration
        // semantic scan; its storage is counted once without walking those rows.
        assert_eq!(other.steps, cost.steps);
        for (steps, bytes) in [
            (0, usize::MAX),
            (usize::MAX, 0),
            (cost.steps - 1, usize::MAX),
        ] {
            assert!(check_with_limits(&report, &facts, steps, bytes).is_err());
        }
        assert_eq!(
            check(&report, &facts)?.steps,
            cost.steps,
            "failed queries poison no state"
        );
    }
    for pair in observations.windows(2) {
        assert_eq!(
            pair[1].1.types - pair[0].1.types,
            pair[1].0 - pair[0].0,
            "each additional callable adds only its distinct item; its signature graph is shared"
        );
    }
    let (report, facts) = checked("mod hidden{pub struct Unmarked;}pub fn f(){}")?;
    assert_eq!(check_with_limits(&report, &facts, 0, 0)?.steps, 0);
    assert_eq!(check(&report, &facts)?.bytes, 0);
    assert_eq!(size_of::<crate::resolve::package::Package>(), 256);
    assert_eq!(size_of::<crate::resolve::stability::Inventory>(), 88);
    assert_eq!(size_of::<crate::body::ir::Body>(), 448);
    assert_eq!(size_of::<Task>(), 8);
    eprintln!(
        "library interfaces costs={observations:?} Task={} Inventory={} Package={} Body={}",
        size_of::<Task>(),
        size_of::<crate::resolve::stability::Inventory>(),
        size_of::<crate::resolve::package::Package>(),
        size_of::<crate::body::ir::Body>()
    );
    Ok(())
}
