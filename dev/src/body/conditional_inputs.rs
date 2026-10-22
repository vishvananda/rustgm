//! Exact original ports carry transport data, never paid-parent authority.
use super::{
    ir::{Body, BodyKey, Owner, Rvalue, ValueId},
    regions::{Contract, Obligation, OutputLink},
};
use crate::support::hash::{HashMap, HashSet};
use crate::{
    resolve::unit::NodeKey,
    sema::{
        inline_constants::InlineInstance,
        model::Declarations,
        values::execution::{DeclarationEpoch, declaration_epoch},
    },
    types::{RegionId, TypeId, instance::Instance, outlives::Pair},
};
use std::sync::Arc;
const LIMIT: usize = 1_000_000;
const BYTES: usize = 64 * 1024 * 1024;

pub(crate) struct Child {
    pub parent: NodeKey,
    pub link: Arc<OutputLink>,
    pub requirements: Vec<Obligation>,
    pub conditions: Option<Arc<[Pair]>>,
}
#[derive(Default)]
pub(crate) struct Sources {
    children: HashMap<NodeKey, Child>,
    parents: HashMap<NodeKey, Vec<NodeKey>>,
    bytes: usize,
    pub steps: usize,
}
pub(crate) struct Port {
    pub operand: ValueId,
    ordinal: usize,
    pub instance: InlineInstance,
    pub result: TypeId,
    pub link: Arc<OutputLink>,
    pub conditions: Option<Arc<[Pair]>>,
    pub slots: Vec<(u16, RegionId)>,
}
pub(crate) struct Pending {
    epoch: DeclarationEpoch,
    key: BodyKey,
    source: NodeKey,
    // One owner for all exact child ports; identical numeric regions elsewhere
    // cannot join this immutable parent interface.
    unit: NodeKey,
    instance: Option<Instance>,
    ports: Vec<Port>,
    index: HashMap<NodeKey, usize>,
}
impl Port {
    pub fn ordinal(&self) -> usize {
        self.ordinal
    }
}
impl Child {
    fn collection_bytes(&self, ports: usize, obligations: usize, seen: usize) -> usize {
        ports
            .saturating_add(1)
            .saturating_mul(4 * size_of::<Port>())
            .saturating_add(
                obligations
                    .saturating_add(self.requirements.len())
                    .saturating_mul(4 * size_of::<Obligation>()),
            )
            .saturating_add(crate::support::retained::new_map::<NodeKey, ()>(
                seen.saturating_add(1),
            ))
    }
    fn validate_port(
        &self,
        facts: &Declarations,
        body: &Body,
        instance: InlineInstance,
        context: (Instance, NodeKey),
        work: &mut usize,
    ) -> Result<(), &'static str> {
        let (enclosing, unit) = context;
        let row = facts
            .inline_constants
            .as_deref()
            .and_then(|r| r.get(instance.source()))
            .ok_or("conditional inline source is missing")?;
        let actual = InlineInstance::select(facts, instance.source(), instance.enclosing())?;
        let link = facts
            .nested_outputs
            .as_deref()
            .and_then(|r| r.get(&instance.source()))
            .ok_or("conditional child lost its source link")?;
        if actual != instance
            || instance.enclosing() != crate::sema::inline_constants::Context::Callable(enclosing)
            || row.parent_body != body.source
            || row.context_owner() != body.source
            || owner_unit(facts, instance.source(), work) != unit
            || !Arc::ptr_eq(link, &self.link)
            || self.link.formation.is_some()
        {
            return Err("conditional inline port belongs to another exact parent");
        }
        Ok(())
    }
}
/// Resolve only the exact owner's indexed unit; never interpret a log offset
/// or a numeric free region as an ownership identity.
fn owner_unit(facts: &Declarations, source: NodeKey, work: &mut usize) -> NodeKey {
    *work = work.saturating_add(1);
    facts.body_regions.unit_of(source)
}
fn parent_context(
    facts: &Declarations,
    body: &Body,
    input: &Contract,
) -> Result<Instance, &'static str> {
    let definition = match body.owner {
        Owner::Function { definition, .. } => definition,
        Owner::Closure { identity, .. } => facts
            .closures
            .identity(identity)
            .and_then(|c| c.owner)
            .ok_or("conditional closure has no original owner")?,
        _ => return Err("conditional parent owner is not implemented"),
    };
    if body.instance.is_some()
        || facts.generics.has_parameters(definition)
        || input.key != body.key()
        || input.source != body.source
    {
        return Err("conditional parent requires an exact lifetime-only original body");
    }
    crate::body::instantiate::original_inline_context(facts, body)?
        .ok_or("conditional parent has no exact original callable identity")
}
impl Sources {
    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }
    pub fn insert(
        &mut self,
        source: NodeKey,
        parent: NodeKey,
        link: &Arc<OutputLink>,
        requirements: &[Obligation],
        conditions: Option<&Arc<[Pair]>>,
    ) -> Result<(), &'static str> {
        let bytes = self
            .bytes
            .saturating_add(size_of::<Child>())
            .saturating_add(requirements.len().saturating_mul(size_of::<Obligation>()))
            .saturating_add(crate::support::retained::new_map::<NodeKey, Child>(1))
            .saturating_add(crate::support::retained::new_map::<NodeKey, Vec<NodeKey>>(
                1,
            ))
            .saturating_add(4 * size_of::<NodeKey>());
        if self.children.len() >= LIMIT || bytes > BYTES {
            return Err("conditional original ports exceed storage budget");
        }
        if link.source != source || link.parent != parent || self.children.contains_key(&source) {
            return Err("conditional original port changed its source or parent");
        }
        self.steps = self.steps.saturating_add(1 + requirements.len());
        self.parents.entry(parent).or_default().push(source);
        self.children.insert(
            source,
            Child {
                parent,
                link: Arc::clone(link),
                requirements: requirements.to_vec(),
                conditions: conditions.cloned(),
            },
        );
        self.bytes = bytes;
        Ok(())
    }
    pub fn prepare(
        &mut self,
        facts: &Declarations,
        body: &Body,
        input: &mut Contract,
    ) -> Result<Option<Pending>, &'static str> {
        if !self.parents.contains_key(&body.source) {
            return Ok(None);
        }
        let enclosing = parent_context(facts, body, input)?;
        let unit = owner_unit(facts, body.source, &mut self.steps);
        let mut ports = Vec::new();
        let mut seen = HashSet::default();
        let mut obligations = Vec::new();
        for (index, value) in body.values.iter().enumerate() {
            self.steps = self.steps.saturating_add(1);
            if self.steps > LIMIT {
                return Err("conditional parent port work budget exhausted");
            }
            let Rvalue::InlineInstance(instance) = value.kind else {
                continue;
            };
            let Some(child) = self.children.get(&instance.source()) else {
                continue;
            };
            if child.parent != body.source {
                continue;
            }
            child.validate_port(facts, body, instance, (enclosing, unit), &mut self.steps)?;
            if seen.contains(&instance.source()) {
                return Err("conditional inline source repeats its operand");
            }
            let storage = child.collection_bytes(ports.len(), obligations.len(), seen.len());
            let slots = child
                .link
                .bridges
                .len()
                .saturating_mul(size_of::<(u16, RegionId)>());
            self.steps = self
                .steps
                .saturating_add(child.link.bridges.len())
                .saturating_add(child.requirements.len());
            self.bytes = self
                .bytes
                .saturating_add(slots)
                .saturating_add(4 * size_of::<Port>())
                .saturating_add(4 * child.requirements.len() * size_of::<Obligation>());
            if self.bytes.saturating_add(storage) > BYTES {
                return Err("conditional parent port storage budget exceeded");
            }
            seen.insert(instance.source());
            ports.push(Port {
                ordinal: ports.len(),
                operand: ValueId(u32::try_from(index).map_err(|_| "too many parent operands")?),
                instance,
                result: value.ty,
                link: Arc::clone(&child.link),
                conditions: child.conditions.clone(),
                slots: child
                    .link
                    .bridges
                    .iter()
                    .map(|b| (b.slot, b.parent))
                    .collect(),
            });
            obligations.extend_from_slice(&child.requirements);
        }
        if ports.is_empty() {
            return Ok(None);
        }
        let index_bytes = crate::support::retained::new_map::<NodeKey, usize>(ports.len());
        if self.bytes.saturating_add(index_bytes) > BYTES {
            return Err("conditional port index exceeds storage budget");
        }
        self.bytes = self.bytes.saturating_add(index_bytes);
        self.steps = self.steps.saturating_add(ports.len());
        if self.steps > LIMIT {
            return Err("conditional port index work budget exhausted");
        }
        let index = ports
            .iter()
            .enumerate()
            .map(|(i, port)| (port.instance.source(), i))
            .collect();
        input.obligations.extend(obligations);
        Ok(Some(Pending {
            epoch: declaration_epoch(facts)?,
            key: body.key(),
            source: body.source,
            unit,
            instance: body.instance,
            ports,
            index,
        }))
    }
}
impl Pending {
    pub fn ports(&self) -> &[Port] {
        &self.ports
    }
    pub fn has_source(&self, source: NodeKey) -> bool {
        self.index.contains_key(&source)
    }
    pub fn validate_context(
        &self,
        facts: &Declarations,
        body: &Body,
        input: &Contract,
        work: &mut usize,
    ) -> Result<(), &'static str> {
        if declaration_epoch(facts)? != self.epoch
            || body.key() != self.key
            || body.source != self.source
            || body.instance != self.instance
            || input.key != self.key
            || input.source != self.source
            || owner_unit(facts, body.source, work) != self.unit
        {
            return Err("conditional request belongs to another actual paid owner");
        }
        Ok(())
    }
    pub fn request(
        &self,
        facts: &Declarations,
        body: &Body,
        input: &Contract,
        instance: InlineInstance,
        result: TypeId,
        work: &mut usize,
    ) -> Result<&Port, &'static str> {
        self.validate_context(facts, body, input, work)?;
        let enclosing = crate::body::instantiate::original_inline_context(facts, body)?
            .ok_or("conditional request lost its original callable context")?;
        if instance.enclosing() != crate::sema::inline_constants::Context::Callable(enclosing) {
            return Err("conditional request changed its complete original parent instance");
        }
        let port = self
            .index
            .get(&instance.source())
            .and_then(|&i| self.ports.get(i))
            .ok_or("conditional request has no exact paid operand")?;
        if port.instance != instance
            || port.result != result
            || owner_unit(facts, instance.source(), work) != self.unit
            || InlineInstance::select(facts, instance.source(), instance.enclosing())? != instance
            || !facts
                .nested_outputs
                .as_deref()
                .and_then(|r| r.get(&instance.source()))
                .is_some_and(|link| Arc::ptr_eq(link, &port.link))
        {
            return Err("conditional request has stale source identity");
        }
        Ok(port)
    }
    pub fn validate(
        &self,
        facts: &Declarations,
        body: &Body,
        input: &Contract,
        work: &mut usize,
    ) -> Result<(), &'static str> {
        self.validate_context(facts, body, input, work)?;
        *work = work.saturating_add(self.ports.len());
        self.validate_body(body, input)?;
        for port in &self.ports {
            if owner_unit(facts, port.instance.source(), work) != self.unit
                || InlineInstance::select(facts, port.instance.source(), port.instance.enclosing())?
                    != port.instance
                || !facts
                    .nested_outputs
                    .as_deref()
                    .and_then(|r| r.get(&port.instance.source()))
                    .is_some_and(|link| Arc::ptr_eq(link, &port.link))
            {
                return Err("conditional parent port has stale source identity");
            }
        }
        Ok(())
    }
    pub fn validate_body(&self, body: &Body, input: &Contract) -> Result<(), &'static str> {
        if body.key() != self.key
            || body.source != self.source
            || body.instance != self.instance
            || input.key != self.key
            || input.source != self.source
        {
            return Err("conditional parent proof belongs to another body");
        }
        for port in &self.ports {
            if !body.values.get(port.operand.0 as usize).is_some_and(|value|
                matches!(value.kind, Rvalue::InlineInstance(instance) if instance == port.instance) && value.ty == port.result)
                || port.link.parent != self.source || port.link.source != port.instance.source() {
                return Err("conditional parent changed its exact operand or result");
            }
        }
        Ok(())
    }
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            .saturating_add(crate::support::retained::map(&self.index))
            .saturating_add(crate::support::retained::vector(&self.ports))
            .saturating_add(
                self.ports
                    .iter()
                    .map(|p| crate::support::retained::vector(&p.slots))
                    .fold(0_usize, usize::saturating_add),
            )
    }
}

#[cfg(test)]
#[path = "../../tests/invariants/conditional_ports.rs"]
mod tests;
