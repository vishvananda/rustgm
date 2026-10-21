//! Canonical sets agree with the independent component walk across scopes and policies.
use super::{Atom, Budget, ComponentKind, HashSet, Sets};
use crate::types::{
    GenericArgs, ItemId, Kind, RegionId, Store, TypeId,
    arguments::Argument,
    copy::Properties,
    outlives::components::Components,
    parameters::{Parameter, ParameterKind},
    projection::{Application, Schema},
    traits,
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn inputs(types: &mut Store) -> Result<Vec<TypeId>> {
    let id = types.intern_parameter(Parameter {
        owner: ItemId(1),
        index: 0,
        kind: ParameterKind::Type { sized: true },
    })?;
    let parameter = types.intern(Kind::Parameter(id))?;
    types.declare_projection(
        ItemId(2),
        Schema {
            trait_item: ItemId(3),
            sized: true,
            copies: Properties::default(),
        },
    )?;
    let trait_application = types.intern_trait_application(traits::Application {
        item: ItemId(3),
        arguments: GenericArgs::EMPTY,
    })?;
    let projected = types.intern(Kind::Projection(Application {
        item: ItemId(2),
        subject: parameter,
        trait_application,
        arguments: GenericArgs::EMPTY,
    }))?;
    let mut values = vec![TypeId::UNIT, parameter, projected];
    for depth in 0..3 {
        let region = RegionId::bound(depth, 0)?;
        let reference = types.intern(Kind::Reference {
            region,
            referent: projected,
            mutable: false,
        })?;
        let arguments = types.intern_arguments(&[
            Argument::Lifetime(region),
            Argument::Type(parameter),
            Argument::Type(reference),
        ])?;
        values.push(reference);
        values.push(types.intern(Kind::Adt(ItemId(4), arguments))?);
        values.push(types.intern(Kind::Array {
            element: reference,
            length: 0,
        })?);
        let parameters = types.intern_list(&[reference, parameter, projected])?;
        let signature = types.intern(Kind::Function {
            bound_regions: 1,
            parameters,
            result: reference,
            unsafe_: false,
            convention: crate::types::callable::Convention::RUST,
        })?;
        values.push(signature);
        values.push(types.intern(Kind::FunctionItem {
            item: ItemId(5),
            signature,
            arguments,
        })?);
    }
    let list = types.intern_list(&values)?;
    values.push(types.intern(Kind::Tuple(list))?);
    Ok(values)
}

#[test]
fn policies_depths_and_targets_preserve_exact_components() -> Result {
    let mut types = Store::default();
    let roots = inputs(&mut types)?;
    for reverse in [false, true] {
        let mut query = Sets::default();
        let mut budget = Budget::default();
        let mut expected = HashSet::new();
        let mut actual = HashSet::new();
        let policies = if reverse {
            [ComponentKind::TypeBound, ComponentKind::Outlives]
        } else {
            [ComponentKind::Outlives, ComponentKind::TypeBound]
        };
        for kind in policies {
            for depth in 0..3 {
                for target in [RegionId::STATIC, RegionId::bound(0, 0)?] {
                    for &root in &roots {
                        let (regions, parameters, projections) = Components::default()
                            .collect_roots(&types, &[root], depth, kind, &mut Budget::default())?;
                        expected.extend(
                            regions
                                .into_iter()
                                .map(Atom::Region)
                                .chain(parameters.into_iter().map(Atom::Parameter))
                                .chain(projections.into_iter().map(Atom::Projection))
                                .map(|atom| (target, atom)),
                        );
                        let found = query.extend(&types, root, depth, kind, target, &mut budget)?;
                        for atom in found {
                            assert!(actual.insert((target, atom)), "component emitted twice");
                        }
                        assert_eq!(actual, expected);
                    }
                }
            }
        }
    }
    Ok(())
}

#[test]
fn shared_wide_lists_are_summarized_once_per_scope_and_policy() -> Result {
    for width in [16_u16, 128, 1024] {
        let mut types = Store::default();
        let parameter = types.intern_parameter(Parameter {
            owner: ItemId(1),
            index: 0,
            kind: ParameterKind::Type { sized: true },
        })?;
        let ty = types.intern(Kind::Parameter(parameter))?;
        let parameters = types.intern_list(&vec![ty; usize::from(width)])?;
        let mut roots = Vec::new();
        for slot in 0..width {
            let result = types.intern(Kind::Reference {
                region: RegionId::bound(2, slot)?,
                referent: ty,
                mutable: false,
            })?;
            roots.push(types.intern(Kind::Function {
                bound_regions: 0,
                parameters,
                result,
                unsafe_: false,
                convention: crate::types::callable::Convention::RUST,
            })?);
        }
        let mut query = Sets::default();
        let mut budget = Budget::default();
        for (slot, root) in roots.into_iter().enumerate() {
            let target = RegionId::bound(0, u16::try_from(slot)?)?;
            let found: HashSet<_> = query
                .extend(
                    &types,
                    root,
                    1,
                    ComponentKind::TypeBound,
                    target,
                    &mut budget,
                )?
                .into_iter()
                .collect();
            assert_eq!(
                found,
                HashSet::from([Atom::Parameter(parameter), Atom::Region(target)])
            );
        }
        assert!(
            budget.0 < 80 * usize::from(width),
            "{width}: {} steps",
            budget.0
        );
    }
    Ok(())
}
