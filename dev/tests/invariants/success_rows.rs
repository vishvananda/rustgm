//! Storage completions that each publish a newer type grow the dense success
//! rows geometrically. Ascending completions copy the rows only when their
//! capacity doubles, each copy charged once, rather than once per completion.
use crate::types::{
    GenericArgs, ItemId, Kind, Primitive, Store, TypeId,
    aggregate::{Definitions, Kind as Aggregate, Repr, Variant},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Field instantiation steps of `count` storage completions of distinct
/// `S{index} { v: u8 }`, in identity order or in reverse, and the store's
/// type count, which bounds the success rows.
fn steps(count: u32, ascending: bool) -> Result<(usize, usize)> {
    let mut types = Store::default();
    let mut definitions = Definitions::default();
    let fields = types.intern_list(&[TypeId::primitive(Primitive::U8)])?;
    let mut roots = Vec::new();
    for index in 0..count {
        let item = ItemId(index);
        definitions.insert(
            item,
            Aggregate::Struct,
            Repr::default(),
            &[Variant {
                fields,
                discriminant: 0,
            }],
        )?;
        roots.push(types.intern(Kind::Adt(item, GenericArgs::EMPTY))?);
    }
    if !ascending {
        roots.reverse();
    }
    for root in roots {
        definitions.prepare_storage_fields(&mut types, root)?;
    }
    Ok((definitions.instantiation_steps(), types.len()))
}

#[test]
fn ascending_storage_completions_copy_success_rows_geometrically() -> Result {
    let mut observations = Vec::new();
    for count in [64, 256, 512] {
        // The reverse order allocates every row at its first completion.
        let (descending, rows) = steps(count, false)?;
        let (ascending, _) = steps(count, true)?;
        eprintln!(
            "success rows count={count} ascending={ascending} descending={descending} rows={rows}"
        );
        observations.push((ascending, descending, rows));
    }
    for &(ascending, descending, rows) in &observations {
        // Doubling copies at most twice the final rows in all.
        assert!(
            ascending <= descending + 2 * rows,
            "ascending completions {ascending} against {descending} with {rows} rows"
        );
    }
    let [(_, small, _), (_, middle, _), (_, large, _)] = observations[..] else {
        return Err("three completion counts".into());
    };
    // Equal increments per completion: 192 from 64 to 256, 256 from 256 to 512.
    assert_eq!(
        (middle - small) * 256,
        (large - middle) * 192,
        "single-allocation completion steps {small} {middle} {large} are not linear"
    );
    Ok(())
}
