//! `core` may name a direct dependency, as rustc's `--extern core=` does;
//! `std` and the path keywords stay reserved.
use super::Dependency;
use crate::support::crates::CrateIdentity;

#[test]
fn core_names_a_direct_dependency_while_std_and_path_keywords_stay_reserved()
-> Result<(), &'static str> {
    let identity = CrateIdentity::new("core", "extern_core")?;
    assert_eq!(
        Dependency::new("core".into(), identity.clone())?.alias(),
        "core"
    );
    for reserved in ["std", "self", "Self", "super", "crate"] {
        assert_eq!(
            Dependency::new(reserved.into(), identity.clone()).err(),
            Some("unsupported external crate alias")
        );
    }
    Ok(())
}
