//! Immutable execution snapshots retain typed CFG operations after proof handoff.
#[cfg(test)]
#[path = "../../tests/invariants/retained_cfg_storage.rs"]
mod storage_tests;
use crate::support::hash::HashMap;
use crate::{
    body::ir::{Block, Body, Local, Owner, Projection, Rvalue, Statement, Value, ValueId},
    resolve::{model::DefId, unit::NodeKey},
    sema::{model::Scalar, values::Constant},
    types::instance::Instance,
};
use std::ops::Index;

#[derive(Clone, Copy)]
pub(crate) enum Inline {
    Closed(NodeKey),
    Instance(crate::sema::inline_constants::InlineInstance),
}
#[derive(Clone, Copy)]
pub(crate) enum PreparedOperand {
    Inline {
        destination: ValueId,
        selected: Inline,
        result: crate::types::TypeId,
    },
    Discriminant(crate::sema::members::VariantId),
}

pub(crate) struct Code {
    pub instance: Option<Instance>,
    pub source: NodeKey,
    pub result: crate::types::TypeId,
    pub owner: Owner,
    pub const_context: bool,
    pub source_only: bool,
    pub locals: Vec<Local>,
    pub values: Vec<Value>,
    pub scalars: Vec<Scalar>,
    pub arguments: Vec<ValueId>,
    pub statements: Vec<Statement>,
    pub projections: Vec<Projection>,
    pub blocks: Vec<Block>,
    pub promotions: HashMap<ValueId, Constant>,
    pub operands: Vec<PreparedOperand>,
    callables: HashMap<Instance, Instance>,
}

impl Code {
    pub(crate) fn storage_bytes(body: &Body) -> Result<usize, &'static str> {
        Ok([
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
            Self::operand_count(body)?.saturating_mul(size_of::<PreparedOperand>()),
        ]
        .into_iter()
        .fold(0, usize::saturating_add))
    }
    fn operand_count(body: &Body) -> Result<usize, &'static str> {
        body.statements.iter().try_fold(0, |count, statement| {
            Ok(count + usize::from(Self::operand(body, statement)?.is_some()))
        })
    }
    pub(crate) fn snapshot(body: &Body) -> Result<Self, &'static str> {
        let mut operands = Vec::with_capacity(Self::operand_count(body)?);
        for statement in &body.statements {
            if let Some(operand) = Self::operand(body, statement)? {
                operands.push(operand);
            }
        }
        Ok(Self {
            instance: body.instance,
            source: body.source,
            result: body.result,
            owner: body.owner,
            const_context: body.const_context,
            source_only: false,
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
                .map_or_else(HashMap::default, |facts| facts.entries().collect()),
            callables: body.callable_targets(),
            operands,
        })
    }
    fn operand(
        body: &Body,
        statement: &Statement,
    ) -> Result<Option<PreparedOperand>, &'static str> {
        let crate::body::ir::Action::Evaluate(id) = statement.action else {
            return Ok(None);
        };
        let value = body
            .values
            .get(id.0 as usize)
            .ok_or("missing prepared constant operand")?;
        let selected = match value.kind {
            Rvalue::InlineConstant(source) => Inline::Closed(source),
            Rvalue::InlineInstance(instance) => Inline::Instance(instance),
            Rvalue::Cast {
                kind,
                operand,
                direct_variant: true,
            } => {
                if kind != crate::sema::model::Cast::EnumToInteger {
                    return Err("direct variant formation requires an enum integer cast");
                }
                // Constant operand formation uses the retained immediate constructor.
                // Reads and calls keep reached semantics; no source propagation occurs.
                let Rvalue::Aggregate {
                    fields,
                    variant: Some(variant),
                } = body
                    .values
                    .get(operand.0 as usize)
                    .ok_or("missing direct variant operand")?
                    .kind
                else {
                    return Err("direct variant formation requires a constructor operand");
                };
                if fields.start != fields.end {
                    return Err("direct variant formation requires an empty constructor");
                }
                return Ok(Some(PreparedOperand::Discriminant(variant)));
            }
            _ => return Ok(None),
        };
        Ok(Some(PreparedOperand::Inline {
            destination: id,
            selected,
            result: value.ty,
        }))
    }
    fn inline_results_match(&self, results: &[(ValueId, crate::types::TypeId)]) -> bool {
        self.operands
            .iter()
            .filter_map(|operand| {
                if let PreparedOperand::Inline {
                    destination,
                    selected: Inline::Instance(_),
                    result,
                } = *operand
                {
                    Some((destination, result))
                } else {
                    None
                }
            })
            .eq(results.iter().copied())
    }
    fn transport_inline_results(
        &mut self,
        source_only: bool,
        results: &[(ValueId, crate::types::TypeId)],
    ) -> Result<(), &'static str> {
        let mut results = results.iter();
        if source_only {
            for operand in &mut self.operands {
                if let PreparedOperand::Inline {
                    destination,
                    selected: Inline::Instance(_),
                    result,
                } = operand
                {
                    let &(id, ty) = results
                        .next()
                        .ok_or("missing transported inline request result")?;
                    if id != *destination {
                        return Err("transported inline request order changed");
                    }
                    *result = ty;
                }
            }
        }
        if results.next().is_some() {
            return Err("foreign transported inline request result");
        }
        Ok(())
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
type AssociatedIndex = HashMap<(crate::sema::values::Instance, crate::types::TypeId), usize>;
type SourceIndex =
    HashMap<crate::sema::values::execution::SourceComputation, (usize, crate::types::TypeId)>;
