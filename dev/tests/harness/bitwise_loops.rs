//! Validate old-value dataflow and backend-updated scratch on each CAS backedge.
use rustgm::lowir::{
    program::{
        BlockId, Function, Instruction, Literal, Operand, Operation, ResultId, Storage, Terminator,
    },
    vocabulary::{Binary, Convert, Predicate, Type},
};
type Error = Box<dyn std::error::Error>;
pub fn check(body: &Function, maximum: Option<bool>) -> Result<(), Error> {
    assert_eq!(body.slots.len(), 1);
    assert!(matches!(body.slots[0], Storage::Scalar(Type::U32)));
    let (mut seeds, mut exchanges, mut scratch, mut tags, mut stores) = (0, 0, 0, 0, 0);
    let mut orders = rustgm::support::hash::HashSet::default();
    for instruction in &body.instructions {
        match instruction.operation {
            Operation::AtomicLoad {
                ty,
                address,
                ordering,
            } => {
                assert_eq!(ty, Type::U32);
                assert!(matches!(address, Operand::Parameter(0)));
                assert_eq!(ordering.encoding(), 0);
                seeds += 1;
            }
            Operation::AtomicCompareExchange {
                address, ordering, ..
            } => {
                assert!(matches!(address, Operand::Parameter(0)));
                assert_eq!(ordering.failure_encoding(), 0);
                orders.insert(ordering.success_encoding());
                exchanges += 1;
            }
            Operation::Load {
                ty: Type::U32,
                address,
                ..
            } => {
                assert!(matches!(address, Operand::Slot(id) if id.0 == 0));
                scratch += 1;
            }
            Operation::Load {
                ty: Type::U8,
                address,
                ..
            } => {
                assert!(matches!(address, Operand::Parameter(2)));
                tags += 1;
            }
            Operation::Store { value, address, .. } => {
                assert_eq!(value.ty, Type::U32);
                assert!(matches!(address, Operand::Slot(id) if id.0 == 0));
                stores += 1;
            }
            Operation::Load { .. }
            | Operation::AtomicStore { .. }
            | Operation::AtomicExchange { .. }
            | Operation::AtomicAddFetch { .. }
            | Operation::CopyObject { .. } => {
                return Err("unexpected bitwise shim memory access".into());
            }
            _ => {}
        }
    }
    assert_eq!((seeds, exchanges, scratch, tags, stores), (5, 5, 5, 1, 5));
    assert_eq!(
        orders,
        rustgm::support::hash::HashSet::from_iter([0, 2, 3, 4, 5])
    );
    let mut loops = 0;
    for (index, block) in body.blocks.iter().enumerate() {
        let instructions =
            &body.instructions[block.instructions.start as usize..block.instructions.end as usize];
        if instructions
            .last()
            .is_some_and(|i| matches!(i.operation, Operation::AtomicCompareExchange { .. }))
        {
            check_loop(body, BlockId(u32::try_from(index)?), instructions, maximum)?;
            loops += 1;
        }
    }
    assert_eq!(loops, 5);
    Ok(())
}
fn temporary(operand: Operand, result: Option<ResultId>) -> bool {
    matches!((operand, result), (Operand::Temporary(id), Some(ResultId::Scalar(actual))) if id == actual)
}
fn check_loop(
    body: &Function,
    id: BlockId,
    code: &[Instruction],
    maximum: Option<bool>,
) -> Result<(), Error> {
    assert!(
        matches!(code[0].operation, Operation::Load { ty: Type::U32, address: Operand::Slot(slot), .. } if slot.0 == 0)
    );
    if let Some(maximum) = maximum {
        check_extremum(body, code, maximum)?;
    } else {
        check_bitwise(body, code)?;
    }
    let exchange = code.last().ok_or("missing CAS")?;
    let Operation::AtomicCompareExchange {
        value, expected, ..
    } = exchange.operation
    else {
        return Err("missing CAS operation".into());
    };
    assert_eq!(value.ty, Type::U32);
    assert!(temporary(value.operand, code[code.len() - 2].result));
    let address = body
        .instructions
        .iter()
        .find(|i| temporary(expected, i.result))
        .ok_or("missing expected address")?;
    assert!(matches!(address.operation, Operation::Address(Operand::Slot(slot)) if slot.0 == 0));
    let Terminator::Branch { condition, yes, no } = body.blocks[id.0 as usize].terminator else {
        return Err("missing CAS backedge".into());
    };
    assert_eq!(no, id);
    assert!(temporary(condition, exchange.result));
    let Terminator::Return(Some(old)) = body.blocks[yes.0 as usize].terminator else {
        return Err("CAS success must return captured old value".into());
    };
    assert_eq!(old.ty, Type::U32);
    assert!(temporary(old.operand, code[0].result));
    check_seed(body, id)
}
fn check_seed(body: &Function, retry: BlockId) -> Result<(), Error> {
    let predecessors = body
        .blocks
        .iter()
        .filter(|b| matches!(b.terminator, Terminator::Jump(id) if id == retry))
        .collect::<Vec<_>>();
    assert_eq!(predecessors.len(), 1);
    let range = predecessors[0].instructions;
    let code = &body.instructions[range.start as usize..range.end as usize];
    assert_eq!(code.len(), 3);
    assert!(matches!(code[0].operation, Operation::AtomicLoad { .. }));
    let Operation::Store { value, address, .. } = code[1].operation else {
        return Err("missing initial expected value".into());
    };
    assert!(temporary(value.operand, code[0].result));
    assert!(matches!(address, Operand::Slot(slot) if slot.0 == 0));
    assert!(matches!(code[2].operation, Operation::Address(Operand::Slot(slot)) if slot.0 == 0));
    Ok(())
}

