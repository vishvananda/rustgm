//! Formation shares structural and component edges while validating every root's binder.
use super::{Bounds, Formation, Templates};
use crate::types::{
    ItemId, Kind, RegionId, Store, TypeId,
    parameters::{Parameter, ParameterKind},
};
type Result = std::result::Result<(), Box<dyn std::error::Error>>;

#[test]
fn growing_reference_debts_share_component_prefixes_and_keep_distinct_targets() -> Result {
    for count in [16, 128, 1024, 4096] {
        let mut types = Store::default();
        let id = types.intern_parameter(Parameter {
            owner: ItemId(7),
            index: 0,
            kind: ParameterKind::Type { sized: true },
        })?;
        let t = types.intern(Kind::Parameter(id))?;
        let mut root = t;
        let mut roots = Vec::new();
        for _ in 0..count {
            let elements = types.intern_list(&[root, t])?;
            root = types.intern(Kind::Tuple(elements))?;
            for slot in [0, 1] {
                roots.push(types.intern(Kind::Reference {
                    region: RegionId::bound(1, slot)?,
                    referent: root,
                    mutable: false,
                })?);
            }
        }
        let bounds = Bounds::default();
        let templates = Templates::default();
        let mut query = Formation::new(&types, &bounds, &templates);
        let mut found = Vec::new();
        for root in roots {
            found.extend_from_slice(query.requirements(root, 0)?.scalars.parameters);
        }
        assert_eq!(
            found,
            [(id, RegionId::bound(0, 0)?), (id, RegionId::bound(0, 1)?)]
        );
        let steps = query.scan.budget.0;
        assert!(steps < 40 * count, "{count} roots: {steps} steps");
        eprintln!("body formation: {count} prefixes with two targets, {steps} steps");
    }
    Ok(())
}

#[test]
fn repeated_roots_validate_empty_binders_and_charge_even_when_the_graph_is_seen() -> Result {
    let mut types = Store::default();
    let root = types.intern(Kind::Reference {
        region: RegionId::bound(0, 1)?,
        referent: TypeId::UNIT,
        mutable: false,
    })?;
    let bounds = Bounds::default();
    let templates = Templates::default();
    let mut query = Formation::new(&types, &bounds, &templates);
    assert!(query.requirements(root, 2)?.scalars.parameters.is_empty());
    assert_eq!(
        query.requirements(root, 0).err(),
        Some("clause formation references an unbound lifetime")
    );
    query.scan.budget.0 = super::super::LIMIT;
    assert_eq!(
        query.requirements(root, 2).err(),
        Some("type outlives work budget exceeded")
    );
    Ok(())
}
