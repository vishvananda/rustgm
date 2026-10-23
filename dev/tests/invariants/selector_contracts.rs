//! Source defaults keep their actual sizing identities and exact selector subjects.
use super::*;
use crate::{
    support::bindings::SourceIntrinsic,
    types::{GenericArgs, ItemId, traits::Application},
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn selector() -> TestResult<(Declarations, Function)> {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pa31/tests/spec/const-eval-select-runtime/item-empty.rs"),
    )?;
    let report = crate::test_source::source_provider(&source, "selector_contract")?;
    let facts = crate::sema::declarations::analyze(&report)?;
    let catalog = facts.source_bindings.as_deref().ok_or("catalog")?;
    let function = facts
        .functions
        .iter()
        .copied()
        .find(|f| {
            catalog.intrinsic_kind(f.definition).ok().flatten()
                == Some(SourceIntrinsic::ConstEvalSelect)
        })
        .ok_or("actual selector")?;
    Ok((facts, function))
}

#[test]
fn actual_selector_defaults_keep_metasized_and_reject_an_additional_user_bound() -> TestResult {
    let (mut facts, function) = selector()?;
    validate(&facts, function)?;
    let types = parameters(&facts, function.definition)?;
    let catalog = facts.source_bindings.as_deref().ok_or("catalog")?;
    let sizing = catalog.sizing.ok_or("sizing")?;
    let defaults = facts
        .generics
        .declared_predicates(function.definition)
        .iter()
        .filter(|clause| {
            let Clause::Predicate(p) = clause.value else {
                return false;
            };
            p.bound.trait_parts().is_some_and(|(id, mode)| {
                mode == Constness::Ordinary
                    && facts
                        .types
                        .trait_application(id)
                        .is_some_and(|app| app.item.0 == sizing[1].0)
            })
        })
        .count();
    assert_eq!(
        defaults, 4,
        "each owned selector parameter retains MetaSized"
    );
    // Replace a redundant sizing clause inside the retained declared prefix.
    // The five required selector clauses stay complete; appending after
    // supertrait elaboration would exercise only the implied suffix.
    let index = facts
        .generics
        .declared_predicates(function.definition)
        .iter()
        .position(|clause| {
            let Clause::Predicate(p) = clause.value else {
                return false;
            };
            p.bound.trait_parts().is_some_and(|(id, _)| {
                facts
                    .types
                    .trait_application(id)
                    .is_some_and(|app| app.item.0 == sizing[1].0)
            })
        })
        .ok_or("declared sizing default")?;
    let predicates = facts
        .generics
        .predicates
        .get_mut(&function.definition)
        .ok_or("declared selector predicates")?;
    predicates[index].value = Clause::Predicate(Predicate {
        ty: types[0],
        bound: Bound::Clone,
    });
    assert_eq!(
        validate(&facts, function),
        Err("source selector has an unsupported defining constraint")
    );
    Ok(())
}

#[test]
fn source_sizing_clauses_require_exact_identity_arguments_effect_and_subject() -> TestResult {
    let (mut facts, function) = selector()?;
    let types = parameters(&facts, function.definition)?;
    let catalog = facts.source_bindings.as_deref().ok_or("catalog")?;
    let sizing = catalog.sizing.ok_or("sizing")?;
    let roles = [LangItem::Tuple, LangItem::FnOnce, LangItem::FnOnceOutput]
        .map(|role| catalog.language(role).map(|d| d.definition));
    let [Some(tuple), Some(call), Some(output)] = roles else {
        return Err("actual callable roles".into());
    };
    let identities = [tuple, call, output];
    let nonempty = facts.types.intern_arguments(&[Argument::Type(types[0])])?;
    let mut accepted = |item: DefId, arguments, mode, subject| -> TestResult<bool> {
        let app = facts.types.intern_trait_application(Application {
            item: ItemId(item.0),
            arguments,
        })?;
        let p = Predicate {
            ty: subject,
            bound: Bound::trait_application(app, mode),
        };
        Ok(requirement(&facts, p, types, identities, sizing).is_ok())
    };
    for role in sizing {
        assert!(accepted(
            role,
            GenericArgs::EMPTY,
            Constness::Ordinary,
            types[0]
        )?);
    }
    assert!(!accepted(
        DefId(u32::MAX),
        GenericArgs::EMPTY,
        Constness::Ordinary,
        types[0]
    )?);
    assert!(!accepted(
        sizing[1],
        nonempty,
        Constness::Ordinary,
        types[0]
    )?);
    for mode in [Constness::Const, Constness::Conditional] {
        assert!(!accepted(sizing[1], GenericArgs::EMPTY, mode, types[0])?);
    }
    assert!(!accepted(
        sizing[1],
        GenericArgs::EMPTY,
        Constness::Ordinary,
        TypeId::UNIT
    )?);
    Ok(())
}
