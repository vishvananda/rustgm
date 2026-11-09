//! Version thirteen's array type rows re-validate from the provider's own
//! exported rows: each element is proved sized from the rows themselves, a
//! value length is canonical on the usize row and a slot length names a
//! usize const parameter. Every forged row is rejected by name.
use super::{Mutation, validate};
use crate::{
    lower::instances::SourcePublication,
    module_packets::{Result, exported_profile, raw},
    support::crates::generics::{self as g, source as s},
};

const PROVIDER: &str = "pa29/tests/spec/array-types/provider.rs";

/// The first array row over a type parameter whose length is a slot, as
/// `(length ref, the element's type slot)`.
fn generic(rows: &s::SourceRows) -> Option<(usize, s::SlotRef)> {
    rows.types.iter().find_map(|row| {
        let s::Type::Array { element, length } = *row else {
            return None;
        };
        let s::Type::Parameter(slot) = *rows.types.get(element.0 as usize)? else {
            return None;
        };
        matches!(
            rows.array_lengths().get(length.0 as usize)?,
            s::ConstTerm::Parameter(_)
        )
        .then_some((length.0 as usize, slot))
    })
}
/// The first array row whose length is the value `bits`.
fn valued(rows: &s::SourceRows, bits: u128) -> Option<usize> {
    rows.types.iter().position(|row| {
        let s::Type::Array { length, .. } = *row else {
            return false;
        };
        matches!(
            rows.array_lengths().get(length.0 as usize),
            Some(s::ConstTerm::Value(value)) if value.bits == bits
        )
    })
}

/// Each forged row and the name validation rejects it by.
const MUTATIONS: [(&str, Mutation, &str); 2] = [
    (
        "slot length names a type slot",
        |rows| {
            let (length, slot) = generic(rows)?;
            rows.cold_mut().array_lengths[length] = s::ConstTerm::Parameter(slot);
            Some(())
        },
        "source array length slot is not a usize const parameter",
    ),
    (
        "element is the str row",
        |rows| {
            let row = valued(rows, 4)?;
            let text = rows
                .types
                .iter()
                .position(|row| matches!(row, s::Type::Str))?;
            let s::Type::Array { length, .. } = rows.types[row] else {
                return None;
            };
            rows.types[row] = s::Type::Array {
                element: g::TypeRef(u32::try_from(text).ok()?),
                length,
            };
            Some(())
        },
        "source array element requires a sized type",
    ),
];

#[test]
fn array_rows_keep_dense_lengths_and_reject_forged_elements_and_slots() -> Result {
    let source = crate::test_source::fixture(PROVIDER)?;
    let interface = exported_profile(&source, SourcePublication::ModuleBodies, "array_rows")?;
    let (rows, _, _) = raw(&interface)?;
    // One length term per array row, dense in type-row order.
    let lengths: Vec<u32> = rows
        .types
        .iter()
        .filter_map(|row| match row {
            s::Type::Array { length, .. } => Some(length.0),
            _ => None,
        })
        .collect();
    let dense: Vec<u32> = (0..u32::try_from(rows.array_lengths().len())?).collect();
    assert_eq!(lengths, dense);
    assert!(valued(&rows, 0).is_some() && valued(&rows, 4).is_some());
    validate(&interface, rows.clone())?;
    for (name, mutation, message) in MUTATIONS {
        let mut forged = rows.clone();
        assert_eq!(mutation(&mut forged), Some(()), "{name}");
        assert_eq!(validate(&interface, forged), Err(message), "{name}");
    }
    Ok(())
}
