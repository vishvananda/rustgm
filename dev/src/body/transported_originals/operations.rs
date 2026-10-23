//! Convert retained operation rows directly; source text is never reconstructed.
mod lexical;
use super::ImportedOriginals;
use crate::types::TypeId;
use crate::{
    body::{
        ir::{
            Action, Block, BlockId, Body, BodySite, DefId, Local, LocalId, LocalKind, Owner, Place,
            Rvalue, Statement, Temporary, Terminator, Value, ValueId,
        },
        scopes, storage,
    },
    resolve::{body_origin::Catalog, crates::Imported},
    support::crates::{CrateIdentity, generics as g},
};
pub(super) fn allocation(raw: &g::Original) -> usize {
    raw.locals
        .len()
        .saturating_mul(size_of::<Local>())
        .saturating_add(raw.values.len().saturating_mul(size_of::<Value>()))
        .saturating_add(raw.statements.len().saturating_mul(size_of::<Statement>()))
        .saturating_add(raw.blocks.len().saturating_mul(size_of::<Block>()))
        .saturating_add(raw.scopes.len().saturating_mul(size_of::<storage::Scope>()))
        .saturating_add(raw.exits.len().saturating_mul(size_of::<storage::Exit>()))
        .saturating_add(raw.temporaries.len().saturating_mul(size_of::<Temporary>()))
}
pub(super) struct Context<'a> {
    pub imported: &'a Imported,
    pub catalog: &'a Catalog,
    pub owner: &'a CrateIdentity,
    pub types: &'a [TypeId],
}
pub(super) fn build(
    context: &Context<'_>,
    raw: &g::Original,
    definition: DefId,
    signature: TypeId,
    constant: bool,
    work: &mut ImportedOriginals,
) -> Result<Body, &'static str> {
    let id = context.imported.generic_body(context.owner, raw.body)?;
    let source = context.catalog.origin(id)?;
    context
        .catalog
        .validate_function_origin(definition, source)?;
    let mut body = Body::original(
        Owner::Function {
            definition,
            signature,
            execution: None,
        },
        source,
        context.types[raw.result.0 as usize],
    );
    body.const_context = constant;
    let site = |index| context.catalog.site(id, index);
    work.charge(
        raw.locals
            .len()
            .saturating_add(raw.values.len())
            .saturating_add(raw.statements.len())
            .saturating_add(raw.blocks.len())
            .saturating_add(raw.scopes.len())
            .saturating_add(raw.exits.len())
            .saturating_add(raw.temporaries.len()),
    )?;
    ports(context, raw, &mut body, &site)?;
    control(raw, &mut body, &site)?;
    lexical::fill(raw, &mut body, &site)?;
    Ok(body)
}
fn ports(
    context: &Context<'_>,
    raw: &g::Original,
    body: &mut Body,
    site: &impl Fn(u32) -> Result<BodySite, &'static str>,
) -> Result<(), &'static str> {
    body.locals = raw
        .locals
        .iter()
        .map(|local| {
            Ok(Local {
                scope: scopes::Id(local.scope),
                storage_parent: local.storage_parent.map(LocalId),
                ty: context.types[local.ty.0 as usize],
                kind: match local.kind {
                    g::LocalKind::Parameter(index) => LocalKind::Parameter(index),
                    g::LocalKind::Binding => LocalKind::Binding,
                    g::LocalKind::Merge => LocalKind::Merge,
                },
                definition: local
                    .definition
                    .map(|d| context.imported.generic_definition(context.owner, d))
                    .transpose()?,
                mutable: local.mutable,
                source: site(local.site)?,
            })
        })
        .collect::<Result<_, &'static str>>()?;
    body.values = raw
        .values
        .iter()
        .map(|value| Value {
            ty: context.types[value.ty.0 as usize],
            kind: match value.kind {
                g::ValueKind::Unit => Rvalue::Unit,
                g::ValueKind::Read(local) => Rvalue::Read(Place::Local(LocalId(local))),
                g::ValueKind::Constrain(value) => Rvalue::Constrain(ValueId(value)),
            },
        })
        .collect();
    Ok(())
}
fn control(
    raw: &g::Original,
    body: &mut Body,
    site: &impl Fn(u32) -> Result<BodySite, &'static str>,
) -> Result<(), &'static str> {
    body.statements = raw
        .statements
        .iter()
        .map(|s| {
            Ok(Statement {
                source: site(s.site)?,
                action: match s.action {
                    g::Action::StorageLive(local) => Action::StorageLive(LocalId(local)),
                    g::Action::StorageDead(local) => Action::StorageDead(LocalId(local)),
                    g::Action::StorageEnd { first, last } => Action::StorageEnd {
                        first: LocalId(first),
                        last: LocalId(last),
                    },
                    g::Action::Evaluate(value) => Action::Evaluate(ValueId(value)),
                    g::Action::Store { local, value } => Action::Store {
                        place: Place::Local(LocalId(local)),
                        value: ValueId(value),
                    },
                    g::Action::CheckInitialized(local) => {
                        Action::CheckInitialized(Place::Local(LocalId(local)))
                    }
                    g::Action::Mention(local) => Action::Mention(Place::Local(LocalId(local))),
                },
            })
        })
        .collect::<Result<_, &'static str>>()?;
    body.blocks = raw
        .blocks
        .iter()
        .map(|block| {
            Ok(Block {
                start: block.statements.start,
                end: block.statements.end,
                source: site(block.site)?,
                terminator: match block.terminator {
                    g::Terminator::Return(value) => Terminator::Return(ValueId(value)),
                    g::Terminator::Jump(block) => Terminator::Jump(BlockId(block)),
                    g::Terminator::Unreachable => Terminator::Unreachable,
                },
            })
        })
        .collect::<Result<_, &'static str>>()?;
    Ok(())
}
