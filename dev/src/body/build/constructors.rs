//! Constructor factories share one retained field-move CFG across ordinary and generic uses.
use super::{Action, Error, Owner, Place, Rvalue, Type, TypeId, ValueId, ValueList};
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
use std::{collections::HashMap, sync::Arc};

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
            let variant = *facts.members.constructors.get(&definition)?;
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
    let mut environments = HashMap::new();
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
        let error = |message| Declarations::error(report, declaration.node, message);
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
        contracts
            .push(crate::body::regions::constructor(facts, declaration, signature).map_err(error)?);
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
    let error = |message| Declarations::error(report, variant.node, message);
    let Some(Type::Function { result, .. }) = facts.types.kind(signature) else {
        return Err(error("missing constructor signature"));
    };
    let mut body = super::Body::new(
        Owner::Function {
            definition,
            signature,
        },
        variant.node,
        result,
    );
    body.storage = Some(crate::body::storage::Facts::new(
        crate::body::scopes::Facts::synthetic(variant.node.node),
    ));
    for (ordinal, field) in facts
        .members
        .fields(variant_id)
        .ok_or_else(|| error("missing constructor fields"))?
        .iter()
        .enumerate()
    {
        let ordinal = u32::try_from(ordinal).map_err(|_| error("too many constructor fields"))?;
        let local = LocalId(ordinal);
        body.locals.push(Local {
            scope: crate::body::scopes::Id(0),
            storage_parent: ordinal.checked_sub(1).map(LocalId),
            ty: field.ty,
            kind: LocalKind::Parameter(ordinal),
            definition: None,
            mutable: false,
            source: field.node.node,
        });
        body.statements.push(Statement {
            source: field.node.node,
            action: Action::StorageLive(local),
        });
        let value = ValueId(ordinal);
        body.values.push(Value {
            ty: field.ty,
            kind: Rvalue::Read(Place::Local(local)),
        });
        body.arguments.push(value);
        body.statements.push(Statement {
            source: field.node.node,
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
    use super::{Block, LocalId, Statement, Terminator, Value};
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
        source: body.source.node,
        action: Action::Evaluate(value),
    });
    let before =
        u32::try_from(body.statements.len()).map_err(|_| "too many constructor statements")?;
    body.storage
        .as_mut()
        .ok_or("missing constructor storage")?
        .exits
        .push(crate::body::storage::Exit {
            location: crate::body::storage::Location::At {
                block: super::BlockId(0),
                before,
            },
            end: crate::body::storage::End::Scopes(crate::body::storage::Range {
                inner: crate::body::scopes::Id(0),
                outer: None,
            }),
        });
    if let Some(last) = count.checked_sub(1).map(LocalId) {
        body.statements.push(Statement {
            source: body.source.node,
            action: Action::end_storage(LocalId(0), last),
        });
    }
    let end =
        u32::try_from(body.statements.len()).map_err(|_| "too many constructor statements")?;
    body.blocks.push(Block {
        start: 0,
        end,
        source: body.source.node,
        terminator: Terminator::Return(value),
    });
    Ok(())
}
