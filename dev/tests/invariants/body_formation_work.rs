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

#[test]
fn component_count_and_target_count_scale_independently() -> Result {
    for (components, targets, sizes) in [
        (false, true, &[16_u16, 128, 1024, 4096][..]),
        (true, false, &[16, 128, 1024, 4096][..]),
        (true, true, &[16, 64, 128][..]),
    ] {
        for &count in sizes {
            formation_axes(count, components, targets)?;
        }
    }
    Ok(())
}

fn formation_axes(count: u16, components: bool, targets: bool) -> Result {
    let mut types = Store::default();
    let mut root = TypeId::UNIT;
    let mut roots = Vec::new();
    let mut ids = Vec::new();
    for index in 0..count {
        let id = types.intern_parameter(Parameter {
            owner: ItemId(7),
            index: if components { u32::from(index) } else { 0 },
            kind: ParameterKind::Type { sized: true },
        })?;
        ids.push(id);
        let parameter = types.intern(Kind::Parameter(id))?;
        let list = types.intern_list(&[root, parameter])?;
        root = types.intern(Kind::Tuple(list))?;
        roots.push(types.intern(Kind::Reference {
            region: RegionId::bound(1, if targets { index } else { 0 })?,
            referent: root,
            mutable: false,
        })?);
    }
    let bounds = Bounds::default();
    let templates = Templates::default();
    let mut query = Formation::new(&types, &bounds, &templates);
    let mut output = 0;
    for (index, root) in roots.into_iter().enumerate() {
        let found = query.requirements(root, 0)?;
        assert!(found.scalars.regions.is_empty());
        assert!(found.projections.is_empty());
        let target = RegionId::bound(0, if targets { u16::try_from(index)? } else { 0 })?;
        let expected = if components && targets {
            &ids[..=index]
        } else {
            &ids[index..=index]
        };
        let expected: crate::support::hash::HashSet<_> =
            expected.iter().map(|&id| (id, target)).collect();
        let actual: crate::support::hash::HashSet<_> =
            found.scalars.parameters.iter().copied().collect();
        assert_eq!(actual, expected);
        assert_eq!(actual.len(), found.scalars.parameters.len());
        output += actual.len();
    }
    let steps = query.scan.budget.0;
    // Fixed-width radix sharing bounds growing prefixes independently of the
    // output relation. Growing both axes really requires a quadratic output.
    assert!(
        steps < 160 * usize::from(count) + 8 * output,
        "{count}: {steps} steps, {output} debts"
    );
    eprintln!(
        "formation axes: count={count}, components={components}, targets={targets}, steps={steps}, output={output}"
    );
    Ok(())
}

#[test]
fn late_target_promotion_rebuilds_once_and_preserves_published_debts() -> Result {
    for count in [16, 128, 1024, 4096] {
        let mut types = Store::default();
        let id = types.intern_parameter(Parameter {
            owner: ItemId(7),
            index: 0,
            kind: ParameterKind::Type { sized: true },
        })?;
        let t = types.intern(Kind::Parameter(id))?;
        let mut prefix = t;
        let mut roots = Vec::new();
        for _ in 0..count {
            let elements = types.intern_list(&[prefix, t])?;
            prefix = types.intern(Kind::Tuple(elements))?;
            roots.push(types.intern(Kind::Reference {
                region: RegionId::bound(1, 0)?,
                referent: prefix,
                mutable: false,
            })?);
        }
        let other = types.intern(Kind::Reference {
            region: RegionId::bound(1, 1)?,
            referent: prefix,
            mutable: false,
        })?;
        let bounds = Bounds::default();
        let templates = Templates::default();
        let mut query = Formation::new(&types, &bounds, &templates);
        for (index, &root) in roots.iter().enumerate() {
            let found = query.requirements(root, 0)?;
            assert_eq!(found.scalars.parameters.len(), usize::from(index == 0));
        }
        assert_eq!(
            query.requirements(other, 0)?.scalars.parameters,
            [(id, RegionId::bound(0, 1)?)]
        );
        // Revisit the old component under the old target through a new root.
        // Formation's structural deduplication cannot mask duplicate publication.
        super::Demand::require(
            &mut query.demand,
            &mut query.scan,
            &types,
            t,
            RegionId::bound(0, 0)?,
            1,
            super::ComponentKind::TypeBound,
        )?;
        assert_eq!(query.scan.parameter_pairs, [(id, RegionId::bound(0, 1)?)]);
        assert!(
            query.scan.budget.0 < 60 * count,
            "{count}: {} steps",
            query.scan.budget.0
        );
    }
    Ok(())
}

#[test]
fn changing_component_policy_promotes_without_losing_referent_requirements() -> Result {
    for reverse in [false, true] {
        let mut types = Store::default();
        let id = types.intern_parameter(Parameter {
            owner: ItemId(7),
            index: 0,
            kind: ParameterKind::Type { sized: true },
        })?;
        let t = types.intern(Kind::Parameter(id))?;
        let root = types.intern(Kind::Reference {
            region: RegionId::bound(1, 0)?,
            referent: t,
            mutable: false,
        })?;
        let mut scan = super::Scan::default();
        let mut demand = super::Deferred::default();
        let policies = if reverse {
            [
                super::ComponentKind::TypeBound,
                super::ComponentKind::Outlives,
            ]
        } else {
            [
                super::ComponentKind::Outlives,
                super::ComponentKind::TypeBound,
            ]
        };
        for kind in policies {
            super::Demand::require(
                &mut demand,
                &mut scan,
                &types,
                root,
                RegionId::STATIC,
                1,
                kind,
            )?;
        }
        assert_eq!(scan.parameter_pairs, [(id, RegionId::STATIC)]);
        assert_eq!(scan.pairs, [(RegionId::bound(0, 0)?, RegionId::STATIC)]);
    }
    Ok(())
}
