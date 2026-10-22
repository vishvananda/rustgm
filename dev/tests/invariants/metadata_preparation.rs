//! A provisional Sized shortcut cannot authorize an immutable metadata query.
use super::*;
use crate::{
    abi::{layout::Layouts, metadata::Metadata},
    types::{
        GenericArgs, Primitive,
        aggregate::{Kind as AggregateKind, Repr, Variant},
    },
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn tuple(types: &mut Store, fields: &[TypeId]) -> Result<TypeId> {
    let fields = types.intern_list(fields)?;
    Ok(types.intern(Kind::Tuple(fields))?)
}
#[test]
fn physical_preparation_upgrades_only_completed_ordinary_entries() -> Result {
    let mut types = Store::default();
    let mut definitions = Definitions::default();
    let slice = types.intern(Kind::Slice(TypeId::primitive(Primitive::U8)))?;
    let item = ItemId(912);
    let unfinished = types.intern(Kind::Adt(item, GenericArgs::EMPTY))?;
    assert_eq!(types.is_sized(unfinished), Some(true));
    // Such a tuple is later rejected by ordinary WF, but can exist canonically
    // before that proof. Preparation itself must not turn its hint into authority.
    let outer = tuple(&mut types, &[slice, unfinished])?;
    assert!(definitions.prepare_metadata(&mut types, outer)?);
    assert!(matches!(
        definitions.metadata_ready.get(&outer),
        Some(Readiness::Ordinary)
    ));
    assert_eq!(
        definitions.prepare_physical_metadata(&mut types, outer)?,
        MetadataPreparation::Fields(item)
    );
    assert!(matches!(
        definitions.metadata_ready.get(&outer),
        Some(Readiness::Ordinary)
    ));
    assert!(!definitions.metadata_ready.contains_key(&unfinished));
    let fields = types.intern_list(&[slice])?;
    definitions.insert(
        item,
        AggregateKind::Struct,
        Repr::default(),
        &[Variant {
            fields,
            discriminant: 0,
        }],
    )?;
    assert_eq!(
        definitions.prepare_physical_metadata(&mut types, outer)?,
        MetadataPreparation::Ready
    );
    assert!(matches!(
        definitions.metadata_ready.get(&outer),
        Some(Readiness::Physical)
    ));
    let work = definitions.instantiation_steps();
    assert!(definitions.prepare_metadata(&mut types, outer)?);
    assert_eq!(definitions.instantiation_steps(), work + 1);
    assert_eq!(
        Layouts::default().prepared_pointer_metadata(&types, &definitions, outer)?,
        Metadata::Length
    );
    assert_eq!(std::mem::size_of::<TypeId>(), 4);
    assert_eq!(std::mem::size_of::<(TypeId, Readiness)>(), 8);
    assert_eq!(
        std::mem::size_of::<crate::support::hash::HashMap<TypeId, Readiness>>(),
        std::mem::size_of::<crate::support::hash::HashSet<TypeId>>()
    );
    Ok(())
}
#[test]
fn missing_fields_keep_ordinary_errors_and_never_publish_readiness() -> Result {
    let mut types = Store::default();
    let mut definitions = Definitions::default();
    let item = ItemId(913);
    let missing = types.intern(Kind::Adt(item, GenericArgs::EMPTY))?;
    let outer = tuple(&mut types, &[TypeId::primitive(Primitive::Str), missing])?;
    // Ordinary behavior still stops at the provisional Sized tail.
    assert!(definitions.prepare_metadata(&mut types, outer)?);
    assert_eq!(
        definitions.prepare_physical_metadata(&mut types, missing)?,
        MetadataPreparation::Fields(item)
    );
    assert!(!definitions.metadata_ready.contains_key(&missing));
    let invalid = TypeId(u32::MAX);
    assert!(definitions.prepare_metadata(&mut types, invalid).is_err());
    assert!(
        definitions
            .prepare_physical_metadata(&mut types, invalid)
            .is_err()
    );
    assert!(!definitions.metadata_ready.contains_key(&invalid));
    Ok(())
}

#[test]
fn unresolved_tails_are_not_physical_readiness_and_old_errors_survive() -> Result {
    use crate::types::parameters::{Parameter, ParameterKind};
    let mut types = Store::default();
    let mut definitions = Definitions::default();
    let parameter = types.intern_parameter(Parameter {
        owner: ItemId(914),
        index: 0,
        kind: ParameterKind::Type { sized: false },
    })?;
    let parameter = types.intern(Kind::Parameter(parameter))?;
    assert!(!definitions.prepare_metadata(&mut types, parameter)?);
    assert_eq!(
        definitions.prepare_physical_metadata(&mut types, parameter)?,
        MetadataPreparation::Unresolved(parameter)
    );
    assert!(definitions.metadata_ready.is_empty());
    let item = ItemId(915);
    let ty = types.intern(Kind::Adt(item, GenericArgs::EMPTY))?;
    let fields = types.intern_list(&[TypeId::primitive(Primitive::Str)])?;
    definitions.insert(
        item,
        AggregateKind::Struct,
        Repr::default(),
        &[Variant {
            fields,
            discriminant: 0,
        }],
    )?;
    let mut rules = crate::types::sizedness::Rules::default();
    assert_eq!(
        rules.prepare(&mut types, &definitions, item, |_, _, _| Ok(false))?,
        None
    );
    types.seal_nominal_sizes(std::sync::Arc::new(rules))?;
    assert_eq!(types.is_sized(ty), Some(false));
    let mut missing = Definitions::default();
    assert_eq!(
        missing.prepare_metadata(&mut types, ty),
        Err("missing aggregate field template")
    );
    assert_eq!(
        missing.prepare_physical_metadata(&mut types, ty)?,
        MetadataPreparation::Fields(item)
    );
    assert!(missing.metadata_ready.is_empty());
    Ok(())
}

#[test]
fn ordinary_known_borrows_add_no_trait_search_or_metadata_entries() -> Result {
    for width in [8, 32, 128] {
        let mut source = String::from("fn keep(x:&u8)->u8{*x}fn main(){");
        for _ in 0..width {
            source.push_str("let _=keep(&{2u8});");
        }
        source.push('}');
        let report = crate::test_source::resolve(&source, true)?;
        let analysis = crate::pipeline::analyze_source(&report)?;
        assert_eq!(analysis.facts.body_work.trait_selections, 0);
        assert_eq!(analysis.facts.aggregates.metadata_ready.capacity(), 0);
        eprintln!(
            "known borrow width={width} inference={} trait=0 metadata=0",
            analysis.facts.body_work.inference_steps
        );
    }
    Ok(())
}
