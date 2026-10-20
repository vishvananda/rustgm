//! Declaration rules describe size conditions without expanding concrete storage.
use rustgm::types::{
    GenericArgs, ItemId, Kind, Primitive, RegionId, Store, TypeId,
    aggregate::{self, Definitions, Repr, Variant, finiteness::Storage},
    arguments::Argument,
    parameters::{
        Parameter, ParameterKind,
        environment::{Abilities, Environment},
        predicates::{Bound, Predicate},
        shape::Shapes,
    },
    projection,
    sizedness::{Rule, Rules},
    traits,
};
use std::sync::Arc;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const BYTE: TypeId = TypeId::primitive(Primitive::U8);
const STR: TypeId = TypeId::primitive(Primitive::Str);

#[derive(Default)]
struct Fixture {
    types: Store,
    definitions: Definitions,
    rules: Rules,
}
impl Fixture {
    fn parameter(&mut self, item: u32, index: u32) -> Result<TypeId> {
        let parameter = self.types.intern_parameter(Parameter {
            owner: ItemId(item),
            index,
            kind: ParameterKind::Type { sized: false },
        })?;
        Ok(self.types.intern(Kind::Parameter(parameter))?)
    }
    fn nominal(&mut self, item: u32, args: &[Argument]) -> Result<TypeId> {
        let args = self.types.intern_arguments(args)?;
        Ok(self.types.intern(Kind::Adt(ItemId(item), args))?)
    }
    fn define(&mut self, item: u32, fields: &[TypeId]) -> Result {
        let fields = self.types.intern_list(fields)?;
        Ok(self.definitions.insert(
            ItemId(item),
            aggregate::Kind::Struct,
            Repr::default(),
            &[Variant {
                fields,
                discriminant: 0,
            }],
        )?)
    }
    fn prepare(&mut self, item: u32) -> Result<Option<ItemId>> {
        Ok(self.rules.prepare(
            &mut self.types,
            &self.definitions,
            ItemId(item),
            |_, _, _| Ok(false),
        )?)
    }
    fn seal(&mut self) -> Result {
        Ok(self
            .types
            .seal_nominal_sizes(Arc::new(std::mem::take(&mut self.rules)))?)
    }
}

#[test]
fn missing_fields_are_dependencies_and_completed_rules_are_reused() -> Result {
    let mut f = Fixture::default();
    let tail = f.nominal(1, &[])?;
    f.define(0, &[BYTE, tail])?;
    assert_eq!(f.prepare(0)?, Some(ItemId(1)));
    assert_eq!(f.rules.get(ItemId(0)), None);
    f.define(1, &[STR])?;
    assert_eq!(f.prepare(0)?, None);
    assert_eq!(f.rules.get(ItemId(0)), Some(Rule::Unsized));
    let steps = f.rules.steps;
    assert_eq!(f.prepare(0)?, None);
    assert_eq!(f.rules.steps, steps);
    assert_eq!(f.definitions.instantiation_steps(), 0);
    Ok(())
}

#[test]
fn suspended_rule_preparation_preserves_its_frontier_and_shared_completions() -> Result {
    for count in [128, 256, 512, 1024] {
        let mut f = Fixture::default();
        let mut next = f.prepare(0)?;
        for owner in 0..count {
            assert_eq!(next, Some(ItemId(owner)));
            let parameter = f.parameter(owner, 0)?;
            let field = if owner + 1 == count {
                parameter
            } else {
                f.nominal(owner + 1, &[Argument::Type(parameter)])?
            };
            f.define(owner, &[field])?;
            if owner + 1 == count {
                // Another query can complete a shared schema during the wait.
                assert_eq!(f.prepare(owner)?, None);
            }
            next = f.prepare(0)?;
        }
        assert_eq!(next, None);
        assert_eq!(f.rules.get(ItemId(0)), Some(Rule::Argument(0)));
        assert!(f.rules.steps < count as usize * 12, "{}", f.rules.steps);
        assert_eq!(f.definitions.instantiation_steps(), 0);
    }
    Ok(())
}

