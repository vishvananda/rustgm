//! Value decoding rejects invalid targets without laundering symbolic storage.
use super::*;
use crate::const_eval::memory::Read;
fn take(read: Read<Constant>) -> Result<Constant, Failure> {
    match read {
        Read::Value(value) => Ok(value),
        Read::Pending => Err("unexpected pending scalar transmute".into()),
    }
}
#[test]
fn strict_transmute_values_preserve_bits_numeric_addresses_and_reject_invalid_or_symbolic_integers()
-> Result<(), String> {
    with_storage(
        1 << 20,
        |types| {
            Ok(types.intern(Kind::Pointer {
                mutable: false,
                pointee: TypeId::primitive(Primitive::U8),
            })?)
        },
        |machine, pointer| {
            let u8_ = TypeId::primitive(Primitive::U8);
            let usize_ = TypeId::primitive(Primitive::Usize);
            let boolean = TypeId::primitive(Primitive::Bool);
            for bits in [0, 1, 13, u64::MAX] {
                let address =
                    take(machine.transmute_value(usize_, pointer, scalar(u128::from(bits)))?)?;
                assert!(
                    matches!(Reader::new(machine.evaluator.facts()).view(address)?,crate::sema::values::View::Address {bits:actual} if actual==bits)
                );
                assert_eq!(
                    take(machine.transmute_value(pointer, usize_, address)?)?,
                    scalar(u128::from(bits))
                );
            }
            assert_eq!(
                take(machine.transmute_value(u8_, boolean, scalar(1))?)?,
                Constant::scalar(Scalar::Boolean(true))
            );
            assert!(machine.transmute_value(u8_, boolean, scalar(2)).is_err());
            assert!(
                machine
                    .transmute_value(
                        TypeId::primitive(Primitive::U32),
                        TypeId::primitive(Primitive::Char),
                        scalar(0xd800)
                    )
                    .is_err()
            );
            let mut allocations = Vec::new();
            let address = machine.allocate(&mut allocations, Some(scalar(7)), u8_)?;
            assert!(machine.transmute_value(pointer, usize_, address).is_err());
            assert_eq!(machine.slots, 0);
            assert!(machine.memory.bytes.roots.is_empty());
            Ok(())
        },
    )
}
#[test]
fn transmute_reference_duties_park_without_reencoding_or_expiring_original_roots()
-> Result<(), String> {
    for width in [1, 8, 32] {
        with_static_references(width, |machine, ty, definitions| {
            use crate::const_eval::ReadOperation;
            let mut values = Vec::new();
            for &definition in definitions {
                values.push(
                    machine
                        .evaluator
                        .static_address(definition, TypeId::primitive(Primitive::U32))?,
                );
            }
            let input = machine.evaluator.fields(&values, None)?;
            machine.read_operation = Some(ReadOperation {
                frame: 1,
                position: 0,
            });
            let roots = machine.memory.bytes.roots.len();
            let mut decoded = None;
            for (index, &definition) in definitions.iter().enumerate() {
                machine.waiting = None;
                assert!(matches!(
                    machine.transmute_value(ty, ty, input)?,
                    Read::Pending
                ));
                let now = machine.memory.bytes.decoded;
                if let Some(first) = decoded {
                    assert_eq!(now, first);
                } else {
                    decoded = Some(now);
                }
                assert_eq!(machine.memory.bytes.roots.len(), roots);
                assert!(
                    matches!(machine.waiting,Some(crate::const_eval::Requirement::InitializerBody(actual)) if actual==definition)
                );
                machine
                    .evaluator
                    .complete_definition(definition, scalar(index as u128 + 11));
            }
            machine.waiting = None;
            let result = take(machine.transmute_value(ty, ty, input)?)?;
            assert_eq!(Some(machine.memory.bytes.decoded), decoded);
            assert_eq!(machine.memory.bytes.paid_duties, 2 * width);
            for (index, original) in values.iter().enumerate() {
                let actual = machine.evaluator.field(result, index as u64)?;
                assert_eq!(actual.location(), original.location());
            }
            assert_eq!(machine.memory.bytes.roots.len(), roots);
            assert_eq!(machine.slots, 0);
            Ok(())
        })?;
    }
    Ok(())
}
