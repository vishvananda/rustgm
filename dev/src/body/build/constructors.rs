//! Constructor factories share one retained field-move CFG across ordinary and generic uses.
use super::{Action, Error, Owner, Place, Rvalue, Type, TypeId, ValueId, ValueList};
use crate::support::hash::HashMap;
use crate::{
    body::{
        ir::Body,
        regions::Contract,
        templates::{Template, Templates},
    },
    resolve::{expansion::Report, model::DefId},
    sema::{members::VariantId, model::Declarations},
    types::destruction::NeedsDrop,
};
use std::sync::Arc;

fn selected(facts: &Declarations) -> impl Iterator<Item = (DefId, TypeId, VariantId)> + '_ {
    facts
        .definitions
        .iter()
        .enumerate()
        .filter_map(|(index, ty)| {
            let Type::FunctionItem {
                item, signature, ..
            } = ty.and_then(|ty| facts.types.kind(ty))?
            else {
                return None;
            };
            if item.0 as usize != index {
                return None;
            }
            let definition = DefId(item.0);
            let variant = facts.members.constructor(definition)?;
            Some((definition, signature, variant))
        })
}

pub(super) fn concrete(
    report: &Report,
    facts: &Declarations,
    program: &mut super::Program,
    mut query: Option<&mut NeedsDrop>,
) -> Result<(), Error> {
    for (definition, signature, variant) in selected(facts) {
        let owner = facts.members.variants[variant.0 as usize].owner;
        if facts.generics.has_parameters(DefId(owner.0)) {
            continue;
        }
        program.bodies.push(build(
            report,
            facts,
            definition,
            signature,
            variant,
            query.as_deref_mut(),
        )?);
    }
    Ok(())
}

/// Add referenced implicit generic constructor bodies and their original contracts.
/// Nominal assumptions are shared by all variants; these templates remain separate
/// from concrete bodies until demand selects their complete type/const arguments.
/// # Errors
/// Rejects missing declaration facts, invalid scopes and construction limits.
pub fn templates(
    report: &Report,
    facts: &mut Declarations,
    templates: &mut Templates,
    contracts: &mut Vec<Contract>,
) -> Result<(), Error> {
    let mut environments = HashMap::default();
    let query = NeedsDrop::new(facts.destructors.implementations().iter().map(|i| i.item));
    let selected = selected(facts)
        .filter(|(definition, _, variant)| {
            facts
                .generics
                .referenced_constructors
                .contains(&crate::types::ItemId(definition.0))
                && facts
                    .generics
                    .has_parameters(DefId(facts.members.variants[variant.0 as usize].owner.0))
        })
        .collect::<Vec<_>>();
    for (definition, signature, variant) in selected {
        let declaration = facts.members.variants[variant.0 as usize];
        let owner = DefId(declaration.owner.0);
        let source = declaration
            .origin
            .require_diagnostic()
            .map_err(|e| Error::at(crate::resolve::unit::FileId(0), 0, e))?;
        let error = |message| report.body_error(source, message);
        let environment = match environments.entry(owner) {
            std::collections::hash_map::Entry::Occupied(entry) => Arc::clone(entry.get()),
            std::collections::hash_map::Entry::Vacant(entry) => Arc::clone(
                entry.insert(Arc::new(
                    facts
                        .generics
                        .parameter_environment(&facts.types, owner)
                        .map_err(error)?,
                )),
            ),
        };
        let Some(Type::Function { result, .. }) = facts.types.kind(signature) else {
            return Err(error("missing constructor signature"));
        };
        facts
            .aggregates
            .prepare(&mut facts.types, result)
            .map_err(error)?;
        let mut scoped = query.scoped(Arc::clone(&environment));
        let body = build(
            report,
            facts,
            definition,
            signature,
            variant,
            Some(&mut scoped),
        )?;
        let destruction =
            crate::body::drops::requirements(facts, &body, &mut scoped).map_err(error)?;
        contracts.push(
            crate::body::regions::constructor(facts, definition, declaration, signature)
                .map_err(error)?,
        );
        templates.bodies.push(Template {
            body,
            destruction,
            environment,
        });
    }
    Ok(())
}

