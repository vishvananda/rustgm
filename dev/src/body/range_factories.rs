//! Integer range next originals use ordinary scalar operations and checked bodies.
use super::{
    collection_factories::{local, projection},
    ir::{
        Action, Block, BlockId, Body, LocalId, LocalKind, Owner, Place, ProjectionKind, Rvalue,
        ScalarId, Terminator, ValueList,
    },
    provided::{statement, value},
    regions::Contract,
    templates::{Template, Templates},
};
use crate::{
    resolve::{expansion::Report, package::Error},
    sema::{
        iteration::range_originals::Original,
        members::{Name, Selection},
        model::{Declarations, Scalar},
    },
    types::{Kind, Primitive, TypeId},
};
use std::sync::Arc;
pub(super) fn append(
    report: &Report,
    facts: &mut Declarations,
    templates: &mut Templates,
    contracts: &mut Vec<Contract>,
) -> Result<(), Error> {
    let Some(registry) = facts.iteration.as_deref().and_then(|r| r.ranges.as_deref()) else {
        return Ok(());
    };
    let canonical = report.resolution.builtin_traits.ranges().ok_or_else(|| {
        report.body_error(registry.canonical.origin, "unregistered range originals")
    })?;
    if &registry.canonical != canonical {
        return Err(report.body_error(
            canonical.origin,
            "range originals differ from canonical identities",
        ));
    }
    let source = canonical.origin;
    let originals = registry.originals.clone();
    let error = |e| report.body_error(source, e);
    for original in originals {
        original.validate(facts).map_err(error)?;
        let environment = Arc::new(
            facts
                .generics
                .parameter_environment(&facts.types, original.declaration.method)
                .map_err(error)?,
        );
        let contract = super::regions::callable(
            facts,
            original.declaration.method,
            original.declaration.method,
            source,
            original.signature,
        )
        .map_err(error)?;
        let mut body = build(facts, original, source).map_err(error)?;
        let mut query = crate::types::destruction::NeedsDrop::new(
            facts.destructors.implementations().iter().map(|i| i.item),
        )
        .scoped(Arc::clone(&environment));
        super::build::factory_owned_values(facts, &mut body, &mut query).map_err(error)?;
        let destruction = super::drops::requirements(facts, &body, &mut query).map_err(error)?;
        contracts.push(contract);
        templates.bodies.push(Template {
            body,
            destruction,
            environment,
        });
    }
    Ok(())
}
fn build(
    facts: &mut Declarations,
    original: Original,
    source: crate::resolve::unit::NodeKey,
) -> Result<Body, &'static str> {
    original.validate(facts)?;
    let Some(Kind::Function {
        parameters, result, ..
    }) = facts.types.kind(original.signature)
    else {
        return Err("missing range next signature");
    };
    let [receiver] = facts
        .types
        .list(parameters)
        .ok_or("missing range receiver")?
    else {
        return Err("range receiver arity");
    };
    let receiver = *receiver;
    let Some(Kind::Reference {
        region,
        mutable: true,
        referent,
    }) = facts.types.kind(receiver)
    else {
        return Err("range next receiver is not mutable");
    };
    if referent != original.subject {
        return Err("range next receiver differs from Self");
    }
    facts
        .aggregates
        .prepare(&mut facts.types, original.subject)?;
    facts.aggregates.prepare(&mut facts.types, result)?;
    let Some(Kind::Adt(owner, _)) = facts.types.kind(original.subject) else {
        return Err("range Self is not nominal");
    };
    let [variant] = facts.members.variants(owner) else {
        return Err("range record differs");
    };
    let variant = *variant;
    let start_name = facts.members.names.intern("start")?;
    let end_name = facts.members.names.intern("end")?;
    let start = facts
        .members
        .field(variant, Name::Named(start_name))
        .ok_or("missing range start field")?;
    let end = facts
        .members
        .field(variant, Name::Named(end_name))
        .ok_or("missing range end field")?;
    let mut body = Body::new(
        Owner::Function {
            definition: original.declaration.method,
            signature: original.signature,
        },
        source,
        result,
    );
    body.storage = Some(super::storage::Facts::new(super::scopes::Facts::synthetic(
        source.node,
    )));
    local(&mut body, receiver, LocalKind::Parameter(0), false)?;
    let root = projection(
        &mut body,
        Place::Local(LocalId(0)),
        original.subject,
        ProjectionKind::Dereference {
            region,
            mutable: true,
        },
    )?;
    let start = projection(
        &mut body,
        root,
        original.element,
        ProjectionKind::Field(Selection::Nominal(start)),
    )?;
    let end = projection(
        &mut body,
        root,
        original.element,
        ProjectionKind::Field(Selection::Nominal(end)),
    )?;
    next_blocks(facts, &mut body, original.element, start, end)?;
    Ok(body)
}

fn next_blocks(
    facts: &Declarations,
    body: &mut Body,
    element: TypeId,
    start: Place,
    end: Place,
) -> Result<(), &'static str> {
    let source = body.root_site();
    let first = value(body, element, Rvalue::Read(start))?;
    let last = value(body, element, Rvalue::Read(end))?;
    let condition = value(
        body,
        TypeId::primitive(Primitive::Bool),
        Rvalue::Binary {
            operands: crate::body::ir::BinaryOperands::Neither,
            operator: crate::lex::token::Punctuation::Less,
            left: first,
            right: last,
        },
    )?;
    let end = u32::try_from(body.statements.len()).map_err(|_| "too many range statements")?;
    body.blocks.push(Block {
        start: 0,
        end,
        source,
        terminator: Terminator::Branch {
            condition,
            yes: BlockId(2),
            no: BlockId(1),
        },
    });
    let Some(Kind::Adt(option, _)) = facts.types.kind(body.result) else {
        return Err("range result is not Option");
    };
    let [none, some] = facts.members.variants(option) else {
        return Err("missing Option variants");
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
    super::build::finish_factory_block(body, end, returned)?;
    let block = u32::try_from(body.statements.len()).map_err(|_| "too many range statements")?;
    body.scalars.push(Scalar::Integer(1));
    let one = value(body, element, Rvalue::Scalar(ScalarId(0)))?;
    let next = value(
        body,
        element,
        Rvalue::Binary {
            operands: crate::body::ir::BinaryOperands::Neither,
            operator: crate::lex::token::Punctuation::Plus,
            left: first,
            right: one,
        },
    )?;
    statement(
        body,
        Action::Store {
            place: start,
            value: next,
        },
    );
    body.arguments.push(first);
    let returned = value(
        body,
        body.result,
        Rvalue::Aggregate {
            variant: Some(some),
            fields: ValueList { start: 0, end: 1 },
        },
    )?;
    super::build::finish_factory_block(body, block, returned)?;
    Ok(())
}
