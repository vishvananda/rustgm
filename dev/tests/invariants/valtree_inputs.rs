//! Real evaluated nominal values retain their declarations while canonical operands share data.
use super::*;
use crate::{
    resolve::{expansion::Report, model::DefKind},
    sema::values::Evaluator,
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const SOURCE: &str = "struct Flags{a:bool,b:bool,c:bool,d:bool}struct Other{a:bool,b:bool,c:bool,d:bool}enum Choice{Empty,First(bool),Second(bool)}const A:Flags=Flags{a:true,b:false,c:true,d:false};const SAME:Flags=Flags{d:false,c:true,b:false,a:true};const B:Other=Other{a:true,b:false,c:true,d:false};const E:Choice=Choice::Empty;const F:Choice=Choice::First(true);const G:Choice=Choice::Second(true);const COPY:Flags=A;";
fn analyzed() -> Result<(Report, Declarations)> {
    let report = crate::test_source::resolve(SOURCE, true)?;
    let facts = crate::sema::declarations::analyze(&report)?;
    let facts = crate::sema::bodies::analyze(&report, facts)?;
    Ok((report, facts))
}
fn evaluated(report: &Report, facts: &mut Declarations, name: &str) -> Result<(TypeId, Constant)> {
    let index = report
        .resolution
        .definitions
        .iter()
        .position(|definition| {
            definition.kind == DefKind::Const && report.package.names.get(definition.name) == name
        })
        .ok_or("actual source constant")?;
    let definition = crate::resolve::model::DefId(u32::try_from(index)?);
    let ty = facts.definitions[index].ok_or("constant type")?;
    let value = Evaluator::new(report, facts).definition(definition)?;
    Ok((ty, value))
}
fn converted(facts: &mut Declarations, ty: TypeId, value: Constant) -> Result<ConstId> {
    Ok(canonicalize(facts, &mut 0, 1_000_000, ty, value)?)
}
#[test]
fn source_products_and_variants_become_exact_canonical_operands_without_rebuilding_arenas() -> Result
{
    let (report, mut facts) = analyzed()?;
    let mut ids = Vec::new();
    for name in ["A", "SAME", "B", "E", "F", "G"] {
        let (ty, value) = evaluated(&report, &mut facts, name)?;
        ids.push(converted(&mut facts, ty, value)?);
    }
    assert_eq!(ids[0], ids[1]);
    assert_ne!(ids[0], ids[2]);
    assert_ne!(ids[3], ids[4]);
    assert_ne!(ids[4], ids[5]);
    let constant = Constant::argument(&facts.types, ids[0])?.ok_or("canonical operand")?;
    assert!(matches!(constant.0, Value::Canonical(_)));
    let reader = Reader::new(&facts);
    let View::Product {
        length: 4,
        variant: Some(_),
    } = reader.view(constant)?
    else {
        return Err("actual source constructor lost".into());
    };
    for (index, expected) in [true, false, true, false].into_iter().enumerate() {
        assert_eq!(
            reader
                .field(constant, index as u64)?
                .as_scalar()
                .map_err(|e| e.message().to_owned())?,
            Scalar::Boolean(expected)
        );
    }
    let ty = facts.types.const_type(ids[0]).ok_or("typed operand")?;
    assert_eq!(reader.field_types(ty, constant)?.len(), 4);
    assert!(
        reader
            .field_types(facts.types.const_type(ids[2]).ok_or("other")?, constant)
            .is_err()
    );
    let mut steps = 0;
    assert_eq!(
        canonicalize(&mut facts, &mut steps, 4, ty, constant)?,
        ids[0]
    );
    assert!(steps <= 2);
    assert_eq!(size_of::<Constant>(), 32);
    Ok(())
}
#[test]
fn compressed_products_keep_large_bounds_and_actual_out_of_bounds_faults() -> Result {
    let report = crate::test_source::resolve("", true)?;
    let mut facts = crate::sema::declarations::analyze(&report)?;
    let ty = facts.types.intern(Kind::Array {
        element: TypeId::primitive(Primitive::Bool),
        length: 1_000_000_000,
    })?;
    let value = Constant(
        facts
            .constant_products
            .repeat(Value::Scalar(Scalar::Boolean(true)), 1_000_000_000)?,
    );
    let mut steps = 0;
    let id = canonicalize(&mut facts, &mut steps, 256, ty, value)?;
    assert!(steps < 128);
    let operand = Constant::argument(&facts.types, id)?.ok_or("operand")?;
    assert!(matches!(
        Reader::new(&facts).view(operand)?,
        View::Repeat {
            length: 1_000_000_000,
            ..
        }
    ));
    let evaluator = Evaluator::new(&report, &mut facts);
    assert_eq!(
        evaluator
            .field(operand, 999_999_999)?
            .as_scalar()
            .map_err(|e| e.message().to_owned())?,
        Scalar::Boolean(true)
    );
    assert!(matches!(
        evaluator.field(operand, 1_000_000_000),
        Err(crate::sema::values::Failure::Fault(
            "constant index is out of bounds"
        ))
    ));
    Ok(())
}
#[test]
fn malformed_nominal_views_scalar_kinds_and_failed_work_never_become_value_permission() -> Result {
    let (report, mut facts) = analyzed()?;
    let (ty, value) = evaluated(&report, &mut facts, "A")?;
    let (other, foreign) = evaluated(&report, &mut facts, "F")?;
    assert!(canonicalize(&mut facts, &mut 0, 1024, ty, foreign).is_err());
    let id = converted(&mut facts, other, foreign)?;
    let operand = Constant::argument(&facts.types, id)?.ok_or("canonical operand")?;
    assert!(canonicalize(&mut facts, &mut 0, 1024, ty, operand).is_err());
    assert!(
        canonicalize(
            &mut facts,
            &mut 0,
            1024,
            TypeId::primitive(Primitive::Bool),
            Constant::scalar(Scalar::Integer(1))
        )
        .is_err()
    );
    let before = facts.types.interned_extent();
    let mut failed = 0;
    assert!(canonicalize(&mut facts, &mut failed, 0, ty, value).is_err());
    assert!(failed > 0);
    assert_eq!(facts.types.interned_extent(), before);
    Ok(())
}

#[test]
fn generic_value_queries_suspend_and_publish_actual_canonical_ids() -> Result {
    use super::super::{Input, Progress, advance, argument};
    use crate::sema::constants::{Discriminants, Scratch};
    let report = crate::test_source::resolve(SOURCE, true)?;
    let mut facts = crate::sema::declarations::analyze(&report)?;
    let initializer = *facts
        .initializers
        .iter()
        .find(|initializer| {
            report
                .package
                .names
                .get(report.resolution.definitions[initializer.definition.0 as usize].name)
                == "COPY"
        })
        .ok_or("actual dependent initializer")?;
    let scope = report.resolution.definitions[initializer.definition.0 as usize].scope;
    let module = report.resolution.scopes[scope.0 as usize].module;
    let input = Input {
        key: initializer.value,
        ty: initializer.ty,
        module,
    };
    let mut scratch = Scratch::default();
    let mut discriminants = Discriminants::default();
    let mut ready = crate::support::hash::HashSet::default();
    let Progress::Prepare(dependency, continuation) = advance(
        &report,
        &mut facts,
        &mut scratch,
        &mut discriminants,
        input,
        None,
        &ready,
    )?
    else {
        return Err("dependent generic query did not suspend".into());
    };
    let crate::sema::constants::ArgumentDependency::Initializer(definition) = dependency else {
        return Err("generic query lost its actual initializer dependency".into());
    };
    assert_eq!(
        report
            .package
            .names
            .get(report.resolution.definitions[definition.0 as usize].name),
        "A"
    );
    facts = crate::sema::bodies::analyze(&report, facts)?;
    Evaluator::new(&report, &mut facts).definition(definition)?;
    ready.insert(definition);
    let Progress::Complete(id) = advance(
        &report,
        &mut facts,
        &mut scratch,
        &mut discriminants,
        input,
        Some(continuation),
        &ready,
    )?
    else {
        return Err("ready generic query did not complete".into());
    };
    assert!(matches!(
        facts.types.const_kind(id),
        Some(ConstKind::Aggregate(_))
    ));
    assert_eq!(
        argument(
            &report,
            &mut facts,
            &mut scratch,
            &mut discriminants,
            input.key,
            input.ty,
            module,
        )?,
        id
    );
    Ok(())
}

#[test]
fn actual_source_value_production_pays_growth_and_each_warm_or_failed_query() -> Result {
    let source = "#![feature(no_core,lang_items)]#![no_core]#[lang=\"pointee_sized\"]trait PointeeSized{}#[lang=\"meta_sized\"]trait MetaSized:PointeeSized{}#[lang=\"sized\"]trait Sized:MetaSized{}".to_owned() + SOURCE;
    let report = crate::test_source::source_provider(&source, "canonical_values")?;
    let facts = crate::sema::declarations::analyze(&report)?;
    let mut facts = crate::sema::bodies::analyze(&report, facts)?;
    let (ty, value) = evaluated(&report, &mut facts, "A")?;
    let before = header_attributes::source_contract_storage(&facts).ok_or("source payer")?;
    let id = converted(&mut facts, ty, value)?;
    let cold = header_attributes::source_contract_storage(&facts).ok_or("paid producer")?;
    assert!(cold.0 > before.0);
    assert!(cold.1 > before.1);
    let operand = Constant::argument(&facts.types, id)?.ok_or("source operand")?;
    for _ in 0..128 {
        assert_eq!(converted(&mut facts, ty, operand)?, id);
    }
    let warm = header_attributes::source_contract_storage(&facts).ok_or("paid warm values")?;
    assert!((128..=256).contains(&(warm.0 - cold.0)));
    assert_eq!(warm.1, cold.1);
    assert!(
        header_attributes::charge_source_contract_query(
            &facts,
            crate::support::crates::MAX_COMPONENTS,
        )
        .is_err()
    );
    let failed = header_attributes::source_contract_storage(&facts).ok_or("failed payer")?;
    assert!(converted(&mut facts, ty, operand).is_err());
    let retried = header_attributes::source_contract_storage(&facts).ok_or("retried payer")?;
    assert!(retried.0 > failed.0);
    assert_eq!(retried.1, failed.1);
    Ok(())
}

#[test]
fn conversion_work_follows_unique_evaluated_nodes_instead_of_expanded_tree_size() -> Result {
    let report = crate::test_source::resolve("", true)?;
    let mut facts = crate::sema::declarations::analyze(&report)?;
    let mut ty = TypeId::primitive(Primitive::Bool);
    let mut value = Constant::scalar(Scalar::Boolean(true));
    for _ in 0..128 {
        let fields = facts.types.intern_list(&[ty, ty])?;
        ty = facts.types.intern(Kind::Tuple(fields))?;
        value = Constant(facts.constant_products.fields(&[value.0, value.0], None)?);
    }
    let before = facts.types.interned_extent();
    let mut steps = 0;
    let id = canonicalize(&mut facts, &mut steps, 32_768, ty, value)?;
    assert!(steps < 32_768);
    assert!((128..=129).contains(&(facts.types.interned_extent() - before)));
    let mut current = Constant::argument(&facts.types, id)?.ok_or("canonical shared root")?;
    let reader = Reader::new(&facts);
    for _ in 0..128 {
        assert!(matches!(
            reader.view(current)?,
            View::Product {
                length: 2,
                variant: None
            }
        ));
        let first = reader.field(current, 0)?;
        assert!(first == reader.field(current, 1)?);
        current = first;
    }
    assert_eq!(current.as_scalar()?, Scalar::Boolean(true));
    let operand = Constant::argument(&facts.types, id)?.ok_or("canonical shared root")?;
    steps = 0;
    assert_eq!(canonicalize(&mut facts, &mut steps, 1, ty, operand)?, id);
    assert_eq!(steps, 1);
    Ok(())
}