fn build(
    report: &Report,
    facts: &Declarations,
    definition: DefId,
    signature: TypeId,
    variant_id: VariantId,
    drop_query: Option<&mut NeedsDrop>,
) -> Result<Body, Error> {
    use super::{Local, LocalId, LocalKind, Statement, Value};
    let variant = facts.members.variants[variant_id.0 as usize];
    let source = variant
        .origin
        .require_diagnostic()
        .map_err(|e| Error::at(crate::resolve::unit::FileId(0), 0, e))?;
    let error = |message| report.body_error(source, message);
    let Some(Type::Function { result, .. }) = facts.types.kind(signature) else {
        return Err(error("missing constructor signature"));
    };
    let mut body = super::Body::new(
        Owner::Function {
            definition,
            signature,
            execution: None,
        },
        source,
        result,
    );
    body.const_context = true;
    body.storage = Some(crate::body::storage::Facts::new(
        crate::body::scopes::Facts::synthetic(source.node),
    ));
    for (ordinal, field) in facts
        .members
        .fields(variant_id)
        .ok_or_else(|| error("missing constructor fields"))?
        .iter()
        .enumerate()
    {
        let field_source = field.origin.require_diagnostic().map_err(error)?;
        let ordinal = u32::try_from(ordinal).map_err(|_| error("too many constructor fields"))?;
        let local = LocalId(ordinal);
        body.locals.push(Local {
            scope: crate::body::scopes::Id(0),
            storage_parent: ordinal.checked_sub(1).map(LocalId),
            ty: field.ty,
            kind: LocalKind::Parameter(ordinal),
            definition: None,
            mutable: false,
            source: field_source.node.into(),
        });
        body.statements.push(Statement {
            source: field_source.node.into(),
            action: Action::StorageLive(local),
        });
        let value = ValueId(ordinal);
        body.values.push(Value {
            ty: field.ty,
            kind: Rvalue::Read(Place::Local(local)),
        });
        body.arguments.push(value);
        body.statements.push(Statement {
            source: field_source.node.into(),
            action: Action::Evaluate(value),
        });
    }
    finish_constructor(&mut body, variant_id).map_err(error)?;
    if let Some(query) = drop_query {
        super::drops::constructor(facts, &mut body, query).map_err(error)?;
    }
    Ok(body)
}

fn finish_constructor(
    body: &mut super::Body,
    variant_id: crate::sema::members::VariantId,
) -> Result<(), &'static str> {
    use super::{Statement, Value};
    let source = body
        .source
        .source()
        .ok_or("source constructor requires a real declaration")?
        .node;
    let count =
        u32::try_from(body.arguments.len()).map_err(|_| "too many constructor arguments")?;
    let value = ValueId(count);
    body.values.push(Value {
        ty: body.result,
        kind: Rvalue::Aggregate {
            variant: Some(variant_id),
            fields: ValueList {
                start: 0,
                end: count,
            },
        },
    });
    body.statements.push(Statement {
        source: source.into(),
        action: Action::Evaluate(value),
    });
    finish_factory(body, value)
}

pub(in crate::body) fn finish_factory(body: &mut Body, value: ValueId) -> Result<(), &'static str> {
    finish_factory_block(body, 0, value)
}

pub(in crate::body) fn finish_factory_block(
    body: &mut Body,
    start: u32,
    value: ValueId,
) -> Result<(), &'static str> {
    use super::{Block, LocalId, Statement, Terminator};
    let source = body
        .source
        .source()
        .ok_or("source factory requires a real declaration")?
        .node;
    let block =
        super::BlockId(u32::try_from(body.blocks.len()).map_err(|_| "too many factory blocks")?);
    let count = u32::try_from(body.locals.len()).map_err(|_| "too many factory parameters")?;
    let before =
        u32::try_from(body.statements.len()).map_err(|_| "too many constructor statements")?;
    body.storage
        .as_mut()
        .ok_or("missing constructor storage")?
        .exits
        .push(crate::body::storage::Exit {
            location: crate::body::storage::Location::At { block, before },
            end: crate::body::storage::End::Scopes(crate::body::storage::Range {
                inner: crate::body::scopes::Id(0),
                outer: None,
            }),
        });
    if let Some(last) = count.checked_sub(1).map(LocalId) {
        body.statements.push(Statement {
            source: source.into(),
            action: Action::end_storage(LocalId(0), last),
        });
    }
    let end =
        u32::try_from(body.statements.len()).map_err(|_| "too many constructor statements")?;
    body.blocks.push(Block {
        start,
        end,
        source: source.into(),
        terminator: Terminator::Return(value),
    });
    Ok(())
}