#[test]
fn nested_rules_select_complete_argument_slots_and_preserve_other_parameters() -> Result {
    let mut f = Fixture::default();
    let inner = f.parameter(0, 1)?;
    f.define(0, &[BYTE, inner])?;
    let outer = f.parameter(1, 0)?;
    let unrelated = f.parameter(1, 1)?;
    let nested = f.nominal(
        0,
        &[
            Argument::Lifetime(RegionId::bound(0, 0)?),
            Argument::Type(outer),
        ],
    )?;
    f.define(1, &[nested])?;
    assert_eq!(f.prepare(1)?, None);
    assert_eq!(f.rules.get(ItemId(0)), Some(Rule::Argument(1)));
    assert_eq!(f.rules.get(ItemId(1)), Some(Rule::Argument(0)));
    let generic = f.nominal(1, &[Argument::Type(outer), Argument::Type(unrelated)])?;
    let sized = f.nominal(1, &[Argument::Type(BYTE), Argument::Type(STR)])?;
    let unsized_ty = f.nominal(1, &[Argument::Type(STR), Argument::Type(BYTE)])?;
    f.seal()?;
    assert_eq!(f.types.is_sized(sized), Some(true));
    assert_eq!(f.types.is_sized(unsized_ty), Some(false));
    assert_eq!(f.types.is_sized(generic), Some(false));
    let Some(Kind::Parameter(a)) = f.types.kind(outer) else {
        return Err("missing outer parameter".into());
    };
    let Some(Kind::Parameter(b)) = f.types.kind(unrelated) else {
        return Err("missing unused parameter".into());
    };
    for (bound, expected) in [(outer, true), (unrelated, false)] {
        let environment = Environment::new(
            &f.types,
            &[(a, Abilities::default()), (b, Abilities::default())],
        )?
        .with_clauses(
            &f.types,
            &[Predicate {
                ty: bound,
                bound: Bound::Sized,
            }
            .into()],
        )?;
        let properties = Shapes::new(Arc::new(environment)).demand(&f.types, generic)?;
        assert_eq!(properties.sized, expected);
        assert!(properties.well_formed);
    }
    assert_eq!(f.definitions.instantiation_steps(), 0);
    Ok(())
}

#[test]
fn sealing_updates_existing_structural_parents_and_new_instances() -> Result {
    let mut f = Fixture::default();
    let nominal = f.nominal(0, &[])?;
    let array = f.types.intern(Kind::Array {
        element: nominal,
        length: 0,
    })?;
    let fields = f.types.intern_list(&[nominal, BYTE])?;
    let bad_tuple = f.types.intern(Kind::Tuple(fields))?;
    let fields = f.types.intern_list(&[BYTE, nominal])?;
    let good_tuple = f.types.intern(Kind::Tuple(fields))?;
    let pointer = f.types.intern(Kind::Pointer {
        pointee: nominal,
        mutable: false,
    })?;
    assert_eq!(f.types.is_well_formed(array), Some(true));
    f.define(0, &[BYTE, STR])?;
    f.prepare(0)?;
    f.seal()?;
    for ty in [nominal, good_tuple] {
        assert_eq!(f.types.is_sized(ty), Some(false));
        assert_eq!(f.types.is_well_formed(ty), Some(true));
    }
    for ty in [array, bad_tuple] {
        assert_eq!(f.types.is_well_formed(ty), Some(false));
    }
    assert_eq!(f.types.is_sized(pointer), Some(true));
    assert_eq!(f.types.contains_nominals(pointer), Some(true));
    assert_eq!(f.types.contains_nominals(BYTE), Some(false));
    let fresh = f.types.intern(Kind::Array {
        element: nominal,
        length: 7,
    })?;
    assert_eq!(f.types.is_well_formed(fresh), Some(false));
    assert!(
        f.types
            .seal_nominal_sizes(Arc::new(Rules::default()))
            .is_err()
    );
    Ok(())
}

#[test]
fn owner_assumptions_do_not_leak_between_rules() -> Result {
    let mut f = Fixture::default();
    let a = f.parameter(0, 0)?;
    let b = f.parameter(1, 0)?;
    f.define(0, &[a])?;
    f.define(1, &[b])?;
    f.rules
        .prepare(&mut f.types, &f.definitions, ItemId(0), |_, owner, ty| {
            Ok(owner == ItemId(0) && ty == a)
        })?;
    f.prepare(1)?;
    assert_eq!(f.rules.get(ItemId(0)), Some(Rule::Sized));
    assert_eq!(f.rules.get(ItemId(1)), Some(Rule::Argument(0)));
    Ok(())
}

#[test]
fn forward_generic_chains_reduce_once_without_concrete_field_expansion() -> Result {
    for count in [512, 1024, 2048, 4096] {
        let mut f = Fixture::default();
        for owner in 0..count {
            let parameter = f.parameter(owner, 0)?;
            let field = if owner + 1 == count {
                parameter
            } else {
                f.nominal(owner + 1, &[Argument::Type(parameter)])?
            };
            f.define(owner, &[field])?;
        }
        let before = f.types.len();
        f.prepare(0)?;
        let steps = f.rules.steps;
        assert!(steps < count as usize * 10, "{count}: {steps}");
        for owner in (0..count).rev() {
            f.prepare(owner)?;
            assert_eq!(f.rules.get(ItemId(owner)), Some(Rule::Argument(0)));
        }
        assert_eq!(f.rules.steps, steps);
        assert_eq!(f.types.len(), before);
        assert_eq!(f.definitions.instantiation_steps(), 0);
    }
    Ok(())
}

