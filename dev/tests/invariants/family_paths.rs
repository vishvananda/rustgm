//! Canonical applications preserve distinct parent and family inputs from resolved syntax.
use super::{Argument, Checker, Kind, RegionId, TestResult, report, reserve};
use crate::{
    sema::{
        associated::OwnerKind, nominals::traits::normal_form, traits::implementations::Availability,
    },
    types::{
        ItemId, Primitive, TypeId,
        arguments::Constant,
        parameters::{
            environment::{Abilities, Environment},
            predicates::{Bound, Predicate},
        },
        substitute::{Normalizer, Resolution, Step},
    },
};

const QUALIFIED: &str = "trait Q<'p>{type Out<'x,T,const N:usize>;}trait R<'p,'x>:Q<'p> where <Self as Q<'p>>::Out<'x,u8,3>:Copy, <Self as Q<'p>>::Out<'p,u16,5>:Copy{}";
const SHORTHAND: &str = "trait Q<'p>{type Out<'x,T,const N:usize>;}trait R<'p,'x>:Q<'p> where Self::Out<'x,u8,3>:Copy, Self::Out<'p,u16,5>:Copy{}";

#[test]
fn canonical_normalization_keeps_family_inputs_and_restores_selected_result_lifetimes() -> TestResult
{
    let report = report("trait Q<'p>{type Out<'a,T>;}")?;
    let mut checker = Checker::new(&report)?;
    reserve(&mut checker)?;
    let member = checker.facts.associated.members()[0];
    let owner = checker.facts.associated.owners()[member.owner.0 as usize].source_owner()?;
    let OwnerKind::Trait { definition, .. } = owner.kind else {
        return Err("missing family trait owner".into());
    };
    let key = report.resolution.definitions[member.definition.0 as usize]
        .node
        .ok_or("missing member source")?;
    checker.declare_associated_type(member.definition, key, owner, definition)?;
    checker.complete_associated_result_bounds(member.definition, key, &mut 0, 100_000)?;
    let template = checker
        .facts
        .associated
        .type_contract(member.definition)
        .ok_or("missing family contract")?
        .projection;
    let facts = &mut checker.facts;
    let Some(Kind::Projection(template)) = facts.types.kind(template) else {
        return Err("missing projection".into());
    };
    let own = facts.types.intern_arguments(&[
        Argument::Lifetime(RegionId::bound(0, 1)?),
        Argument::Type(TypeId::primitive(Primitive::U8)),
    ])?;
    let application = crate::types::projection::Application {
        arguments: own,
        ..template
    };
    let root = facts.types.intern(Kind::Projection(application))?;
    let result = facts.types.intern(Kind::Reference {
        mutable: false,
        region: RegionId::bound(0, 1)?,
        referent: TypeId::primitive(Primitive::U8),
    })?;
    let premise = Predicate {
        ty: root,
        bound: Bound::ProjectionEqual(result),
    };
    let parent = Predicate {
        ty: application.subject,
        bound: Bound::Trait(application.trait_application),
    };
    let Some(Kind::Parameter(subject)) = facts.types.kind(application.subject) else {
        return Err("missing trait Self parameter".into());
    };
    let environment = std::sync::Arc::new(
        Environment::new(&facts.types, &[(subject, Abilities::default())])?
            .with_clauses(&facts.types, &[parent.into(), premise.into()])?,
    );
    for (input, expected) in [
        (Primitive::U8, Resolution::Type(result)),
        (Primitive::U16, Resolution::Opaque),
    ] {
        let own = facts.types.intern_arguments(&[
            Argument::Lifetime(RegionId::bound(0, 1)?),
            Argument::Type(TypeId::primitive(input)),
        ])?;
        let root = facts
            .types
            .intern(Kind::Projection(crate::types::projection::Application {
                arguments: own,
                ..application
            }))?;
        let mut fold = Normalizer::default();
        let Step::Resolve(request) = fold.start(&mut facts.types, root)? else {
            return Err("family query was not demanded".into());
        };
        let normal_form::Progress::Ready(selected) = normal_form::begin(
            facts,
            Some(std::sync::Arc::clone(&environment)),
            request,
            100,
            false,
            Availability::Declarations,
        )?
        else {
            return Err("unexpected unfinished header".into());
        };
        assert_eq!(selected.resolution, Some(expected));
        assert!(selected.source.is_some());
        let own = selected
            .proof
            .restoration()?
            .apply_arguments(&mut facts.types, selected.family)?;
        assert_eq!(own, request.alias.arguments());
        let Step::Complete(output) = fold.resume(&mut facts.types, request, expected)? else {
            return Err("family result did not complete normalization".into());
        };
        assert_eq!(output, if input == Primitive::U8 { result } else { root });
    }
    Ok(())
}

