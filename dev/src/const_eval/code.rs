//! Immutable execution snapshots retain typed CFG operations after proof handoff.
#[cfg(test)]
#[path = "../../tests/invariants/retained_cfg_storage.rs"]
mod storage_tests;
use crate::{
    body::ir::{Block, Body, Local, Owner, Projection, Rvalue, Statement, Value, ValueId},
    resolve::{model::DefId, unit::NodeKey},
    sema::{model::Scalar, values::Constant},
    types::instance::Instance,
};
use std::{collections::HashMap, ops::Index};

#[derive(Clone, Copy)]
pub(crate) enum Inline {
    Closed(NodeKey),
    Instance(crate::sema::inline_constants::InlineInstance),
}

pub(crate) struct Code {
    pub source: NodeKey,
    pub result: crate::types::TypeId,
    pub owner: Owner,
    pub const_context: bool,
    pub locals: Vec<Local>,
    pub values: Vec<Value>,
    pub scalars: Vec<Scalar>,
    pub arguments: Vec<ValueId>,
    pub statements: Vec<Statement>,
    pub projections: Vec<Projection>,
    pub blocks: Vec<Block>,
    pub promotions: HashMap<ValueId, Constant>,
    pub inline: Vec<(ValueId, Inline, crate::types::TypeId)>,
    callables: HashMap<Instance, Instance>,
}

impl Code {
    pub(crate) fn storage_bytes(body: &Body) -> usize {
        [
            std::mem::size_of::<Self>(),
            std::mem::size_of_val(body.locals.as_slice()),
            std::mem::size_of_val(body.values.as_slice()),
            std::mem::size_of_val(body.scalars.as_slice()),
            std::mem::size_of_val(body.arguments.as_slice()),
            std::mem::size_of_val(body.statements.as_slice()),
            std::mem::size_of_val(body.projections.as_slice()),
            std::mem::size_of_val(body.blocks.as_slice()),
            body.promotions.as_ref().map_or(0, |facts| {
                crate::support::retained::new_map::<ValueId, Constant>(facts.entries().count())
            }),
            body.callable_allocation_bytes(),
            body.values
                .iter()
                .filter(|value| {
                    matches!(
                        value.kind,
                        Rvalue::InlineConstant(_) | Rvalue::InlineInstance(_)
                    )
                })
                .count()
                .saturating_mul(size_of::<(ValueId, Inline, crate::types::TypeId)>()),
        ]
        .into_iter()
        .fold(0, usize::saturating_add)
    }
    pub(crate) fn snapshot(body: &Body) -> Self {
        let count = body
            .values
            .iter()
            .filter(|value| {
                matches!(
                    value.kind,
                    Rvalue::InlineConstant(_) | Rvalue::InlineInstance(_)
                )
            })
            .count();
        let mut inline = Vec::with_capacity(count);
        for statement in &body.statements {
            let crate::body::ir::Action::Evaluate(id) = statement.action else {
                continue;
            };
            let Some(value) = body.values.get(id.0 as usize) else {
                continue;
            };
            let selected = match value.kind {
                Rvalue::InlineConstant(source) => Some(Inline::Closed(source)),
                Rvalue::InlineInstance(instance) => Some(Inline::Instance(instance)),
                _ => None,
            };
            if let Some(selected) = selected {
                inline.push((id, selected, value.ty));
            }
        }
        Self {
            source: body.source,
            result: body.result,
            owner: body.owner,
            const_context: body.const_context,
            locals: body.locals.clone(),
            values: body.values.clone(),
            scalars: body.scalars.clone(),
            arguments: body.arguments.clone(),
            statements: body.statements.clone(),
            projections: body.projections.clone(),
            blocks: body.blocks.clone(),
            promotions: body
                .promotions
                .as_ref()
                .map_or_else(HashMap::new, |facts| facts.entries().collect()),
            callables: body.callable_targets(),
            inline,
        }
    }
    pub(crate) fn callable_target(&self, original: Instance) -> Instance {
        self.callables.get(&original).copied().unwrap_or(original)
    }
    pub(crate) fn storage_range(
        &self,
        first: crate::body::ir::LocalId,
        last: crate::body::ir::LocalId,
    ) -> crate::body::ir::StorageRange<'_> {
        crate::body::ir::StorageRange::new(&self.locals, first, last)
    }
}

