//! Sequence loops borrow elements without reading or moving their storage.
mod ordering;
use super::{
    Body, Declarations, Kind, Local, LocalId, LocalKind, Place, Projection, ProjectionId,
    ProjectionKind, Rvalue, TypeId, Validated, ValueId, dereference, statement, value,
};
use crate::{
    body::ir::{Action, Block, BlockId, ScalarId, Terminator, ValueList},
    lex::token::Punctuation,
    sema::model::Scalar,
    types::{Primitive, arguments::Argument},
};
const INDEX: LocalId = LocalId(2);
fn integer(body: &mut Body, bits: u128, ty: TypeId) -> Result<ValueId, &'static str> {
    let id = ScalarId(u32::try_from(body.scalars.len()).map_err(|_| "too many sequence scalars")?);
    body.scalars
        .push(if ty == TypeId::primitive(Primitive::Bool) {
            Scalar::Boolean(bits != 0)
        } else {
            Scalar::Integer(bits)
        });
    value(body, ty, Rvalue::Scalar(id))
}
fn block(body: &mut Body, start: u32, terminator: Terminator) -> Result<u32, &'static str> {
    let end = u32::try_from(body.statements.len()).map_err(|_| "too many sequence statements")?;
    body.blocks.push(Block {
        start,
        end,
        terminator,
        source: body.source.node,
    });
    Ok(end)
}
fn container(
    body: &mut Body,
    facts: &Declarations,
    ordinal: u32,
    parameter: TypeId,
) -> Result<(Place, TypeId), &'static str> {
    let place = dereference(
        body,
        Place::Local(LocalId(ordinal)),
        parameter,
        &facts.types,
    )?;
    let Some(Kind::Reference { referent, .. }) = facts.types.kind(parameter) else {
        return Err("sequence parameter is not borrowed");
    };
    if matches!(facts.types.kind(referent), Some(Kind::Reference { .. })) {
        let place = dereference(body, place, referent, &facts.types)?;
        let Some(Kind::Reference { referent, .. }) = facts.types.kind(referent) else {
            return Err("sequence reference lost referent");
        };
        Ok((place, referent))
    } else {
        Ok((place, referent))
    }
}
fn indexed(
    body: &mut Body,
    facts: &Declarations,
    place: Place,
    ty: TypeId,
    index: ValueId,
) -> Result<(Place, TypeId), &'static str> {
    let (element, kind) = match facts.types.kind(ty) {
        Some(Kind::Array { element, length }) => (element, ProjectionKind::Index { index, length }),
        Some(Kind::Slice(element)) => (element, ProjectionKind::SliceIndex { index }),
        _ => return Err("sequence loop requires a concrete array or slice"),
    };
    let id = ProjectionId(
        u32::try_from(body.projections.len()).map_err(|_| "too many sequence projections")?,
    );
    body.projections.push(Projection {
        base: place,
        kind,
        ty: element,
    });
    statement(body, Action::Project(id));
    Ok((Place::Projection(id), element))
}
fn element_call(
    body: &mut Body,
    facts: &mut Declarations,
    source: &Validated,
    left: (Place, TypeId),
    right: (Place, TypeId),
) -> Result<ValueId, &'static str> {
    let ((left, a), (right, b)) = (left, right);
    let ne = if source.comparison == Some(crate::support::comparisons::Method::Eq) {
        facts
            .associated
            .library_defaults()
            .find_map(|(definition, default)| match default {
                crate::sema::associated::LibraryDefault::PartialEqNe { eq }
                    if eq == source.definition =>
                {
                    Some(definition)
                }
                _ => None,
            })
            .ok_or("sequence equality has no canonical element ne member")?
    } else {
        source.definition
    };
    let (callable, opened) =
        super::forwarded::callable(facts, ne, vec![Argument::Type(a), Argument::Type(b)])?;
    let Some(Kind::Function {
        parameters: call_parameters,
        result,
        ..
    }) = facts.types.kind(opened)
    else {
        return Err("sequence element comparison is not callable");
    };
    let borrowed = facts
        .types
        .list(call_parameters)
        .ok_or("missing sequence borrow parameters")?
        .to_vec();
    if borrowed.len() != 2 || result != source.result {
        return Err("sequence element comparison has invalid signature");
    }
    let first = u32::try_from(body.arguments.len()).map_err(|_| "too many sequence arguments")?;
    for (place, ty) in [left, right].into_iter().zip(borrowed) {
        let argument = value(
            body,
            ty,
            Rvalue::Borrow {
                place,
                kind: crate::body::ir::BorrowKind::Immediate,
            },
        )?;
        body.arguments.push(argument);
    }
    let callee = value(body, callable, Rvalue::Function(ne))?;
    value(
        body,
        result,
        Rvalue::Call {
            signature: opened,
            callee,
            arguments: ValueList {
                start: first,
                end: first.checked_add(2).ok_or("too many sequence arguments")?,
            },
        },
    )
}
pub(super) fn operation(
    body: &mut Body,
    facts: &mut Declarations,
    source: &Validated,
    parameters: [TypeId; 2],
) -> Result<(), &'static str> {
    if source.comparison != Some(crate::support::comparisons::Method::Eq) {
        return ordering::operation(body, facts, source, parameters);
    }
    let usize_ = TypeId::primitive(Primitive::Usize);
    let boolean = TypeId::primitive(Primitive::Bool);
    body.locals.push(Local {
        scope: crate::body::scopes::Id(0),
        storage_parent: Some(LocalId(1)),
        ty: usize_,
        kind: LocalKind::Binding,
        definition: None,
        mutable: true,
        source: body.source.node,
    });
    statement(body, Action::StorageLive(INDEX));
    let zero = integer(body, 0, usize_)?;
    statement(
        body,
        Action::Store {
            place: Place::Local(INDEX),
            value: zero,
        },
    );
    let (left, a) = container(body, facts, 0, parameters[0])?;
    let (right, b) = container(body, facts, 1, parameters[1])?;
    let len = value(body, usize_, Rvalue::Length(left))?;
    let other = value(body, usize_, Rvalue::Length(right))?;
    let same = value(
        body,
        boolean,
        Rvalue::Binary {
            operands: crate::body::ir::BinaryOperands::Neither,
            operator: Punctuation::EqualEqual,
            left: len,
            right: other,
        },
    )?;
    let start = super::ordering::branch(body, 0, same, 1, 5)?;
    let index = value(body, usize_, Rvalue::Read(Place::Local(INDEX)))?;
    let more = value(
        body,
        boolean,
        Rvalue::Binary {
            operands: crate::body::ir::BinaryOperands::Neither,
            operator: Punctuation::Less,
            left: index,
            right: len,
        },
    )?;
    let start = super::ordering::branch(body, start, more, 2, 4)?;
    let (left, a) = indexed(body, facts, left, a, index)?;
    let (right, b) = indexed(body, facts, right, b, index)?;
    let different = element_call(body, facts, source, (left, a), (right, b))?;
    let start = super::ordering::branch(body, start, different, 5, 3)?;
    let one = integer(body, 1, usize_)?;
    let next = value(
        body,
        usize_,
        Rvalue::Binary {
            operands: crate::body::ir::BinaryOperands::Neither,
            operator: Punctuation::Plus,
            left: index,
            right: one,
        },
    )?;
    statement(
        body,
        Action::Store {
            place: Place::Local(INDEX),
            value: next,
        },
    );
    let start = block(body, start, Terminator::Jump(BlockId(1)))?;
    let yes = integer(body, 1, boolean)?;
    crate::body::build::finish_factory_block(body, start, yes)?;
    let start = u32::try_from(body.statements.len()).map_err(|_| "too many sequence statements")?;
    let no = integer(body, 0, boolean)?;
    crate::body::build::finish_factory_block(body, start, no)
}
pub(super) fn validate(body: &Body, source: &Validated) -> Result<(), &'static str> {
    let Some(local) = body.locals.get(INDEX.0 as usize) else {
        return Err("sequence loop has no index local");
    };
    if local.ty != TypeId::primitive(Primitive::Usize)
        || local.kind != LocalKind::Binding
        || !local.mutable
        || local.definition.is_some()
        || local.storage_parent != Some(LocalId(1))
        || local.scope != crate::body::scopes::Id(0)
    {
        return Err("sequence factory index local differs from its checked schema");
    }
    if source.comparison != Some(crate::support::comparisons::Method::Eq) {
        ordering::validate(body, source)?;
    }
    Ok(())
}
