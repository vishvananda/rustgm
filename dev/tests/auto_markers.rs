//! Coinduction must close every field before it accepts a recursive marker bound.
use rustgm::types::{
    GenericArgs, ItemId, Kind, Primitive, RegionId, Store, TypeId,
    aggregate::{self, Definitions, Repr, Variant},
    arguments::Argument,
    auto_traits::{AutoTrait, Obligation, Query, Step},
    parameters::{Parameter, ParameterKind},
};

const U8: TypeId = TypeId::primitive(Primitive::U8);

#[derive(Default)]
struct Fixture {
    types: Store,
    definitions: Definitions,
}
impl Fixture {
    fn fields(&mut self, item: ItemId, fields: &[TypeId]) -> Result<(), &'static str> {
        let fields = self.types.intern_list(fields)?;
        self.definitions.insert(
            item,
            aggregate::Kind::Struct,
            Repr::default(),
            &[Variant {
                fields,
                discriminant: 0,
            }],
        )
    }
    fn reference(&mut self, ty: TypeId, mutable: bool) -> Result<TypeId, &'static str> {
        self.types.intern(Kind::Reference {
            mutable,
            region: RegionId::STATIC,
            referent: ty,
        })
    }
    fn pointer(&mut self, ty: TypeId) -> Result<TypeId, &'static str> {
        self.types.intern(Kind::Pointer {
            mutable: false,
            pointee: ty,
        })
    }
    fn prove(&mut self, ty: TypeId, marker: AutoTrait) -> Result<(bool, usize), &'static str> {
        let mut query = Query::new(Obligation { ty, marker });
        let Step::Ready(result) =
            query.advance(&mut self.types, &mut self.definitions, |_, _| {
                Ok(rustgm::types::auto_traits::Rule::Structural)
            })?
        else {
            return Err("expected a closed structural proof");
        };
        Ok((result, query.steps))
    }
    fn parameter(&mut self, owner: ItemId) -> Result<TypeId, &'static str> {
        let id = self.types.intern_parameter(Parameter {
            owner,
            index: 0,
            kind: ParameterKind::Type { sized: false },
        })?;
        self.types.intern(Kind::Parameter(id))
    }
}

#[test]
fn scalar_references_and_raw_pointers_follow_distinct_rules() -> Result<(), &'static str> {
    let mut f = Fixture::default();
    let text = TypeId::primitive(Primitive::Str);
    let shared = f.reference(text, false)?;
    let mutable = f.reference(U8, true)?;
    let pointer = f.pointer(U8)?;
    let zero = f.types.intern(Kind::Array {
        element: pointer,
        length: 0,
    })?;
    for (ty, expected) in [
        (U8, [true; 5]),
        (text, [true; 5]),
        (TypeId::NEVER, [true; 5]),
        (shared, [true; 5]),
        (mutable, [true, true, true, false, true]),
        (pointer, [false, false, true, true, true]),
        (zero, [false, false, true, true, true]),
    ] {
        for (marker, expected) in AutoTrait::ALL.into_iter().zip(expected) {
            assert_eq!(f.prove(ty, marker)?.0, expected, "{ty:?}: {marker:?}");
        }
    }
    Ok(())
}

#[test]
fn abstract_referents_request_the_transformed_marker() -> Result<(), &'static str> {
    let mut f = Fixture::default();
    let ty = f.parameter(ItemId(1))?;
    let shared = f.reference(ty, false)?;
    let mutable = f.reference(ty, true)?;
    let pointer = f.pointer(ty)?;
    for (subject, marker, needed) in [
        (shared, AutoTrait::Send, AutoTrait::Sync),
        (mutable, AutoTrait::Send, AutoTrait::Send),
        (mutable, AutoTrait::Sync, AutoTrait::Sync),
        (shared, AutoTrait::UnwindSafe, AutoTrait::RefUnwindSafe),
        (pointer, AutoTrait::UnwindSafe, AutoTrait::RefUnwindSafe),
        (mutable, AutoTrait::RefUnwindSafe, AutoTrait::RefUnwindSafe),
    ] {
        for answer in [false, true] {
            let mut query = Query::new(Obligation {
                ty: subject,
                marker,
            });
            let required = Obligation { ty, marker: needed };
            assert_eq!(
                query.advance(&mut f.types, &mut f.definitions, |_, _| Ok(
                    rustgm::types::auto_traits::Rule::Structural
                ))?,
                Step::Proof(required)
            );
            let before = query.steps;
            assert_eq!(
                query.advance(&mut f.types, &mut f.definitions, |_, _| Ok(
                    rustgm::types::auto_traits::Rule::Structural
                ))?,
                Step::Proof(required)
            );
            assert_eq!(query.steps, before);
            assert!(
                query
                    .answer(
                        Obligation {
                            ty: U8,
                            marker: needed
                        },
                        true
                    )
                    .is_err()
            );
            query.answer(required, answer)?;
            assert_eq!(
                query.advance(&mut f.types, &mut f.definitions, |_, _| Ok(
                    rustgm::types::auto_traits::Rule::Structural
                ))?,
                Step::Ready(answer)
            );
        }
    }
    for subject in [shared, mutable, pointer] {
        assert!(f.prove(subject, AutoTrait::Unpin)?.0);
    }
    Ok(())
}

