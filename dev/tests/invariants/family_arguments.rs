//! Parent and family mappings remain simultaneous, scoped and keyed independently.
use super::{Application, DefId, Parameter, joint};
use crate::types::{
    GenericArgs, ItemId, Kind, Primitive, RegionId, Store, TypeId,
    arguments::{Argument, Constant},
    parameters::{Parameter as Formal, ParameterKind},
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const BYTE: TypeId = TypeId::primitive(Primitive::U8);
const WORD: TypeId = TypeId::primitive(Primitive::U16);
const SIZE: TypeId = TypeId::primitive(Primitive::Usize);

fn parameter(store: &mut Store, owner: u32, index: u32, constant: bool) -> TestResult<Parameter> {
    let id = store.intern_parameter(Formal {
        owner: ItemId(owner),
        index,
        kind: if constant {
            ParameterKind::Const { ty: SIZE }
        } else {
            ParameterKind::Type { sized: true }
        },
    })?;
    Ok(Parameter {
        anonymous: false,
        binding: DefId(index),
        argument: if constant {
            Argument::Const(store.intern_const(crate::types::arguments::ConstKind::Parameter(id))?)
        } else {
            Argument::Type(store.intern(Kind::Parameter(id))?)
        },
        default: None,
    })
}
fn reference(store: &mut Store, region: RegionId, referent: TypeId) -> TestResult<TypeId> {
    Ok(store.intern(Kind::Reference {
        region,
        mutable: false,
        referent,
    })?)
}
fn formal_type(parameter: Parameter) -> TestResult<TypeId> {
    let Argument::Type(ty) = parameter.argument else {
        return Err("expected type formal".into());
    };
    Ok(ty)
}

#[test]
fn mixed_arguments_preserve_anonymous_captures_and_nested_local_binders() -> TestResult {
    let mut store = Store::default();
    let parent = parameter(&mut store, 1, 1, false)?;
    let element = parameter(&mut store, 2, 1, false)?;
    let constant = parameter(&mut store, 2, 2, true)?;
    let lifetime = Parameter {
        anonymous: false,
        binding: DefId(9),
        argument: Argument::Lifetime(RegionId::bound(0, 2)?),
        default: None,
    };
    let Argument::Const(length) = constant.argument else {
        return Err("missing const formal".into());
    };
    let array = store.intern(Kind::ArrayTerm {
        element: formal_type(element)?,
        length,
    })?;
    let local = reference(&mut store, RegionId::bound(0, 0)?, formal_type(element)?)?;
    let captured = reference(&mut store, RegionId::bound(1, 2)?, formal_type(parent)?)?;
    let anonymous = reference(&mut store, RegionId::bound(1, 1)?, formal_type(element)?)?;
    let parameters = store.intern_list(&[local, captured, anonymous])?;
    let template = store.intern(Kind::Function {
        parameters,
        result: array,
        bound_regions: 1,
        unsafe_: false,
    })?;
    let length = store.intern_constant(Constant { ty: SIZE, bits: 3 })?;
    let parent_arguments = store.intern_arguments(&[
        Argument::Lifetime(RegionId(7)),
        Argument::Lifetime(RegionId(8)),
        Argument::Type(WORD),
    ])?;
    let own = store.intern_arguments(&[
        Argument::Lifetime(RegionId::bound(0, 4)?),
        Argument::Type(BYTE),
        Argument::Const(length),
    ])?;
    let application = Application {
        owner: DefId(1),
        parent: parent_arguments,
        member: DefId(2),
        own,
    };
    let mut fold = joint(
        &store,
        application,
        (&[parent], &[lifetime, element, constant]),
        2,
    )?;
    let result = fold.apply(&mut store, template)?;
    let local = reference(&mut store, RegionId::bound(0, 0)?, BYTE)?;
    let captured = reference(&mut store, RegionId::bound(1, 4)?, WORD)?;
    let anonymous = reference(&mut store, RegionId(8), BYTE)?;
    let parameters = store.intern_list(&[local, captured, anonymous])?;
    let array = store.intern(Kind::Array {
        element: BYTE,
        length: 3,
    })?;
    let expected = store.intern(Kind::Function {
        parameters,
        result: array,
        bound_regions: 1,
        unsafe_: false,
    })?;
    assert_eq!(result, expected);
    let before = fold.steps;
    assert_eq!(fold.apply(&mut store, template)?, expected);
    assert!(fold.steps - before <= 1);
    Ok(())
}

#[test]
fn replacement_arguments_are_not_substituted_again_by_the_other_owner() -> TestResult {
    let mut store = Store::default();
    let parent = parameter(&mut store, 1, 0, false)?;
    let member = parameter(&mut store, 2, 0, false)?;
    let parent_arguments = store.intern_arguments(&[member.argument])?;
    let own = store.intern_arguments(&[Argument::Type(BYTE)])?;
    let application = Application {
        owner: DefId(1),
        parent: parent_arguments,
        member: DefId(2),
        own,
    };
    let mut fold = joint(&store, application, (&[parent], &[member]), 0)?;
    let list = store.intern_list(&[formal_type(parent)?, formal_type(member)?])?;
    let template = store.intern(Kind::Tuple(list))?;
    let list = store.intern_list(&[formal_type(member)?, BYTE])?;
    let expected = store.intern(Kind::Tuple(list))?;
    assert_eq!(fold.apply(&mut store, template)?, expected);
    Ok(())
}

#[test]
fn malformed_family_arity_kinds_and_captured_slots_cannot_build_a_mapping() -> TestResult {
    let mut store = Store::default();
    let ty = parameter(&mut store, 2, 1, false)?;
    let constant = parameter(&mut store, 2, 2, true)?;
    let lifetime = Parameter {
        anonymous: false,
        binding: DefId(9),
        argument: Argument::Lifetime(RegionId::bound(0, 1)?),
        default: None,
    };
    let value = store.intern_constant(Constant { ty: SIZE, bits: 3 })?;
    let wrong = store.intern_constant(Constant { ty: BYTE, bits: 3 })?;
    let parent = store.intern_arguments(&[Argument::Lifetime(RegionId(7))])?;
    let valid = [
        Argument::Lifetime(RegionId(8)),
        Argument::Type(WORD),
        Argument::Const(value),
    ];
    for arguments in [
        valid[..2].to_vec(),
        vec![valid[0], valid[2], valid[1]],
        vec![valid[0], valid[1], Argument::Const(wrong)],
    ] {
        let own = store.intern_arguments(&arguments)?;
        assert!(
            joint(
                &store,
                Application {
                    owner: DefId(1),
                    parent,
                    member: DefId(2),
                    own
                },
                (&[], &[lifetime, ty, constant]),
                1
            )
            .is_err()
        );
    }
    let own = store.intern_arguments(&valid)?;
    let application = Application {
        owner: DefId(1),
        parent,
        member: DefId(2),
        own,
    };
    assert!(joint(&store, application, (&[], &[lifetime, ty, constant]), 1).is_ok());
    assert!(
        joint(
            &store,
            Application {
                parent: GenericArgs::EMPTY,
                ..application
            },
            (&[], &[lifetime, ty, constant]),
            1
        )
        .is_err()
    );
    let incorrect = Parameter {
        argument: Argument::Lifetime(RegionId::bound(0, 0)?),
        ..lifetime
    };
    assert!(joint(&store, application, (&[], &[incorrect, ty, constant]), 1).is_err());
    assert!(
        joint(
            &store,
            Application {
                member: DefId(3),
                ..application
            },
            (&[], &[lifetime, ty, constant]),
            1
        )
        .is_err()
    );
    Ok(())
}

#[expect(
    clippy::default_trait_access,
    reason = "the fixture uses resolution defaults without depending on expansion internals"
)]
fn declarations(source: &str) -> TestResult<crate::sema::model::Declarations> {
    use crate::resolve::load::{InputFile, Sources};
    struct NoFiles;
    impl Sources for NoFiles {
        fn read(&mut self, _: &std::path::Path) -> Result<Option<InputFile>, String> {
            Ok(None)
        }
    }
    let input = InputFile {
        identity: "families.rs".into(),
        path: "families.rs".into(),
        source: crate::support::source::SourceFile::new(source.as_bytes().to_vec())?,
    };
    let expanded = crate::resolve::expansion::analyze(input, &mut NoFiles, &Default::default())?;
    let report = crate::resolve::analyze_expanded(expanded)?;
    crate::sema::declarations::analyze(&report).map_err(Into::into)
}

