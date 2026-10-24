//! Physical type visibility is separate from definition-site name selection.
use super::{fixture, retained_items::lower};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const POSITIVE: &[&str] = &[
    "public-function-pointer",
    "public-trait-method",
    "public-opaque-bound",
    "public-nested-adt",
];
const NEGATIVE: &[&str] = &[
    "private-function-value",
    "private-return-value",
    "private-nested-signature",
    "private-inherent-method",
    "private-associated-constant",
    "private-opaque-bound",
];

#[test]
fn scoped_names_preserve_physical_type_and_associated_item_visibility() -> TestResult {
    let mut failures = Vec::new();
    for &name in POSITIVE {
        if let Err(error) = lower(&fixture(&format!("semantic-privacy/{name}"))?) {
            failures.push(format!("{name}: {error}"));
        }
    }
    for &name in NEGATIVE {
        match lower(&fixture(&format!("semantic-privacy/{name}"))?) {
            Ok(_) => failures.push(format!("{name}: private semantic use accepted")),
            Err(error) if error.to_string().contains("private") => {}
            Err(error) => failures.push(format!("{name}: wrong rejection: {error}")),
        }
    }
    if !failures.is_empty() {
        return Err(failures.join("\n").into());
    }
    Ok(())
}

#[test]
#[ignore = "requires the pinned CPPGM backend and host C linker"]
fn public_interfaces_preserve_their_private_implementation_native_answers() -> TestResult {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root
        .join("obj/parallel/pa31/type-privacy-native")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    let consumer =
        std::fs::read_to_string(root.join("pa31/tests/spec/source-panic-macros/consumer.c"))?;
    for &name in POSITIVE {
        let program = lower(&fixture(&format!("semantic-privacy/{name}"))?)
            .map_err(|error| format!("{name}: {error}"))?;
        crate::ffi_program::check(root, &folder, name, &program, &consumer, "7\n")?;
    }
    Ok(())
}