#[test]
fn recursive_failures_reach_every_member_of_the_cycle() -> Result<(), &'static str> {
    for bad in [false, true] {
        let mut f = Fixture::default();
        let a = f.types.intern(Kind::Adt(ItemId(1), GenericArgs::EMPTY))?;
        let b = f.types.intern(Kind::Adt(ItemId(2), GenericArgs::EMPTY))?;
        let ra = f.reference(a, false)?;
        let rb = f.reference(b, false)?;
        let tail = if bad { f.pointer(U8)? } else { U8 };
        f.fields(ItemId(1), &[rb, tail])?;
        f.fields(ItemId(2), &[ra])?;
        let fields = f.types.intern_list(&[a, b])?;
        let root = f.types.intern(Kind::Tuple(fields))?;
        for subject in [root, a, b] {
            for marker in [AutoTrait::Send, AutoTrait::Sync] {
                assert_eq!(f.prove(subject, marker)?.0, !bad);
            }
        }
    }
    Ok(())
}

#[test]
fn callable_signatures_and_unused_arguments_do_not_create_dependencies() -> Result<(), &'static str>
{
    let mut f = Fixture::default();
    let parameter = f.parameter(ItemId(1))?;
    let inputs = f.types.intern_list(&[parameter])?;
    let signature = f.types.intern(Kind::Function {
        bound_regions: 0,
        parameters: inputs,
        result: parameter,
        unsafe_: false,
    })?;
    f.fields(ItemId(1), &[signature])?;
    let pointer = f.pointer(U8)?;
    let arguments = f.types.intern_arguments(&[Argument::Type(pointer)])?;
    let nominal = f.types.intern(Kind::Adt(ItemId(1), arguments))?;
    let function = f.types.intern(Kind::FunctionItem {
        item: ItemId(3),
        arguments,
        signature,
    })?;
    for subject in [signature, function, nominal] {
        for marker in AutoTrait::ALL {
            assert!(f.prove(subject, marker)?.0);
        }
    }
    Ok(())
}

#[test]
fn unfinished_fields_resume_without_publishing_a_positive_cycle() -> Result<(), &'static str> {
    let mut f = Fixture::default();
    let item = ItemId(1);
    let ty = f.types.intern(Kind::Adt(item, GenericArgs::EMPTY))?;
    let mut query = Query::new(Obligation {
        ty,
        marker: AutoTrait::Sync,
    });
    assert_eq!(
        query.advance(&mut f.types, &mut f.definitions, |_, _| Ok(
            rustgm::types::auto_traits::Rule::Structural
        ))?,
        Step::Fields(item)
    );
    let reference = f.reference(ty, false)?;
    let pointer = f.pointer(U8)?;
    f.fields(item, &[reference, pointer])?;
    assert_eq!(
        query.advance(&mut f.types, &mut f.definitions, |_, _| Ok(
            rustgm::types::auto_traits::Rule::Structural
        ))?,
        Step::Ready(false)
    );
    Ok(())
}

#[test]
fn shared_diamonds_and_deep_chains_have_linear_work() -> Result<(), &'static str> {
    for count in [64_u32, 256, 16_384] {
        let mut f = Fixture::default();
        let mut tail = f.pointer(U8)?;
        for index in 0..count {
            let item = ItemId(index);
            f.fields(item, &[tail, tail])?;
            tail = f.types.intern(Kind::Adt(item, GenericArgs::EMPTY))?;
        }
        let (result, steps) = f.prove(tail, AutoTrait::Send)?;
        assert!(!result);
        assert!(steps < 10 * count as usize + 10, "{count}: {steps}");
    }
    Ok(())
}
