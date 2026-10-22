//! Each generated impl has its own assumptions, including under recursive references.
use crate::qualify as rustc;
use rustgm::support::hash::HashMap;
use rustgm::types::{
    ItemId, Kind, RegionId, Store, TypeId,
    aggregate::{self, Definitions, Repr, Variant},
    arguments::Argument,
    copy::{Copies, Properties as Copy},
    equality::{Equality, Properties as Equal},
    parameters::{
        Parameter, ParameterId, ParameterKind,
        environment::{Abilities, Environment},
    },
};
use std::sync::Arc;
type TestResult = Result<(), Box<dyn std::error::Error>>;
fn parameter(types: &mut Store, owner: u32) -> Result<(ParameterId, TypeId), &'static str> {
    let id = types.intern_parameter(Parameter {
        owner: ItemId(owner),
        index: 0,
        kind: ParameterKind::Type { sized: true },
    })?;
    Ok((id, types.intern(Kind::Parameter(id))?))
}
fn define(
    types: &mut Store,
    definitions: &mut Definitions,
    owner: u32,
    field: TypeId,
    packed: bool,
) -> Result<(), &'static str> {
    let fields = types.intern_list(&[field])?;
    definitions.insert(
        ItemId(owner),
        aggregate::Kind::Struct,
        Repr {
            packed: packed.then_some(1),
            ..Repr::default()
        },
        &[Variant {
            fields,
            discriminant: 0,
        }],
    )
}

#[test]
fn clone_does_not_borrow_the_separate_copy_impls_stronger_assumptions() -> TestResult {
    rustc::qualify(
        "derive_separate_impls",
        "#[repr(packed)]#[derive(Clone,Copy)]struct A<T>{v:T}#[derive(Clone,Copy)]struct S<T>{v:A<T>}",
        false,
    )?;
    let mut types = Store::default();
    let mut definitions = Definitions::default();
    let (id, outer) = parameter(&mut types, 0)?;
    let (_, inner) = parameter(&mut types, 1)?;
    define(&mut types, &mut definitions, 1, inner, true)?;
    let arguments = types.intern_arguments(&[Argument::Type(outer)])?;
    let field = types.intern(Kind::Adt(ItemId(1), arguments))?;
    let selected = HashMap::from_iter([(
        ItemId(1),
        Copy {
            copy: true,
            clone_: true,
        }
        .into(),
    )]);
    let equal = HashMap::from_iter([(
        ItemId(1),
        Equal {
            partial_eq: true,
            eq: true,
        },
    )]);
    for copy in [false, true, false] {
        let environment = Environment::new(&types, &[(id, Abilities { copy, clone_: true })])?;
        let mut query = Copies::for_derive(Some(Arc::new(environment)));
        assert_eq!(
            query.demand(&types, &definitions, &selected, field)?.clone_,
            copy
        );
        let mut equality = Equality::for_derive(
            ItemId(0),
            Equal {
                partial_eq: true,
                eq: true,
            },
        );
        let result = equality.demand_with_copy(&types, &definitions, &equal, field, |ty| {
            query
                .demand(&types, &definitions, &selected, ty)
                .map(|p| p.copy)
        })?;
        assert_eq!(result.partial_eq, copy);
        assert_eq!(result.eq, copy);
    }
    assert_eq!(definitions.instantiation_steps(), 0);
    Ok(())
}

#[test]
fn growing_recursive_reference_fields_use_their_conditional_contract() -> TestResult {
    rustc::qualify(
        "derive_growing_reference",
        "#[derive(PartialEq,Eq)]struct S<'a,T>{p:&'a S<'a,[T;1]>,v:T}",
        true,
    )?;
    let mut types = Store::default();
    let mut definitions = Definitions::default();
    let (_, ty) = parameter(&mut types, 0)?;
    let array = types.intern(Kind::Array {
        element: ty,
        length: 1,
    })?;
    let arguments = types.intern_arguments(&[Argument::Type(array)])?;
    let inner = types.intern(Kind::Adt(ItemId(0), arguments))?;
    let field = types.intern(Kind::Reference {
        mutable: false,
        region: RegionId::STATIC,
        referent: inner,
    })?;
    define(&mut types, &mut definitions, 0, field, false)?;
    let both = Equal {
        partial_eq: true,
        eq: true,
    };
    let selected = HashMap::from_iter([(ItemId(0), both)]);
    let mut query = Equality::for_derive(ItemId(0), both);
    let before = types.len();
    for _ in 0..100 {
        assert_eq!(query.demand(&types, &definitions, &selected, field)?, both);
    }
    assert!(query.work < 30, "{}", query.work);
    assert_eq!(types.len(), before);
    assert_eq!(definitions.instantiation_steps(), 0);
    // This proof is conditional on this declaration's generated predicates.
    let (_, foreign) = parameter(&mut types, 1)?;
    assert!(
        query
            .demand(&types, &definitions, &selected, foreign)
            .is_err()
    );
    Ok(())
}
