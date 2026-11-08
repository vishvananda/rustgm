//! A declaration-time body's storage walk that waits for field templates its
//! declarations publish one at a time resumes at the waited nominal. Every
//! visited type is charged once, so the walk stays linear in its chain.
use super::PendingFields;
use crate::types::{
    GenericArgs, ItemId, Kind, Primitive, Store, TypeId,
    aggregate::{Definitions, Kind as Aggregate, Repr, Variant},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Field instantiation steps of a walk from `S0` through `S0 { v: S1 }` to
/// `S{count - 1} { v: u8 }`, where each template is published only after the
/// walk waits for it. A `restart` walk forgets its progress at every wait.
fn steps(count: u32, restart: bool) -> Result<usize> {
    let mut types = Store::default();
    let mut definitions = Definitions::default();
    let chain = (0..count)
        .map(|index| types.intern(Kind::Adt(ItemId(index), GenericArgs::EMPTY)))
        .collect::<std::result::Result<Vec<TypeId>, _>>()?;
    let mut pending = PendingFields::default();
    let mut waits = 0;
    while let Some(item) = definitions.resume_storage_fields(&mut types, chain[0], &mut pending)? {
        waits += 1;
        let next = item.0 as usize + 1;
        let field = chain
            .get(next)
            .copied()
            .unwrap_or(TypeId::primitive(Primitive::U8));
        let fields = types.intern_list(&[field])?;
        definitions.insert(
            item,
            Aggregate::Struct,
            Repr::default(),
            &[Variant {
                fields,
                discriminant: 0,
            }],
        )?;
        if restart {
            pending = PendingFields::default();
        }
    }
    assert_eq!(waits, count, "each nominal is waited for once");
    let steps = definitions.instantiation_steps();
    let mut warm = PendingFields::default();
    assert_eq!(
        definitions.resume_storage_fields(&mut types, chain[0], &mut warm)?,
        None
    );
    assert_eq!(
        definitions.instantiation_steps(),
        steps,
        "a prepared root is free"
    );
    Ok(steps)
}

#[test]
fn waited_field_templates_resume_their_storage_walk_linearly() -> Result {
    let [small, middle, large] = [64, 256, 512].map(|count| steps(count, false));
    let (small, middle, large) = (small?, middle?, large?);
    // Equal increments per nominal: 192 from 64 to 256, 256 from 256 to 512.
    assert_eq!(
        (middle - small) * 256,
        (large - middle) * 192,
        "resumed walk steps {small} {middle} {large} are not linear"
    );
    // The control: restarting at the root after each wait revisits the
    // chain's published prefix, which is quadratic in the chain.
    let restarted = steps(512, true)?;
    assert!(
        restarted > large * 64,
        "restarted walk {restarted} against resumed {large}"
    );
    Ok(())
}
