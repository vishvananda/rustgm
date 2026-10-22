//! Offset values retain per-pointer restrictions without demanding stored contents.
use super::*;
use crate::{
    sema::{
        self,
        model::Scalar,
        values::{Evaluator, Reader, WritePermission},
    },
    test_source,
    types::{ItemId, arguments::Argument},
};
fn with_machine(
    check: impl FnOnce(&mut Machine<'_, '_>, Instance, TypeId) -> Result<(), Failure>,
) -> Result<(), String> {
    let report = test_source::resolve("unsafe fn f(p:*mut u8)->*mut u8{unsafe{p.add(1)}}", true)?;
    let facts = sema::declarations::analyze(&report).map_err(|e| e.to_string())?;
    let mut facts = sema::bodies::analyze(&report, facts).map_err(|e| e.to_string())?;
    let definition = facts
        .intrinsics
        .get(Intrinsic::MutPointerAdd)
        .ok_or("method")?
        .definition;
    let byte = TypeId::primitive(Primitive::U8);
    let array = facts.types.intern(Kind::Array {
        element: byte,
        length: 4,
    })?;
    let pointer = facts.types.intern(Kind::Pointer {
        mutable: true,
        pointee: byte,
    })?;
    let parameters = facts
        .types
        .intern_list(&[pointer, TypeId::primitive(Primitive::Usize)])?;
    let signature = facts.types.intern(Kind::Function {
        parameters,
        result: pointer,
        unsafe_: true,
        convention: Convention::RUST,
        bound_regions: 0,
    })?;
    let arguments = facts.types.intern_arguments(&[Argument::Type(byte)])?;
    let callable = facts.types.intern(Kind::FunctionItem {
        item: ItemId(definition.0),
        signature,
        arguments,
    })?;
    let instance = Instance::new(&facts.types, callable)?;
    let evaluator = Evaluator::new(&report, &mut facts);
    let mut machine = Machine::new(
        evaluator,
        std::iter::empty(),
        crate::const_eval::Limits::default(),
    )?;
    check(&mut machine, instance, array).map_err(|e| e.message().to_owned())
}
fn count(n: u128) -> Constant {
    Constant::scalar(Scalar::Integer(n))
}
#[test]
fn offset_preserves_authority_and_does_not_read_uninitialized_storage() -> Result<(), String> {
    with_machine(|machine, instance, array| {
        let address = machine.allocate(&mut Vec::new(), None, array)?;
        for incoming in [address, address.restrict_shared()] {
            assert!(
                machine.pointer_offset(
                    Intrinsic::MutPointerAdd,
                    instance,
                    &[incoming, count(0)]
                )? == incoming
            );
            let next = machine.pointer_offset(
                Intrinsic::MutPointerAdd,
                instance,
                &[incoming, count(1)],
            )?;
            assert_eq!(next.write_permission(), incoming.write_permission());
            let (view, _) = machine.evaluator.word_address(next, 1024)?;
            assert_eq!(view.root(), address.location().ok_or("root")?);
            assert_eq!(view.offset(), 1);
            let place = crate::const_eval::memory::Resolved {
                address: next,
                ty: TypeId::primitive(Primitive::U8),
                raw: Some(crate::const_eval::memory::RawAccess {
                    alignment: crate::const_eval::memory::RawAlignment::Required,
                    location: next.location(),
                    ty: TypeId::primitive(Primitive::U8),
                }),
                union: None,
            };
            assert!(machine.read(place).is_err());
            if incoming.write_permission() == Some(WritePermission::SharedDerived) {
                assert!(machine.check_raw_write(place).is_err());
            }
        }
        assert!(
            machine
                .pointer_offset(Intrinsic::ConstPointerAdd, instance, &[address, count(0)])
                .is_err()
        );
        assert!(
            machine
                .pointer_offset(Intrinsic::MutPointerAdd, instance, &[address, count(5)])
                .is_err()
        );
        let root = address.location().ok_or("root")?;
        machine.memory.expire(&[root]);
        assert!(
            machine.pointer_offset(Intrinsic::MutPointerAdd, instance, &[address, count(0)])?
                == address
        );
        assert!(
            machine
                .pointer_offset(Intrinsic::MutPointerAdd, instance, &[address, count(1)])
                .is_err()
        );
        assert_eq!(
            Reader::new(machine.evaluator.facts()).location(root)?.ty,
            array
        );
        Ok(())
    })
}
