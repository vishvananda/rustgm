//! Canonical immutable operands become independent mutable execution storage.
use super::{Constant, Failure, Machine, RawAccess, RawAlignment, Resolved};
use crate::{
    body,
    resolve::model::DefKind,
    sema,
    sema::{model::Scalar, values::Reader},
    test_source,
    types::{Primitive, TypeId, arguments},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn canonical_products_share_reads_but_preserve_mutation_authority_and_liveness() -> Result {
    let report = test_source::resolve(
        "struct Flags{a:bool,b:bool}const A:Flags=Flags{a:true,b:false};const fn f(){}",
        true,
    )?;
    let facts = sema::declarations::analyze(&report)?;
    let mut facts = sema::bodies::analyze(&report, facts)?;
    let bodies = body::build(&report, &facts)?;
    let index = report
        .resolution
        .definitions
        .iter()
        .position(|definition| definition.kind == DefKind::Const)
        .ok_or("actual constant declaration")?;
    let ty = facts.definitions[index].ok_or("actual nominal type")?;
    let boolean = TypeId::primitive(Primitive::Bool);
    let mut children = Vec::new();
    for bits in [1, 0] {
        children.push(facts.types.intern_const(arguments::ConstKind::Value(
            arguments::Constant { ty: boolean, bits },
        ))?);
    }
    let id = facts
        .types
        .intern_aggregate_constant(ty, None, &children, |_, _| Ok(()))?;
    let operand = Constant::argument(&facts.types, id)?.ok_or("canonical operand")?;
    assert_eq!(Reader::new(&facts).field_types(ty, operand)?.len(), 2);
    let evaluator = sema::values::Evaluator::new(&report, &mut facts);
    let mut machine = Machine::new(
        evaluator,
        &bodies.bodies,
        crate::const_eval::Limits::default(),
    )?;
    let mut allocations = Vec::new();
    let first = machine.allocate(&mut allocations, Some(operand), ty)?;
    let second = machine.allocate(&mut allocations, Some(operand), ty)?;
    let root = |address| Resolved {
        address,
        ty,
        raw: None,
        union: None,
    };
    let retained = machine.memory.retained_cells();
    for _ in 0..16 {
        assert!(machine.read(root(first))? == Some(operand));
        assert!(machine.read(root(second))? == Some(operand));
    }
    assert_eq!(machine.memory.retained_cells(), retained);
    let address = machine.evaluator.projection(first, 1, boolean)?;
    let field = Resolved {
        address,
        ty: boolean,
        raw: None,
        union: None,
    };
    machine.write_place(field, Constant::scalar(Scalar::Boolean(true)))?;
    let changed = machine.read(root(first))?.ok_or("changed snapshot")?;
    let reader = Reader::new(machine.evaluator.facts());
    assert_eq!(
        reader.field(changed, 1)?.as_scalar()?,
        Scalar::Boolean(true)
    );
    assert_eq!(
        reader.field(operand, 1)?.as_scalar()?,
        Scalar::Boolean(false)
    );
    assert!(machine.read(root(second))? == Some(operand));
    let shared = Resolved {
        address: address.restrict_shared(),
        raw: Some(RawAccess {
            alignment: RawAlignment::Required,
            location: address.location(),
            ty: boolean,
        }),
        ..field
    };
    assert_eq!(
        machine.check_raw_write(shared).err().map(Failure::message),
        Some("constant raw pointer write is derived from a shared reference")
    );
    assert_eq!(
        machine.read(field)?.ok_or("unchanged field")?.as_scalar()?,
        Scalar::Boolean(true)
    );
    machine.memory.expire(&allocations);
    assert_eq!(
        machine.read(root(first)).err().map(Failure::message),
        Some("constant pointer refers to expired storage")
    );
    assert_eq!(
        machine.read(field).err().map(Failure::message),
        Some("constant pointer refers to expired storage")
    );
    Ok(())
}