#[test]
fn qualified_and_shorthand_paths_keep_complete_distinct_arguments() -> TestResult {
    for source in [QUALIFIED, SHORTHAND] {
        let report = report(source)?;
        let mut checker = Checker::new(&report)?;
        reserve(&mut checker)?;
        let mut applications = Vec::new();
        for predicates in checker.facts.generics.predicates.values() {
            for predicate in predicates {
                let crate::types::parameters::clauses::Clause::Predicate(predicate) =
                    predicate.value
                else {
                    continue;
                };
                if predicate.bound == Bound::Copy
                    && let Some(Kind::Projection(application)) =
                        checker.facts.types.kind(predicate.ty)
                {
                    applications.push(application);
                }
            }
        }
        assert_eq!(applications.len(), 2);
        assert_ne!(applications[0].arguments, applications[1].arguments);
        for (application, (region, ty, count)) in applications
            .iter()
            .zip([(1, Primitive::U8, 3), (0, Primitive::U16, 5)])
        {
            let parent = checker
                .facts
                .types
                .trait_application(application.trait_application)
                .ok_or("missing parent")?;
            let parent: Vec<_> = checker
                .facts
                .types
                .arguments(parent.arguments)
                .ok_or("missing parent arguments")?
                .collect();
            assert_eq!(parent, [Argument::Lifetime(RegionId::bound(0, 0)?)]);
            let own: Vec<_> = checker
                .facts
                .types
                .arguments(application.arguments)
                .ok_or("missing member arguments")?
                .collect();
            assert_eq!(own[0], Argument::Lifetime(RegionId::bound(0, region)?));
            assert_eq!(own[1], Argument::Type(TypeId::primitive(ty)));
            let Argument::Const(id) = own[2] else {
                return Err("missing const argument".into());
            };
            assert_eq!(
                checker
                    .facts
                    .types
                    .constant(id)
                    .ok_or("missing constant")?
                    .bits,
                count
            );
        }
    }
    Ok(())
}

#[test]
fn family_path_arity_is_checked_before_publishing_a_projection() -> TestResult {
    for (source, expected) in [
        (
            "trait Q{type Out<T,const N:usize>;}trait R:Q where Self::Out<u8>:Copy{}",
            "missing required generic argument",
        ),
        (
            "trait Q{type Out<T>;}trait R:Q where <Self as Q>::Out<u8,u16>:Copy{}",
            "too many generic arguments",
        ),
    ] {
        let report = report(source)?;
        let mut checker = Checker::new(&report)?;
        let error = reserve(&mut checker).err().ok_or("accepted wrong arity")?;
        assert!(error.to_string().contains(expected), "{error}");
    }
    Ok(())
}

#[test]
fn trait_and_member_substitution_maps_self_and_both_owners_simultaneously() -> TestResult {
    let report = report("trait Q<'p,T>{type Out<'x,U,const N:usize>;}")?;
    let mut checker = Checker::new(&report)?;
    reserve(&mut checker)?;
    let member = checker.facts.associated.members()[0].definition;
    let owner = checker
        .facts
        .associated
        .trait_owner(member)
        .ok_or("missing trait")?;
    let schema = checker
        .facts
        .associated
        .schema(owner)
        .ok_or("missing schema")?;
    let subject = schema.self_type;
    let parent = &checker.facts.generics.parameters[&owner];
    let Argument::Type(parent_ty) = parent[1].argument else {
        return Err("missing parent type".into());
    };
    let own = &checker.facts.generics.parameters[&member];
    let Argument::Type(own_ty) = own[1].argument else {
        return Err("missing family type".into());
    };
    let Argument::Const(length) = own[2].argument else {
        return Err("missing family const".into());
    };
    let store = &mut checker.facts.types;
    let array = store.intern(Kind::ArrayTerm {
        element: parent_ty,
        length,
    })?;
    let reference = store.intern(Kind::Reference {
        region: RegionId::bound(0, 1)?,
        mutable: false,
        referent: array,
    })?;
    let list = store.intern_list(&[subject, parent_ty, own_ty, reference])?;
    let template = store.intern(Kind::Tuple(list))?;
    let parent =
        store.intern_arguments(&[Argument::Lifetime(RegionId(7)), Argument::Type(own_ty)])?;
    let length = store.intern_constant(Constant {
        ty: TypeId::primitive(Primitive::Usize),
        bits: 3,
    })?;
    let own = store.intern_arguments(&[
        Argument::Lifetime(RegionId(8)),
        Argument::Type(TypeId::primitive(Primitive::U8)),
        Argument::Const(length),
    ])?;
    let trait_application = store.intern_trait_application(crate::types::traits::Application {
        item: ItemId(owner.0),
        arguments: parent,
    })?;
    let predicate = Predicate {
        ty: TypeId::primitive(Primitive::U16),
        bound: Bound::Trait(trait_application),
    };
    let array = store.intern(Kind::Array {
        element: own_ty,
        length: 3,
    })?;
    let reference = store.intern(Kind::Reference {
        region: RegionId(8),
        mutable: false,
        referent: array,
    })?;
    let list = store.intern_list(&[
        predicate.ty,
        own_ty,
        TypeId::primitive(Primitive::U8),
        reference,
    ])?;
    let expected = store.intern(Kind::Tuple(list))?;
    let mut fold = checker
        .facts
        .trait_substitution(predicate, Some((member, own)))?;
    assert_eq!(fold.apply(&mut checker.facts.types, template)?, expected);
    assert!(
        checker
            .facts
            .trait_substitution(predicate, Some((owner, own)))
            .is_err()
    );
    Ok(())
}
