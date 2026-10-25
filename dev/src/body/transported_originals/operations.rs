//! Convert retained operation rows directly; source text is never reconstructed.
pub(super) mod lexical;
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
pub(super) fn allocation(record: &g::Original) -> usize {
    record
        .arguments
        .len()
        .saturating_mul(size_of::<ValueId>())
        .saturating_add(
            record
                .locals
                .len()
                .saturating_mul(size_of::<Local>())
                .saturating_add(record.values.len().saturating_mul(size_of::<Value>()))
                .saturating_add(
                    record
                        .statements
                        .len()
                        .saturating_mul(size_of::<Statement>()),
                )
                .saturating_add(record.blocks.len().saturating_mul(size_of::<Block>()))
                .saturating_add(
                    record
                        .scopes
                        .len()
                        .saturating_mul(size_of::<storage::Scope>()),
                )
                .saturating_add(
                    record
                        .exits
                        .len()
                        .saturating_mul(size_of::<storage::Exit>()),
                )
                .saturating_add(
                    record
                        .temporaries
                        .len()
                        .saturating_mul(size_of::<Temporary>()),
                ),
        )
}
pub(super) struct Context<'a> {
    pub imported: &'a Imported,
    pub catalog: &'a Catalog,
    pub owner: &'a CrateIdentity,
    pub types: &'a [TypeId],
    pub linkage: &'a crate::sema::crates::Linkage,
}
pub(super) fn build(
    context: &Context<'_>,
    record: &g::Original,
    definition: DefId,
    signature: TypeId,
    constant: bool,
    work: &mut ImportedOriginals,
) -> Result<Body, &'static str> {
    let id = context.imported.generic_body(context.owner, record.body)?;
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
        context.types[record.result.0 as usize],
    );
    body.const_context = constant;
    let site = |index| context.catalog.site(id, index);
    work.charge(
        record
            .locals
            .len()
            .saturating_add(record.values.len())
            .saturating_add(record.statements.len())
            .saturating_add(record.blocks.len())
            .saturating_add(record.scopes.len())
            .saturating_add(record.exits.len())
            .saturating_add(record.temporaries.len())
            .saturating_add(record.arguments.len()),
    )?;
    ports(context, record, &mut body, &site)?;
    control(record, &mut body, &site)?;
    lexical::fill(record, &mut body, &site)?;
    Ok(body)
}
fn ports(
    context: &Context<'_>,
    record: &g::Original,
    body: &mut Body,
    site: &impl Fn(u32) -> Result<BodySite, &'static str>,
) -> Result<(), &'static str> {
    body.locals = locals(&record.locals, |local| {
        Ok(LocalData {
            ty: context
                .types
                .get(local.ty.0 as usize)
                .copied()
                .ok_or("imported local type is outside its actual graph")?,
            definition: local
                .definition
                .map(|d| context.imported.generic_definition(context.owner, d))
                .transpose()?,
            source: site(local.site)?,
        })
    })?;
    body.values = record
        .values
        .iter()
        .map(|value| {
            Ok(Value {
                ty: context.types[value.ty.0 as usize],
                kind: match value.kind {
                    g::ValueKind::Unit => Rvalue::Unit,
                    g::ValueKind::Read(local) => Rvalue::Read(Place::Local(LocalId(local))),
                    g::ValueKind::Constrain(value) => Rvalue::Constrain(ValueId(value)),
                    g::ValueKind::Function(application) => Rvalue::Function(
                        context
                            .linkage
                            .original_application(context.owner, application)?
                            .target,
                    ),
                    g::ValueKind::Call {
                        signature,
                        callee,
                        arguments,
                        ..
                    } => Rvalue::Call {
                        signature: context.types[signature.0 as usize],
                        callee: ValueId(callee),
                        arguments: crate::body::ir::ValueList {
                            start: arguments.start,
                            end: arguments.end,
                        },
                    },
                },
            })
        })
        .collect::<Result<_, &'static str>>()?;
    body.arguments = record.arguments.iter().copied().map(ValueId).collect();
    Ok(())
}
pub(super) struct LocalData {
    pub ty: TypeId,
    pub definition: Option<DefId>,
    pub source: BodySite,
}
pub(super) fn locals(
    record: &[g::Local],
    mut data: impl FnMut(&g::Local) -> Result<LocalData, &'static str>,
) -> Result<Vec<Local>, &'static str> {
    record
        .iter()
        .map(|local| {
            let data = data(local)?;
            Ok(Local {
                scope: scopes::Id(local.scope),
                storage_parent: local.storage_parent.map(LocalId),
                ty: data.ty,
                kind: match local.kind {
                    g::LocalKind::Parameter(index) => LocalKind::Parameter(index),
                    g::LocalKind::Binding => LocalKind::Binding,
                    g::LocalKind::Merge => LocalKind::Merge,
                },
                definition: data.definition,
                mutable: local.mutable,
                source: data.source,
            })
        })
        .collect()
}
fn control(
    record: &g::Original,
    body: &mut Body,
    site: &impl Fn(u32) -> Result<BodySite, &'static str>,
) -> Result<(), &'static str> {
    body.statements = record
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
    body.blocks = blocks(&record.blocks, &mut |index| site(index))?;
    Ok(())
}
pub(super) fn blocks(
    record: &[g::Block],
    site: &mut impl FnMut(u32) -> Result<BodySite, &'static str>,
) -> Result<Vec<Block>, &'static str> {
    record
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
        .collect::<Result<_, &'static str>>()
}
