//! A source atomic update carries its typed operands and evaluated orderings.
use super::checked_atomic_access::{Word, ordering, word_kind};
use crate::{
    resolve::model::DefId,
    sema::model::Declarations,
    support::{atomic_ordering::Ordering, word_operation::Update},
    types::{Kind, Primitive, TypeId, arguments::Argument, callable::Convention},
};

#[derive(Clone, Copy)]
pub(crate) struct Selection {
    pub pointer: TypeId,
    pub operand: TypeId,
    pub result: TypeId,
    pub success: Ordering,
    pub failure: Ordering,
}

pub(crate) fn select(
    facts: &Declarations,
    operation: Update,
    callee: TypeId,
    signature: TypeId,
) -> Result<Option<Selection>, &'static str> {
    let types = &facts.types;
    let Some(Kind::FunctionItem {
        item, arguments, ..
    }) = types.kind(callee)
    else {
        return Err("source atomic update requires its complete function item");
    };
    let mut arguments = types
        .arguments(arguments)
        .ok_or("source atomic update lost its generic arguments")?;
    let Some(Argument::Type(word)) = arguments.next() else {
        return Err("source atomic update requires its word type argument");
    };
    let (operand, order_index) = if operation.takes_operand_type() {
        let Some(Argument::Type(operand)) = arguments.next() else {
            return Err("source atomic update requires its operand type argument");
        };
        (operand, 2)
    } else {
        (word, 1)
    };
    let Some(Argument::Const(success)) = arguments.next() else {
        return Err("source atomic update requires its success ordering");
    };
    let definition = DefId(item.0);
    let Some(success) = ordering(facts, definition, order_index, success)? else {
        return Ok(None);
    };
    let failure = if operation.compares() {
        let Some(Argument::Const(failure)) = arguments.next() else {
            return Err("source atomic comparison requires its failure ordering");
        };
        let failure = ordering(facts, definition, order_index + 1, failure)?
            .ok_or("source atomic comparison requires its evaluated failure ordering")?;
        if !failure.load_valid() {
            return Err("invalid source atomic comparison failure ordering");
        }
        failure
    } else {
        Ordering::Relaxed
    };
    if arguments.next().is_some() {
        return Err("source atomic update has unexpected generic arguments");
    }
    let kind = word_kind(types, word)?;
    let expected = if operation.takes_operand_type() && matches!(kind, Word::Pointer) {
        TypeId::primitive(Primitive::Usize)
    } else {
        word
    };
    if operand != expected {
        return Err("source atomic update has an incompatible operand type");
    }
    if let Update::Min { signed } | Update::Max { signed } = operation
        && !matches!(kind, Word::Integer { signed: actual } if signed == actual)
    {
        return Err("source atomic extremum requires the indicated integer signedness");
    }
    let (pointer, result) = opened(types, operation, signature, word, operand)?;
    Ok(Some(Selection {
        pointer,
        operand,
        result,
        success,
        failure,
    }))
}

fn opened(
    types: &crate::types::Store,
    operation: Update,
    signature: TypeId,
    word: TypeId,
    operand: TypeId,
) -> Result<(TypeId, TypeId), &'static str> {
    let Some(Kind::Function {
        bound_regions: 0,
        parameters,
        result,
        unsafe_: true,
        convention: Convention::RUST,
    }) = types.kind(signature)
    else {
        return Err("source atomic update requires its opened unsafe Rust signature");
    };
    let Some((&pointer, inputs)) = types.list(parameters).and_then(<[_]>::split_first) else {
        return Err("source atomic update has no pointer input");
    };
    if !matches!(types.kind(pointer), Some(Kind::Pointer { mutable: true, pointee }) if pointee == word)
    {
        return Err("source atomic update requires a mutable raw pointer to its word");
    }
    let valid = if operation.compares() {
        inputs == [word; 2]
            && matches!(types.kind(result), Some(Kind::Tuple(fields))
                if types.list(fields) == Some(&[word, TypeId::primitive(Primitive::Bool)]))
    } else {
        inputs == [operand] && result == word
    };
    if !valid {
        return Err("source atomic update has different value inputs or result");
    }
    Ok((pointer, result))
}
