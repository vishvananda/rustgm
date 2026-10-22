//! Field-only Clone is a canonical declaration policy, never a source derive shortcut.
use super::*;
use crate::types::copy::{Declaration, Properties};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
#[test]
fn canonical_iterator_clone_checks_fields_without_element_bounds() -> Result {
    let report = crate::test_source::resolve(
        "struct V;fn check<'a,T:'a>(){fn need<X:Clone>(){}need::<core::slice::Iter<'a,T>>();}fn main(){}",
        true,
    )?;
    let facts = crate::pipeline::analyze_source(&report)?.facts;
    let declaration = report
        .resolution
        .library_types
        .get(LibraryType::SliceIter)
        .ok_or("iterator")?;
    let selected = facts
        .derives
        .get(&ItemId(declaration.definition.0))
        .ok_or("Clone declaration")?;
    assert!(!selected.has_argument_bounds());
    assert_eq!(
        selected.properties(),
        Properties {
            copy: false,
            clone_: true
        }
    );
    assert!(
        Declaration::derived(Properties {
            copy: false,
            clone_: true
        })
        .has_argument_bounds()
    );
    assert_eq!(size_of::<Properties>(), 2);
    assert_eq!(size_of::<Declaration>(), 3);
    assert_eq!(
        size_of::<(ItemId, Properties)>(),
        size_of::<(ItemId, Declaration)>()
    );
    Ok(())
}
#[test]
fn source_clone_keeps_argument_bounds_and_noncanonical_schema_rejects() -> Result {
    let report = crate::test_source::resolve(
        "#[derive(Clone)]struct W<'a,T>{r:&'a T}struct V;fn need<T:Clone>(){}fn main(){need::<W<'static,V>>();}",
        true,
    )?;
    assert!(crate::pipeline::analyze_source(&report).is_err());
    let report = crate::test_source::resolve("fn main(){let _:Option<u8>=None;}", true)?;
    let mut facts = crate::pipeline::analyze_source(&report)?.facts;
    let mut declaration = report
        .resolution
        .library_types
        .get(LibraryType::Option)
        .ok_or("Option")?
        .clone();
    declaration.kind = LibraryType::SliceIter;
    let before = facts.types.len();
    let mut builder = super::super::comparisons::LibraryContracts {
        report: &report,
        facts: &mut facts,
    };
    assert!(builder.build_library_nominal(&declaration).is_err());
    assert_eq!(facts.types.len(), before);
    Ok(())
}

#[test]
fn collection_contracts_stay_cold_for_custom_iteration_and_source_method_winners() -> Result {
    for source in [
        "fn main(){let a=1u8;let _=a+2;}",
        "struct I;impl core::iter::Iterator for I{type Item=u8;fn next(&mut self)->Option<u8>{None}}fn main(){for _ in I{}}",
        "trait X{fn iter(&self)->u8;}impl X for [u8;2]{fn iter(&self)->u8{9}}fn main(){let a=[1u8,2];let _=a.iter();}",
    ] {
        let report = crate::test_source::resolve(source, true)?;
        let facts = crate::pipeline::analyze_source(&report)?.facts;
        assert!(
            facts
                .iteration
                .as_deref()
                .is_none_or(|r| r.collections.is_none())
        );
        if let Some(nominal) = report.resolution.library_types.get(LibraryType::SliceIter) {
            assert!(facts.aggregates.get(ItemId(nominal.definition.0)).is_none());
            assert!(!facts.derives.contains_key(&ItemId(nominal.definition.0)));
        }
    }
    Ok(())
}

