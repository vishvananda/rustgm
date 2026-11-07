//! The demand-only view shares canonical edge positions and pays before growth.
use super::*;
use crate::test_source;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn check(width: usize) -> Result<(usize, usize)> {
    use std::fmt::Write;
    let mut source = String::new();
    for index in 0..width {
        write!(source, "pub fn f{index}(){{#[cold]||{{}};}}")?;
    }
    source.push_str("pub fn tail()->impl Fn(){#[cold]||{}}pub fn bad(){let f=#[cold]||{};f();}");
    let report = test_source::resolve(&source, true)?;
    let unit = &report.package.units[0];
    let (mut work, mut bytes) = (0usize, 0usize);
    let view = AttributePositions::prepare(unit, FileId(0), |steps, storage| {
        work = work.saturating_add(steps);
        bytes = bytes.saturating_add(storage);
        Ok(())
    })?;
    let mut required = 0;
    for owner in &unit.configured.owners {
        if owner.enabled {
            required += usize::from(view.requires_expression_feature(unit, owner.node)?);
        }
    }
    assert_eq!(required, 1);
    assert!(work <= unit.configured.ast.nodes.len() * 12);
    assert!(bytes > unit.configured.ast.nodes.len());
    let foreign = test_source::resolve(&source, true)?;
    assert!(
        view.requires_expression_feature(&foreign.package.units[0], unit.configured.owners[0].node)
            .is_err()
    );
    let mut attempts = 0;
    assert!(
        AttributePositions::prepare(unit, FileId(0), |_, _| {
            attempts += 1;
            Err(Error::unlocated("exhausted actual position payer"))
        })
        .is_err()
    );
    assert_eq!(attempts, 1);
    Ok((work, bytes))
}
#[test]
fn configured_attribute_positions_are_linear_paid_and_unit_bound() -> Result {
    let small = check(32)?;
    let large = check(128)?;
    assert!(large.0 <= small.0 * 4);
    assert!(large.1 <= small.1 * 5);
    assert_eq!(size_of::<AttributePositions<'_>>(), 32);
    Ok(())
}
