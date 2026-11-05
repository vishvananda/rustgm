//! Poll declarations remain ordinary sources until an actual operation is demanded.
use crate::resolve as source;
use rustgm::pipeline;
type Result = std::result::Result<(), Box<dyn std::error::Error>>;

#[test]
fn ordinary_polling_roles_preserve_unused_shapes_and_all_body_checks() -> Result {
    for (name, valid) in [
        ("actual", true),
        ("alternate", true),
        ("foreign-parent-unused", true),
        ("invalid-unused-body", false),
    ] {
        let text = source::fixture(&format!("pa31/tests/spec/polling-source/{name}.rs"))?;
        let report = source::source_provider(&text, "polling_source")?;
        let result = pipeline::analyze_source(&report);
        if valid {
            result?;
        } else {
            let error = result.err().ok_or("accepted invalid unused body")?;
            assert!(error.message.contains("incompatible types"), "{error}");
        }
    }
    Ok(())
}