#[test]
fn collection_activation_prepares_exact_nominal_once_and_rejects_unregistered_owner() -> Result {
    for source in [
        "fn main(){let a=[1u8,2];for _ in &a{}}",
        "fn f<T>(s:&[T]){for _ in s{}}fn main(){f(&[1u8,2]);}",
        "fn main(){let a=[1u8,2];let _=<[u8]>::iter(&a);}",
    ] {
        let report = crate::test_source::resolve(source, true)?;
        let mut facts = crate::pipeline::analyze_source(&report)?.facts;
        let method = facts
            .iteration
            .as_deref()
            .and_then(|r| r.collections.as_deref())
            .ok_or("collection catalog")?
            .canonical
            .declarations[crate::support::iteration::Collection::SliceMethod as usize]
            .method;
        assert!(
            facts
                .iteration
                .as_deref()
                .and_then(|r| r.collections.as_deref())
                .is_some_and(|r| r.status == crate::sema::iteration::collections::Status::Complete)
        );
        let types = facts.types.len();
        super::super::iteration::collections::prepare(&report, &mut facts, method, true)?;
        assert_eq!(facts.types.len(), types);
        let invalid = DefId(u32::MAX);
        assert!(
            super::super::iteration::collections::prepare(&report, &mut facts, invalid, true)
                .is_err()
        );
        assert_eq!(facts.types.len(), types);
    }
    Ok(())
}

#[test]
fn collection_late_outlives_and_variance_use_existing_solvers_without_dependencies() -> Result {
    let report = crate::test_source::resolve(
        "struct Old<'a,T>(&'a T);struct Dep<'a,T>(Old<'a,T>);fn main(){let _:Option<core::slice::Iter<'static,u8>>=None;}",
        true,
    )?;
    let mut facts = crate::pipeline::analyze_source(&report)?.facts;
    let nominal = |name: &str| {
        report
            .resolution
            .definitions
            .iter()
            .enumerate()
            .find(|(_, d)| {
                d.kind == crate::resolve::model::DefKind::Struct
                    && report.package.names.get(d.name) == name
            })
            .map(|(i, _)| ItemId(u32::try_from(i).unwrap_or(u32::MAX)))
            .ok_or("nominal")
    };
    let old = nominal("Old")?;
    let dependent = nominal("Dep")?;
    let iterator = ItemId(
        report
            .resolution
            .library_types
            .get(LibraryType::SliceIter)
            .ok_or("Iter")?
            .definition
            .0,
    );
    let mut bounds = crate::types::outlives::Bounds::default();
    let mut variances = crate::types::variance::Variances::default();
    for item in [old, iterator] {
        let Some(Type::Adt(_, arguments)) =
            facts.definitions[item.0 as usize].and_then(|t| facts.types.kind(t))
        else {
            return Err("formal nominal".into());
        };
        bounds.append_leaf(&mut facts.types, &facts.aggregates, item)?;
        variances.append_independent(&facts.types, &facts.aggregates, item, arguments)?;
        assert_eq!(
            variances.get(item),
            Some([crate::types::variance::Variance::Covariant; 2].as_slice())
        );
        assert_eq!(bounds.parameters(item).ok_or("outlives")?.len(), 1);
    }
    let old_bounds = bounds.parameters(old).ok_or("old")?.to_vec();
    let old_variance = variances.get(old).ok_or("old variance")?.to_vec();
    assert!(
        bounds
            .append_leaf(&mut facts.types, &facts.aggregates, dependent)
            .is_err()
    );
    let Some(Type::Adt(_, arguments)) =
        facts.definitions[dependent.0 as usize].and_then(|t| facts.types.kind(t))
    else {
        return Err("dependent nominal".into());
    };
    assert!(
        variances
            .append_independent(&facts.types, &facts.aggregates, dependent, arguments)
            .is_err()
    );
    assert_eq!(bounds.parameters(old), Some(old_bounds.as_slice()));
    assert_eq!(variances.get(old), Some(old_variance.as_slice()));
    assert!(
        bounds
            .append_leaf(&mut facts.types, &facts.aggregates, iterator)
            .is_err()
    );
    Ok(())
}

