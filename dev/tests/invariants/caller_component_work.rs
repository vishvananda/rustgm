//! Shared caller subjects cost graph edges, not the sum of every growing prefix.
use super::{Budget, ComponentKind, Components};
use crate::types::{
    ItemId, Kind, RegionId, Store,
    parameters::{Parameter, ParameterKind},
};
type Result = std::result::Result<(), Box<dyn std::error::Error>>;

#[test]
fn growing_roots_share_component_work_and_drop_only_clause_local_regions() -> Result {
    for count in [16, 128, 1024, 4096] {
        let mut types = Store::default();
        let id = types.intern_parameter(Parameter {
            owner: ItemId(7),
            index: 0,
            kind: ParameterKind::Type { sized: true },
        })?;
        let parameter = types.intern(Kind::Parameter(id))?;
        let local = types.intern(Kind::Reference {
            region: RegionId::bound(0, 0)?,
            referent: parameter,
            mutable: false,
        })?;
        let captured = types.intern(Kind::Reference {
            region: RegionId::bound(1, 0)?,
            referent: parameter,
            mutable: false,
        })?;
        let mut root = captured;
        let mut roots = Vec::new();
        for _ in 0..count {
            let elements = types.intern_list(&[root, captured, local, parameter])?;
            root = types.intern(Kind::Tuple(elements))?;
            roots.push(root);
        }
        let mut query = Components::default();
        let mut budget = Budget(0);
        let (regions, parameters, projections) =
            query.collect_roots(&types, &roots, 1, ComponentKind::TypeBound, &mut budget)?;
        assert_eq!(regions, [RegionId::bound(0, 0)?]);
        assert_eq!(parameters, [id]);
        assert!(projections.is_empty());
        assert!(budget.0 < 8 * count, "{count} roots: {} steps", budget.0);
        eprintln!("caller component walk: {count} roots, {} steps", budget.0);
    }
    Ok(())
}
