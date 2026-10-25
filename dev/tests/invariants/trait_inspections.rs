//! Actual non-role numbered rows retain complete source contracts without import permission.
use super::*;
use crate::unpaid_cases::{Result, trait_extended};
const HOLDER: &str = "\npub trait Holder<V>{fn carry<U>(value:U)->U{value}fn borrow(value:&V)->&V{value}}\nimpl Holder<u8> for u8{}";

#[test]
fn numbered_trait_inputs_retain_full_family_and_actual_contract_capacity() -> Result {
    let actual = trait_extended(HOLDER)?;
    let rows = actual.inventory.artifacts()[0]
        .source_descriptor()
        .ok_or("actual Provided data")?
        .rows();
    assert!(
        rows.header_attributes().is_none(),
        "a raw sidecar is not an imported known contract"
    );
    let holder = rows
        .traits
        .iter()
        .find(|header| {
            rows.declarations[header.declaration.0 as usize]
                .name
                .as_deref()
                == Some("Holder")
        })
        .ok_or("actual nonrole trait")?;
    assert!(
        !rows
            .roles
            .iter()
            .any(|role| role.declaration.declaration == holder.declaration)
    );
    assert_eq!(rows.roles.len(), 9);
    assert_eq!(rows.body_units.len(), rows.originals.len());
    let contracts = &actual.contracts;
    let bytes = actual.contract_bytes;
    let reasons: usize = contracts
        .functions
        .iter()
        .chain(&contracts.traits)
        .filter_map(|row| {
            row.must_use
                .as_ref()
                .and_then(|contract| contract.reason.as_ref())
        })
        .map(String::capacity)
        .sum();
    assert_eq!(
        bytes,
        size_of::<s::HeaderAttributeColumns>()
            + retained::vector(&contracts.functions)
            + retained::vector(&contracts.traits)
            + reasons
    );
    assert_eq!(contracts.functions.len(), rows.functions.len());
    assert_eq!(contracts.traits.len(), rows.traits.len());
    assert!(actual.work.reserved_bytes >= bytes);
    check_family(rows, contracts, holder.declaration)?;
    eprintln!(
        "trait inspection enclosing={} source={} columns={} functions={}/{} traits={}/{} reason_capacity={} retained_columns={} source_reserved={} source_scratch={} steps={}",
        size_of::<TraitInspection>(),
        size_of::<SourceInspection>(),
        size_of::<s::HeaderAttributeColumns>(),
        contracts.functions.len(),
        contracts.functions.capacity(),
        contracts.traits.len(),
        contracts.traits.capacity(),
        reasons,
        bytes,
        actual.work.reserved_bytes,
        actual.work.scratch_bytes,
        actual.work.steps
    );
    Ok(())
}
fn check_family(
    rows: s::ValidatedView<'_>,
    contracts: &s::HeaderAttributeColumns,
    holder: crate::support::crates::owner_catalog::DeclarationRef,
) -> Result {
    for name in ["clone", "clone_from", "identity", "carry", "borrow"] {
        let (index, header) = rows
            .functions
            .iter()
            .enumerate()
            .find(|(_, header)| {
                rows.declarations[header.declaration.0 as usize]
                    .name
                    .as_deref()
                    == Some(name)
            })
            .ok_or("actual complete named family")?;
        assert_eq!(contracts.functions[index].declaration, header.declaration);
        match name {
            "clone" => assert!(
                contracts.functions[index]
                    .must_use
                    .as_ref()
                    .and_then(|contract| contract.reason.as_deref())
                    .is_some_and(|reason| reason.contains("cloning"))
            ),
            "clone_from" | "identity" => assert!(contracts.functions[index].inline.is_some()),
            "carry" => {
                let scope = rows
                    .scopes
                    .iter()
                    .find(|scope| scope.owner == header.scope)
                    .ok_or("actual member scope")?;
                assert_eq!(scope.parent, Some(holder));
                assert_eq!(header.regions.total, 0);
            }
            "borrow" => {
                assert_eq!(header.regions.total, 1);
                let s::FunctionBody::Original(body) = header.body else {
                    return Err("actual nonzero body".into());
                };
                assert_eq!(rows.body_units[body.0 as usize].universals, 1);
            }
            _ => {}
        }
    }
    Ok(())
}
fn inspect(suffix: &str) -> Result<(SourceInspection, s::HeaderAttributeColumns, usize)> {
    inspect_prefixed("", suffix)
}
fn inspect_prefixed(
    prefix: &str,
    suffix: &str,
) -> Result<(SourceInspection, s::HeaderAttributeColumns, usize)> {
    let mut source = String::from(prefix);
    source.push_str(&crate::test_source::fixture(
        "pa31/tests/spec/source-header-transport/semantic-provider-qualified.rs",
    )?);
    source.push_str(suffix);
    let report = crate::test_source::source_provider(&source, "trait_inspection")?;
    let identity = report
        .package
        .source_core()
        .ok_or("actual source policy")?
        .provider
        .clone();
    let mut analysis = crate::pipeline::analyze_source(&report)?;
    Ok(
        crate::lower::instances::inspect_source_trait_inputs_with_numbering(
            &report,
            &mut analysis.facts,
            &identity,
            analysis.prepared,
        )?
        .into_parts(),
    )
}
#[test]
fn numbered_trait_inputs_preserve_defaultness_fences_and_failed_work() -> Result {
    for (declaration, expected) in [
        (
            "trait Local{fn item(){}} impl Local for u8{default fn item(){}}",
            "source defaultness contract transport is not implemented",
        ),
        (
            "trait Local{} default impl Local for u8{}",
            "default impl specialization graph semantics are not implemented",
        ),
        (
            "trait Local{const VALUE:u8;} impl Local for u8{default const VALUE:u8=7;}",
            "default associated value specialization semantics are not implemented",
        ),
        (
            "trait Local{type Value;} impl Local for u8{default type Value=u8;}",
            "default associated value specialization semantics are not implemented",
        ),
    ] {
        let error = inspect_prefixed("#![feature(specialization)]\n", declaration)
            .err()
            .ok_or("unsupported default contract escaped raw completion")?;
        assert!(error.to_string().contains(expected), "{error}");
    }
    inspect(&format!(
        "{HOLDER}\n#[cfg(any())] impl Missing for u8{{default fn item(){{}}}}"
    ))?;
    let (source, _, bytes) = inspect(HOLDER)?;
    let mut storage = source.work;
    let previous = storage.steps;
    assert!(
        validate(
            &source.rows,
            &mut storage,
            crate::support::crates::MAX_BYTES
        )
        .is_err()
    );
    assert!(storage.steps > previous);
    let mut work = SourceWork {
        steps: crate::support::crates::MAX_COMPONENTS,
        ..source.work
    };
    assert!(validate(&source.rows, &mut work, bytes).is_err());
    assert!(work.steps > crate::support::crates::MAX_COMPONENTS);
    Ok(())
}