#[test]
fn selected_definition_retains_both_argument_groups_and_member_identity() -> TestResult {
    use crate::sema::{projections::Definition, traits::selection::Source};
    let mut facts = declarations(
        "trait Q<T>{type Out;type Other;}struct S;impl<T>Q<T> for S{type Out=T;type Other=T;}",
    )?;
    let implementation = facts
        .generics
        .implementations()
        .next()
        .ok_or("missing impl")?;
    let owner = implementation.owner;
    let members = implementation.types.clone();
    let mut selections = Vec::new();
    let mut templates = Vec::new();
    for (declared, member) in members {
        let parent = facts.definitions[member.0 as usize].ok_or("missing template")?;
        // Install the future family's typed inputs at this internal boundary.
        // This exercises retention and substitution, not source formation proof.
        let own = parameter(&mut facts.types, member.0, 0, false)?;
        facts.generics.parameters.insert(member, vec![own]);
        let list = facts.types.intern_list(&[parent, formal_type(own)?])?;
        let template = facts.types.intern(Kind::Tuple(list))?;
        facts.definitions[member.0 as usize] = None;
        templates.push((member, template));
        for parent in [BYTE, WORD] {
            for own in [BYTE, WORD] {
                let arguments = facts.types.intern_arguments(&[Argument::Type(parent)])?;
                let family = facts.types.intern_arguments(&[Argument::Type(own)])?;
                let list = facts.types.intern_list(&[parent, own])?;
                let expected = facts.types.intern(Kind::Tuple(list))?;
                let selected = Definition::select(
                    &facts,
                    ItemId(declared.0),
                    family,
                    Source::Implementation { owner, arguments },
                )?;
                assert_eq!(selected.result(&mut facts)?, None);
                selections.push((selected, expected));
            }
        }
    }
    // A completed selection must not replay the removed implementation search.
    facts.generics.implementations = None;
    for (member, template) in templates {
        facts.definitions[member.0 as usize] = Some(template);
        let parent = facts.types.intern_arguments(&[Argument::Type(BYTE)])?;
        assert!(
            facts
                .instantiate_associated_definition(
                    owner,
                    parent,
                    member,
                    GenericArgs::EMPTY,
                    template,
                )
                .is_err()
        );
    }
    for (selected, expected) in &selections {
        assert_eq!(selected.result(&mut facts)?, Some(*expected));
    }
    let cache = facts
        .generics
        .family_substitutions
        .as_ref()
        .ok_or("missing cache")?;
    assert_eq!(cache.completed.len(), selections.len());
    let before: usize = cache.completed.values().map(|fold| fold.steps).sum();
    for (selected, expected) in &selections {
        assert_eq!(selected.result(&mut facts)?, Some(*expected));
    }
    let after: usize = facts
        .generics
        .family_substitutions
        .as_ref()
        .ok_or("missing cache")?
        .completed
        .values()
        .map(|fold| fold.steps)
        .sum();
    assert!(after - before <= selections.len());
    Ok(())
}