#[test]
fn closed_shared_tails_are_not_rescanned_for_each_owner() -> Result {
    for count in [512, 1024, 2048, 4096] {
        let mut f = Fixture::default();
        let mut tail = STR;
        for _ in 0..count {
            let fields = f.types.intern_list(&[BYTE, tail])?;
            tail = f.types.intern(Kind::Tuple(fields))?;
        }
        for owner in 0..count {
            f.define(owner, &[BYTE, tail])?;
        }
        for owner in 0..count {
            f.prepare(owner)?;
            assert_eq!(f.rules.get(ItemId(owner)), Some(Rule::Unsized));
        }
        assert!(f.rules.steps < count as usize * 10, "{}", f.rules.steps);
        assert_eq!(f.definitions.instantiation_steps(), 0);
    }
    Ok(())
}

#[test]
fn positive_size_cycles_still_require_the_independent_storage_gate() -> Result {
    let mut f = Fixture::default();
    let a = f.nominal(0, &[])?;
    let b = f.nominal(1, &[])?;
    f.define(0, &[b])?;
    f.define(1, &[a])?;
    f.prepare(0)?;
    assert_eq!(f.rules.get(ItemId(0)), Some(Rule::Sized));
    assert_eq!(f.rules.get(ItemId(1)), Some(Rule::Sized));
    let schemas = [
        (ItemId(0), GenericArgs::EMPTY),
        (ItemId(1), GenericArgs::EMPTY),
    ];
    assert!(Storage::infer(&f.types, &f.definitions, &schemas).is_err());
    Ok(())
}

#[test]
fn projected_conditions_substitute_inputs_without_inventing_result_permission() -> Result {
    let mut f = Fixture::default();
    let t = f.parameter(0, 0)?;
    let application = f.types.intern_trait_application(traits::Application {
        item: ItemId(10),
        arguments: GenericArgs::EMPTY,
    })?;
    f.types.declare_projection(
        ItemId(11),
        projection::Schema {
            trait_item: ItemId(10),
            sized: false,
            copies: rustgm::types::copy::Properties::default(),
        },
    )?;
    let projected = f.types.intern(Kind::Projection(projection::Application {
        item: ItemId(11),
        subject: t,
        trait_application: application,
        arguments: GenericArgs::EMPTY,
    }))?;
    f.define(0, &[projected])?;
    let u = f.parameter(1, 0)?;
    let instance = f.nominal(0, &[Argument::Type(u)])?;
    f.define(1, &[instance])?;
    f.prepare(1)?;
    let expected = f.types.intern(Kind::Projection(projection::Application {
        item: ItemId(11),
        subject: u,
        trait_application: application,
        arguments: GenericArgs::EMPTY,
    }))?;
    assert_eq!(f.rules.get(ItemId(1)), Some(Rule::Projection(expected)));
    // A permission on one owner's complete subject cannot become a global
    // physical summary or a premise for another owner using the same subject.
    let instance = f.nominal(0, &[Argument::Type(BYTE)])?;
    let expected = f.types.intern(Kind::Projection(projection::Application {
        item: ItemId(11),
        subject: BYTE,
        trait_application: application,
        arguments: GenericArgs::EMPTY,
    }))?;
    f.define(2, &[instance])?;
    f.rules
        .prepare(&mut f.types, &f.definitions, ItemId(2), |_, owner, ty| {
            Ok(owner == ItemId(2) && ty == instance)
        })?;
    f.define(3, &[instance])?;
    f.prepare(3)?;
    assert_eq!(f.rules.get(ItemId(2)), Some(Rule::Sized));
    assert_eq!(f.rules.get(ItemId(3)), Some(Rule::Projection(expected)));
    assert_eq!(f.definitions.instantiation_steps(), 0);
    Ok(())
}

#[test]
fn sealing_refreshes_shared_trait_arguments_before_their_projected_parents() -> Result {
    let mut f = Fixture::default();
    let nominal = f.nominal(0, &[])?;
    let array = f.types.intern(Kind::Array {
        element: nominal,
        length: 0,
    })?;
    let arguments = f.types.intern_arguments(&[Argument::Type(array)])?;
    let application = f.types.intern_trait_application(traits::Application {
        item: ItemId(10),
        arguments,
    })?;
    f.types.declare_projection(
        ItemId(11),
        projection::Schema {
            trait_item: ItemId(10),
            sized: true,
            copies: rustgm::types::copy::Properties::default(),
        },
    )?;
    let projected = f.types.intern(Kind::Projection(projection::Application {
        item: ItemId(11),
        subject: BYTE,
        trait_application: application,
        arguments: GenericArgs::EMPTY,
    }))?;
    assert_eq!(f.types.is_well_formed(projected), Some(true));
    f.define(0, &[STR])?;
    f.prepare(0)?;
    f.seal()?;
    assert_eq!(f.types.is_well_formed(projected), Some(false));
    let later = f.types.intern(Kind::Projection(projection::Application {
        item: ItemId(11),
        subject: TypeId::UNIT,
        trait_application: application,
        arguments: GenericArgs::EMPTY,
    }))?;
    assert_eq!(f.types.is_well_formed(later), Some(false));
    Ok(())
}