fn check_bitwise(body: &Function, code: &[Instruction]) -> Result<(), Error> {
    assert!(matches!(code.len(), 3 | 4));
    let Operation::Binary { op, left, right } = code[1].operation else {
        return Err("missing bitwise operand operation".into());
    };
    assert!(matches!(op, Binary::And | Binary::Or | Binary::Xor));
    assert_eq!(left.ty, Type::U32);
    assert!(temporary(left.operand, code[0].result));
    assert!(matches!(right, Operand::Parameter(1)));
    if code.len() == 4 {
        assert!(matches!(op, Binary::And));
        let Operation::Binary {
            op: Binary::Xor,
            left,
            right: Operand::Literal(mask),
        } = code[2].operation
        else {
            return Err("missing nand complement".into());
        };
        assert_eq!(left.ty, Type::U32);
        assert!(temporary(left.operand, code[1].result));
        assert!(
            matches!(body.literals[mask.0 as usize], Literal::Integer(value) if value == u128::from(u32::MAX))
        );
    }
    Ok(())
}
fn check_extremum(body: &Function, code: &[Instruction], maximum: bool) -> Result<(), Error> {
    assert_eq!(code.len(), 8);
    let Operation::Compare {
        predicate,
        left,
        right,
    } = code[1].operation
    else {
        return Err("missing unsigned extremum comparison".into());
    };
    assert!(matches!(
        (maximum, predicate),
        (true, Predicate::Ult) | (false, Predicate::Ugt)
    ));
    assert_eq!(left.ty, Type::U32);
    assert!(temporary(left.operand, code[0].result));
    assert!(matches!(right, Operand::Parameter(1)));
    let Operation::Convert {
        op: Convert::Trunc,
        value,
    } = code[2].operation
    else {
        return Err("missing compare-result truncation".into());
    };
    assert_eq!(value.ty, Type::I64);
    assert!(temporary(value.operand, code[1].result));
    for instruction in &code[2..7] {
        let Some(ResultId::Scalar(id)) = instruction.result else {
            return Err("missing u32 update value".into());
        };
        assert_eq!(body.temporaries[id.0 as usize], Type::U32);
    }
    let Operation::Binary {
        op: Binary::Sub,
        left,
        right,
    } = code[3].operation
    else {
        return Err("missing wrapping comparison mask".into());
    };
    assert_eq!(left.ty, Type::U32);
    let Operand::Literal(zero) = left.operand else {
        return Err("missing mask zero".into());
    };
    assert!(matches!(
        body.literals[zero.0 as usize],
        Literal::Integer(0)
    ));
    assert!(temporary(right, code[2].result));
    for (index, op, left, right) in [
        (4, Binary::Xor, 0, None),
        (5, Binary::And, 4, Some(3)),
        (6, Binary::Xor, 0, Some(5)),
    ] {
        let Operation::Binary {
            op: actual,
            left: value,
            right: operand,
        } = code[index].operation
        else {
            return Err("missing extremum selection dataflow".into());
        };
        assert!(matches!(
            (actual, op),
            (Binary::Xor, Binary::Xor) | (Binary::And, Binary::And)
        ));
        assert_eq!(value.ty, Type::U32);
        assert!(temporary(value.operand, code[left].result));
        if let Some(right) = right {
            assert!(temporary(operand, code[right].result));
        } else {
            assert!(matches!(operand, Operand::Parameter(1)));
        }
    }
    Ok(())
}
