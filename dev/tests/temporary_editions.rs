//! Actual destruction and borrow endpoints follow the selected source edition.
use rustgm::support::edition::Edition;
use std::path::Path;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const NATIVE: &[&str] = &[
    "if-failure",
    "if-success",
    "function-tail",
    "nested-tail",
    "extended-borrow",
    "boolean-condition",
    "match-scrutinee",
    "macro-tail",
];
const EDITIONS: &[(Edition, &str)] = &[(Edition::Rust2021, "2021"), (Edition::Rust2024, "2024")];

fn source(name: &str) -> Result<String> {
    Ok(std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pa34/tests/spec/temporary-editions")
            .join(format!("{name}.rs")),
    )?)
}

#[test]
fn destruction_sources_and_opposing_borrows_reach_complete_lowering() -> Result {
    for &(edition, suffix) in EDITIONS {
        for name in NATIVE {
            let name = format!("{name}-{suffix}");
            assert!(!crate::edition_packages::lowir(&source(&name)?, edition)?.is_empty());
        }
        for (name, accepted) in [
            ("tail-borrow", edition == Edition::Rust2024),
            ("argument-block", edition == Edition::Rust2021),
        ] {
            match crate::edition_packages::lowir(&source(name)?, edition) {
                Ok(bytes) => assert!(accepted && !bytes.is_empty(), "{name}/{suffix}"),
                Err(error) => {
                    assert!(!accepted, "{name}/{suffix}: {error}");
                    let message = error.to_string();
                    assert!(
                        ["borrow", "storage", "region", "lifetime", "loan"]
                            .iter()
                            .any(|word| message.contains(word)),
                        "{name}/{suffix}: {error}"
                    );
                }
            }
        }
    }
    Ok(())
}

#[test]
#[ignore = "requires pinned rustc and CPPGM native backend"]
fn destruction_order_and_accepted_borrows_execute_in_both_editions() -> Result {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("missing root")?;
    let folder = root
        .join("obj/parallel/pa34-temporary-native")
        .join(format!(
            "{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_nanos()
        ));
    std::fs::create_dir_all(&folder)?;
    for &(edition, suffix) in EDITIONS {
        for name in NATIVE {
            let name = format!("{name}-{suffix}");
            crate::edition_packages::native(&folder, &name, &source(&name)?, edition, root)?;
        }
        let name = if edition == Edition::Rust2021 {
            "argument-block"
        } else {
            "tail-borrow"
        };
        crate::edition_packages::native(&folder, name, &source(name)?, edition, root)?;
    }
    Ok(())
}