#[test]
fn incomplete_collection_catalog_never_authorizes_factory_publication() -> Result {
    let report = crate::test_source::resolve("fn main(){let a=[1u8];for _ in &a{}}", true)?;
    let mut facts = crate::pipeline::analyze_source(&report)?.facts;
    let registry = facts
        .iteration
        .as_deref_mut()
        .and_then(|r| r.collections.as_deref_mut())
        .ok_or("collections")?;
    let originals = registry.originals;
    registry.status = crate::sema::iteration::collections::Status::Prepared;
    for original in originals.into_iter().flatten() {
        assert!(original.validate(&facts).is_err());
    }
    let owner = originals[0].ok_or("original")?.declaration.definition;
    super::super::iteration::collections::prepare(&report, &mut facts, owner, true)?;
    for original in originals.into_iter().flatten() {
        original.validate(&facts)?;
        let mut wrong = original;
        wrong.declaration.method = original.declaration.parameter;
        assert!(wrong.validate(&facts).is_err());
        wrong = original;
        wrong.item = crate::types::TypeId::UNIT;
        assert!(wrong.validate(&facts).is_err());
    }
    Ok(())
}

#[test]
fn shared_collection_activation_is_once_and_local_under_unrelated_nominal_growth() -> Result {
    use std::fmt::Write;
    let mut observations = Vec::new();
    for (loops, unused) in [(8, 0), (32, 0), (128, 0), (1, 8), (1, 32), (1, 128)] {
        let mut source = String::new();
        for i in 0..unused {
            write!(source, "struct U{i}(u8);")?;
        }
        source.push_str("fn main(){let a=[1u8,2];");
        for _ in 0..loops {
            source.push_str("for _ in &a{}");
        }
        source.push('}');
        let report = crate::test_source::resolve(&source, true)?;
        let facts = crate::sema::declarations::analyze(&report)?;
        assert!(
            facts
                .iteration
                .as_deref()
                .is_some_and(|r| r.collections.is_none())
        );
        let before = (
            facts.lifetimes.nominal_requirements.steps,
            facts.lifetimes.variances.steps,
        );
        let facts = crate::sema::bodies::analyze(&report, facts)?;
        let registry = facts.iteration.as_deref().ok_or("iteration")?;
        assert_eq!(registry.plans.len(), loops);
        assert_eq!(
            registry
                .collections
                .as_deref()
                .ok_or("collections")?
                .originals
                .iter()
                .flatten()
                .count(),
            4
        );
        assert_eq!(facts.body_work.trait_selections, 2 * loops);
        let appended = (
            facts.lifetimes.nominal_requirements.steps - before.0,
            facts.lifetimes.variances.steps - before.1,
        );
        assert!(appended.0 > 0 && appended.1 > 0);
        eprintln!(
            "shared iteration loops={loops} unrelated_nominals={unused} appended={appended:?} inference={} selections={}",
            facts.body_work.inference_steps, facts.body_work.trait_selections
        );
        observations.push(appended);
    }
    assert!(observations.iter().all(|entry| *entry == observations[0]));
    Ok(())
}

#[test]
fn collection_layout_preserves_scalar_owners_and_padded_derive_entries() {
    fn variant_size<T>(_: fn(&LibraryNominal) -> &Vec<T>) -> usize {
        size_of::<T>()
    }
    eprintln!(
        "collection sizes library_types={} nominal={} variant={} field={} resolution={} declarations={} type_kind={} store={} context={} declaration={} properties={} derive_pair={}",
        size_of::<crate::resolve::model::LibraryTypes>(),
        size_of::<LibraryNominal>(),
        variant_size(|n| &n.variants),
        size_of::<LibraryField>(),
        size_of::<crate::resolve::model::Resolution>(),
        size_of::<crate::sema::model::Declarations>(),
        size_of::<Type>(),
        size_of::<crate::types::Store>(),
        size_of::<crate::types::infer::Context>(),
        size_of::<Declaration>(),
        size_of::<Properties>(),
        size_of::<(ItemId, Declaration)>()
    );
    // The integrated pointer-add identities add two 32-byte slots and one
    // eight-byte lazy method catalog to this worker's 8472-byte base.
    assert_eq!(size_of::<crate::sema::model::Declarations>(), 8544);
    assert_eq!(size_of::<Type>(), 24);
    assert_eq!(
        size_of::<(ItemId, Declaration)>(),
        size_of::<(ItemId, Properties)>()
    );
}