type AnonymousIndex = HashMap<
    (
        crate::sema::inline_constants::InlineInstance,
        crate::types::TypeId,
    ),
    usize,
>;
#[derive(Default)]
pub(crate) struct Catalog {
    pub bytes: usize,
    bodies: Vec<std::rc::Rc<Code>>,
    pub functions: HashMap<Instance, usize>,
    pub definitions: HashMap<DefId, usize>,
    pub expressions: HashMap<NodeKey, usize>,
    pub anonymous: Option<Box<AnonymousIndex>>,
}
impl Catalog {
    pub(crate) fn initializer(
        &mut self,
        index: usize,
        definition: DefId,
        kind: crate::sema::model::InitializerKind,
    ) -> Result<(), &'static str> {
        let body = self
            .bodies
            .get_mut(index)
            .and_then(std::rc::Rc::get_mut)
            .ok_or("constant code is still in use during initializer registration")?;
        body.owner = Owner::Initializer { definition, kind };
        self.definitions.insert(definition, index);
        Ok(())
    }
    pub(crate) fn insert(&mut self, body: &Body) -> Result<usize, &'static str> {
        let bytes = self
            .bytes
            .checked_add(Code::storage_bytes(body))
            .and_then(|bytes| {
                bytes.checked_add(if matches!(body.owner, Owner::AnonymousConstant) {
                    crate::support::retained::new_map::<NodeKey, usize>(1)
                } else if matches!(body.owner, Owner::AnonymousInstance(_)) {
                    crate::support::retained::new_map::<
                        (
                            crate::sema::inline_constants::InlineInstance,
                            crate::types::TypeId,
                        ),
                        usize,
                    >(1)
                    .saturating_add(if self.anonymous.is_none() {
                        size_of::<AnonymousIndex>()
                    } else {
                        0
                    })
                } else {
                    0
                })
            })
            .filter(|n| *n <= 64 * 1024 * 1024)
            .ok_or("constant execution code exceeds 64 MiB budget")?;
        let index = self.bodies.len();
        if u32::try_from(body.values.len()).is_err() {
            return Err("constant CFG value identity limit exceeded");
        }
        if let Some(instance) = body.instance {
            if self.functions.contains_key(&instance) {
                return Err("duplicate constant function instance");
            }
            self.functions.insert(instance, index);
        } else if let Owner::Function { definition, .. } | Owner::Initializer { definition, .. } =
            body.owner
        {
            if self.definitions.contains_key(&definition) {
                return Err("duplicate constant body definition");
            }
            self.definitions.insert(definition, index);
        } else if matches!(body.owner, Owner::AnonymousConstant) {
            if self.expressions.contains_key(&body.source) {
                return Err("duplicate anonymous constant body");
            }
            self.expressions.insert(body.source, index);
        }
        if let Owner::AnonymousInstance(instance) = body.owner
            && self
                .anonymous
                .get_or_insert_with(Default::default)
                .insert((instance, body.result), index)
                .is_some()
        {
            return Err("duplicate anonymous constant instance body");
        }
        self.bodies.push(std::rc::Rc::new(Code::snapshot(body)));
        self.bytes = bytes;
        Ok(index)
    }
}
impl Index<usize> for Catalog {
    type Output = std::rc::Rc<Code>;
    fn index(&self, index: usize) -> &Self::Output {
        &self.bodies[index]
    }
}
pub(crate) enum CatalogRef<'a> {
    Owned(Catalog),
    Borrowed(&'a Catalog),
}
impl std::ops::Deref for CatalogRef<'_> {
    type Target = Catalog;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Owned(catalog) => catalog,
            Self::Borrowed(catalog) => catalog,
        }
    }
}