#[derive(Default)]
pub(crate) struct Catalog {
    pub bytes: usize,
    bodies: Vec<std::rc::Rc<Code>>,
    pub functions: HashMap<Instance, usize>,
    pub definitions: HashMap<DefId, usize>,
    pub expressions: HashMap<NodeKey, usize>,
    pub anonymous: Option<Box<AnonymousIndex>>,
    pub associated: Option<Box<AssociatedIndex>>,
    pub sources: Option<Box<SourceIndex>>,
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
        self.insert_body(body, false, None, &[])
    }
    pub(crate) fn source_initializer(
        &mut self,
        facts: &mut crate::sema::model::Declarations,
        body: &Body,
        source: (crate::sema::values::Instance, NodeKey, crate::types::TypeId),
        witness: crate::sema::values::execution::SourceComputation,
        results: &[(ValueId, crate::types::TypeId)],
    ) -> Result<usize, &'static str> {
        let selected = witness.resolve(facts, source.2)?;
        if selected.selected != source.0
            || selected.original != source.1
            || !matches!(body.owner, Owner::InitializerInstance { instance, .. } if instance == source.0)
            || body.source != source.1
        {
            return Err("source computation selects another checked initializer body");
        }
        if let Some(&(index, actual)) = self.sources.as_deref().and_then(|map| map.get(&witness)) {
            let previous = self.bodies.get(index).ok_or("source code index is stale")?;
            if !previous.source_only
                || previous.source != body.source
                || !matches!(previous.owner, Owner::InitializerInstance {instance,..} if instance==source.0)
                || previous.result != source.2
                || actual != body.result
                || !previous.inline_results_match(results)
            {
                return Err("source initializer code identity changed");
            }
            return Ok(index);
        }
        let bytes = self
            .bytes
            .saturating_add(crate::support::retained::new_map::<
                crate::sema::values::execution::SourceComputation,
                (usize, crate::types::TypeId),
            >(1))
            .saturating_add(if self.sources.is_none() {
                size_of::<SourceIndex>()
            } else {
                0
            });
        if bytes > 64 * 1024 * 1024 {
            return Err("source code index storage budget exceeded");
        }
        let prior = self.bytes;
        self.bytes = bytes;
        let index = match self.insert_body(body, true, Some(source.2), results) {
            Ok(index) => index,
            Err(error) => {
                self.bytes = prior;
                return Err(error);
            }
        };
        self.sources
            .get_or_insert_with(Default::default)
            .insert(witness, (index, body.result));
        Ok(index)
    }
    pub(crate) fn anonymous_result(
        &mut self,
        body: &Body,
        requested: crate::types::TypeId,
        results: &[(ValueId, crate::types::TypeId)],
    ) -> Result<usize, &'static str> {
        let Owner::AnonymousInstance(instance) = body.owner else {
            return Err("anonymous publication requires its exact instance owner");
        };
        let source_only = matches!(
            instance.enclosing(),
            crate::sema::inline_constants::Context::SourceInitializer(_)
        );
        self.insert_body(body, source_only, Some(requested), results)
    }
    fn insert_body(
        &mut self,
        body: &Body,
        source_only: bool,
        requested: Option<crate::types::TypeId>,
        results: &[(ValueId, crate::types::TypeId)],
    ) -> Result<usize, &'static str> {
        if !source_only
            && matches!(body.owner, Owner::AnonymousInstance(instance)
            if matches!(instance.enclosing(), crate::sema::inline_constants::Context::SourceInitializer(_)))
        {
            return Err("source anonymous code requires checked source publication");
        }
        let published = requested.unwrap_or(body.result);
        let bytes = self
            .bytes
            .checked_add(Code::storage_bytes(body)?)
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
                } else if !source_only && matches!(body.owner, Owner::InitializerInstance { .. }) {
                    crate::support::retained::new_map::<
                        (crate::sema::values::Instance, crate::types::TypeId),
                        usize,
                    >(1)
                    .saturating_add(if self.associated.is_none() {
                        size_of::<AssociatedIndex>()
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
        let mut code = Code::snapshot(body)?;
        code.transport_inline_results(source_only, results)?;
        code.source_only = source_only;
        code.result = published;
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
                .insert((instance, published), index)
                .is_some()
        {
            return Err("duplicate anonymous constant instance body");
        }
        if let Owner::InitializerInstance { instance, .. } = body.owner
            && !source_only
            && self
                .associated
                .get_or_insert_with(Default::default)
                .insert((instance, body.result), index)
                .is_some()
        {
            return Err("duplicate selected initializer execution body");
        }
        self.bodies.push(std::rc::Rc::new(code));
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
