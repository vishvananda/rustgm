//! Lexicographic ordering preserves the selected element comparison and first non-equal result.
use super::super::ordering::{Results, append, branch};
use super::{
    Action, BlockId, Body, Declarations, INDEX, Local, LocalId, LocalKind, Place, Primitive,
    Projection, ProjectionId, ProjectionKind, Rvalue, Terminator, TypeId, Validated, ValueId,
    block, container, element_call, indexed, integer, statement, value,
};
use crate::{
    lex::token::Punctuation,
    sema::members::{Name, Selection},
};
const RESULT: LocalId = LocalId(3);
struct Inputs {
    left: (Place, TypeId),
    right: (Place, TypeId),
    length: ValueId,
    other: ValueId,
}
fn initialize(
    body: &mut Body,
    facts: &Declarations,
    parameters: [TypeId; 2],
) -> Result<Inputs, &'static str> {
    let usize_ = TypeId::primitive(Primitive::Usize);
    for (id, ty) in [(INDEX, usize_), (RESULT, body.result)] {
        body.locals.push(Local {
            scope: crate::body::scopes::Id(0),
            storage_parent: Some(LocalId(id.0 - 1)),
            ty,
            kind: LocalKind::Binding,
            definition: None,
            mutable: true,
            source: body.source.node,
        });
        statement(body, Action::StorageLive(id));
    }
    let zero = integer(body, 0, usize_)?;
    statement(
        body,
        Action::Store {
            place: Place::Local(INDEX),
            value: zero,
        },
    );
    let left = container(body, facts, 0, parameters[0])?;
    let right = container(body, facts, 1, parameters[1])?;
    let length = value(body, usize_, Rvalue::Length(left.0))?;
    let other = value(body, usize_, Rvalue::Length(right.0))?;
    Ok(Inputs {
        left,
        right,
        length,
        other,
    })
}
fn equal(
    body: &mut Body,
    facts: &Declarations,
    results: &Results,
) -> Result<ValueId, &'static str> {
    let mut place = Place::Local(RESULT);
    if let Some([_, some]) = results.option {
        let field = facts
            .members
            .field(some, Name::Index(0))
            .ok_or("missing ordering payload")?;
        let id = ProjectionId(
            u32::try_from(body.projections.len()).map_err(|_| "too many sequence projections")?,
        );
        body.projections.push(Projection {
            base: place,
            kind: ProjectionKind::Field(Selection::Nominal(field)),
            ty: results.ordering,
        });
        statement(body, Action::Project(id));
        place = Place::Projection(id);
    }
    value(
        body,
        TypeId::primitive(Primitive::Bool),
        Rvalue::IsVariant {
            place,
            variant: results.variants[1],
        },
    )
}
pub(super) fn operation(
    body: &mut Body,
    facts: &mut Declarations,
    source: &Validated,
    parameters: [TypeId; 2],
) -> Result<(), &'static str> {
    let input = initialize(body, facts, parameters)?;
    let results = Results::new(facts, source)?;
    let boolean = TypeId::primitive(Primitive::Bool);
    let usize_ = TypeId::primitive(Primitive::Usize);
    let start = block(body, 0, Terminator::Jump(BlockId(1)))?;
    let index = value(body, usize_, Rvalue::Read(Place::Local(INDEX)))?;
    let left = value(
        body,
        boolean,
        Rvalue::Binary {
            operands: crate::body::ir::BinaryOperands::Neither,
            operator: Punctuation::Less,
            left: index,
            right: input.length,
        },
    )?;
    let right = value(
        body,
        boolean,
        Rvalue::Binary {
            operands: crate::body::ir::BinaryOperands::Neither,
            operator: Punctuation::Less,
            left: index,
            right: input.other,
        },
    )?;
    let more = value(
        body,
        boolean,
        Rvalue::Binary {
            operands: crate::body::ir::BinaryOperands::Neither,
            operator: Punctuation::And,
            left,
            right,
        },
    )?;
    let start = branch(body, start, more, 2, 6)?;
    let left = indexed(body, facts, input.left.0, input.left.1, index)?;
    let right = indexed(body, facts, input.right.0, input.right.1, index)?;
    let comparison = element_call(body, facts, source, left, right)?;
    statement(
        body,
        Action::Store {
            place: Place::Local(RESULT),
            value: comparison,
        },
    );
    let start = if let Some([_, some]) = results.option {
        let some = value(
            body,
            boolean,
            Rvalue::IsVariant {
                place: Place::Local(RESULT),
                variant: some,
            },
        )?;
        branch(body, start, some, 3, 5)?
    } else {
        block(body, start, Terminator::Jump(BlockId(3)))?
    };
    let same = equal(body, facts, &results)?;
    let start = branch(body, start, same, 4, 5)?;
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
    let result = value(body, source.result, Rvalue::Read(Place::Local(RESULT)))?;
    crate::body::build::finish_factory_block(body, start, result)?;
    let start = u32::try_from(body.statements.len()).map_err(|_| "too many sequence statements")?;
    append(body, facts, source, start, input.length, input.other)
}
pub(super) fn validate(body: &Body, source: &Validated) -> Result<(), &'static str> {
    let local = body
        .locals
        .get(RESULT.0 as usize)
        .ok_or("missing sequence comparison local")?;
    if local.ty != source.result
        || local.kind != LocalKind::Binding
        || !local.mutable
        || local.definition.is_some()
        || local.storage_parent != Some(INDEX)
        || local.scope != crate::body::scopes::Id(0)
    {
        return Err("sequence ordering local differs from checked result schema");
    }
    Ok(())
}
