//! next splits the shared source slice, then updates only the iterator's stored view.
use super::{
    Action, Body, Declarations, Kind, LocalId, LocalKind, Original, Place, ProjectionKind, Rvalue,
    TypeId, ValueList, dereference, iterator_variant, local, projection, slice_reference,
    statement, value,
};
use crate::{
    body::ir::{Block, BlockId, BorrowKind, ScalarId, Terminator},
    lex::token::Punctuation,
    sema::members::{Name, Selection},
    types::Primitive,
};
pub(super) fn build(
    facts: &mut Declarations,
    original: Original,
    body: &mut Body,
) -> Result<(), &'static str> {
    let (stored, slice, reference, element) = snapshot(facts, original, body)?;
    let empty = empty_test(body, slice)?;
    let end = u32::try_from(body.statements.len()).map_err(|_| "too many next statements")?;
    body.blocks.push(Block {
        start: 0,
        end,
        source: body.root_site(),
        terminator: Terminator::Branch {
            condition: empty,
            yes: BlockId(1),
            no: BlockId(2),
        },
    });
    let Some(Kind::Adt(option, _)) = facts.types.kind(body.result) else {
        return Err("next result is not Option");
    };
    let [none, some] = facts.members.variants(option) else {
        return Err("next requires canonical Option variants");
    };
    let (none, some) = (*none, *some);
    let returned = value(
        body,
        body.result,
        Rvalue::Aggregate {
            variant: Some(none),
            fields: ValueList { start: 0, end: 0 },
        },
    )?;
    crate::body::build::finish_factory_block(body, end, returned)?;
    let start = u32::try_from(body.statements.len()).map_err(|_| "too many next statements")?;
    let slice = if original.declaration.kind.mutable() {
        super::mutable_next::snapshot(facts, original, body, stored, reference)?
    } else {
        slice
    };
    let first = projection(body, slice, element, ProjectionKind::ConstantIndex(0))?;
    let first = value(
        body,
        original.item,
        Rvalue::Borrow {
            place: first,
            kind: BorrowKind::Immediate,
        },
    )?;
    let rest = projection(
        body,
        slice,
        body.place_type(slice).ok_or("missing next slice type")?,
        ProjectionKind::Subslice {
            prefix: 1,
            suffix: 0,
        },
    )?;
    let rest = value(
        body,
        reference,
        Rvalue::Borrow {
            place: rest,
            kind: BorrowKind::Immediate,
        },
    )?;
    statement(
        body,
        Action::Store {
            place: stored,
            value: rest,
        },
    );
    let argument_start =
        u32::try_from(body.arguments.len()).map_err(|_| "too many next arguments")?;
    body.arguments.push(first);
    let returned = value(
        body,
        body.result,
        Rvalue::Aggregate {
            variant: Some(some),
            fields: ValueList {
                start: argument_start,
                end: argument_start + 1,
            },
        },
    )?;
    crate::body::build::finish_factory_block(body, start, returned)
}

fn snapshot(
    facts: &mut Declarations,
    original: Original,
    body: &mut Body,
) -> Result<(Place, Place, TypeId, TypeId), &'static str> {
    let iterator = dereference(facts, body, Place::Local(LocalId(0)))?;
    if body.place_type(iterator) != Some(original.iterator) {
        return Err("next receiver differs from canonical iterator");
    }
    let variant = iterator_variant(facts, original)?;
    let name = facts.members.names.intern("value")?;
    let field = facts
        .members
        .field(variant, Name::Named(name))
        .ok_or("missing iterator source field")?;
    let reference = slice_reference(facts, original)?;
    let stored = projection(
        body,
        iterator,
        reference,
        ProjectionKind::Field(Selection::Nominal(field)),
    )?;
    let slice = if original.declaration.kind.mutable() {
        dereference(facts, body, stored)?
    } else {
        let copied = value(body, reference, Rvalue::Read(stored))?;
        let snapshot = local(body, reference, LocalKind::Binding, false)?;
        statement(
            body,
            Action::Store {
                place: Place::Local(snapshot),
                value: copied,
            },
        );
        dereference(facts, body, Place::Local(snapshot))?
    };
    let Some(Kind::Slice(element)) = body.place_type(slice).and_then(|t| facts.types.kind(t))
    else {
        return Err("next source is not slice");
    };
    if element != original.parameter {
        return Err("next source differs from element schema");
    }
    Ok((stored, slice, reference, element))
}

fn empty_test(body: &mut Body, slice: Place) -> Result<crate::body::ir::ValueId, &'static str> {
    let length = value(
        body,
        TypeId::primitive(Primitive::Usize),
        Rvalue::Length(slice),
    )?;
    body.scalars.push(crate::sema::model::Scalar::Integer(0));
    let zero = value(
        body,
        TypeId::primitive(Primitive::Usize),
        Rvalue::Scalar(ScalarId(0)),
    )?;
    value(
        body,
        TypeId::primitive(Primitive::Bool),
        Rvalue::Binary {
            operands: crate::body::ir::BinaryOperands::Neither,
            operator: Punctuation::EqualEqual,
            left: length,
            right: zero,
        },
    )
}
