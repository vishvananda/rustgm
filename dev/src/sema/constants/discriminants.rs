//! Declaration-time enum values are queried independently of provisional field shapes.
use super::{
    Declarations, DefId, Error, NodeKey, Primitive, Queries, Query, Report, Scalar, Scratch, State,
    TypeId, Value, VariantId, Work, arithmetic,
};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy)]
pub(in crate::sema) struct Discriminant {
    pub variant: VariantId,
    pub value: Option<NodeKey>,
    pub tag: Primitive,
    pub previous: Option<VariantId>,
}
#[derive(Clone, Copy)]
struct Entry {
    spec: Discriminant,
    value: Option<u128>,
}
/// Shared across declaration queries; only completed values are cached.
#[derive(Default)]
pub(in crate::sema) struct Pending {
    entries: Vec<Entry>,
    index: HashMap<VariantId, usize>,
    captures: crate::sema::generics::captures::Captures,
    cursor: usize,
    explicit_values: usize,
    used: HashSet<(crate::types::ItemId, u128)>,
}
impl Pending {
    pub(in crate::sema) fn push(&mut self, spec: Discriminant) -> Result<(), &'static str> {
        if self
            .index
            .insert(spec.variant, self.entries.len())
            .is_some()
        {
            return Err("enum discriminant already declared");
        }
        self.explicit_values += usize::from(spec.value.is_some());
        self.entries.push(Entry { spec, value: None });
        Ok(())
    }
    pub(super) fn len(&self) -> usize {
        self.entries.len()
    }
    fn entry(&self, variant: VariantId) -> Result<Entry, &'static str> {
        self.index
            .get(&variant)
            .and_then(|&index| self.entries.get(index))
            .copied()
            .ok_or("missing pending discriminant query")
    }
    fn complete(&mut self, variant: VariantId, value: u128) -> Result<(), &'static str> {
        let &index = self
            .index
            .get(&variant)
            .ok_or("missing discriminant query")?;
        self.entries[index].value = Some(value);
        Ok(())
    }
}

pub(in crate::sema) enum Progress {
    Complete,
    Prepare {
        source: NodeKey,
        dependency: super::suspension::Dependency,
        continuation: Box<super::suspension::Continuation>,
    },
}

pub(in crate::sema) fn advance(
    report: &Report,
    facts: &mut Declarations,
    pending: &mut Pending,
    scratch: &mut Scratch,
    continuation: Option<Box<super::suspension::Continuation>>,
) -> Result<Progress, Error> {
    if pending.entries.is_empty() {
        return Ok(Progress::Complete);
    }
    if pending.explicit_values == 0 {
        // Preserve the linear, query-free completion path for implicit tags.
        let mut previous = 0;
        for entry in &mut pending.entries {
            visit(facts)?;
            let spec = entry.spec;
            let member = facts.members.variants[spec.variant.0 as usize];
            let source = member
                .origin
                .require_diagnostic()
                .map_err(|e| Error::at(super::FileId(0), 0, e))?;
            let error = |message| Declarations::error(report, source, message);
            let value = if let Some(value) = entry.value {
                value
            } else if spec.previous.is_none() {
                0
            } else {
                arithmetic::successor(spec.tag, previous).ok_or_else(|| {
                    error("implicit enum discriminant overflows its representation")
                })?
            };
            entry.value = Some(value);
            facts
                .aggregates
                .set_discriminant(member.owner, member.ordinal, value)
                .map_err(error)?;
            previous = value;
        }
        return Ok(Progress::Complete);
    }
    let count = pending.len();
    let cursor = pending.cursor;
    let mut queries = Queries::with_scratch(report, facts, Some(pending), std::mem::take(scratch));
    let mut resumed = continuation.is_some();
    if let Some(continuation) = continuation {
        queries.resume(*continuation);
    }
    for index in cursor..count {
        visit(queries.typing.facts)?;
        let spec = queries
            .discriminants
            .as_ref()
            .ok_or_else(|| Error::at(super::FileId(0), 0, "missing pending discriminants"))?
            .entries[index]
            .spec;
        let variant = spec.variant;
        queries.value_scope = spec
            .value
            .map(|body| super::instances::ValueScope::Anonymous {
                body,
                ty: TypeId::primitive(spec.tag),
            });
        let query = Query::Variant(variant);
        if !resumed {
            queries.work.push(Work::Enter(query));
        }
        resumed = false;
        if let Some(dependency) = queries.drain_until(None)? {
            let source = queries.typing.facts.members.variants[variant.0 as usize]
                .origin
                .require_diagnostic()
                .map_err(|e| queries.error(query, e))?;
            let (saved, continuation) = queries.suspend();
            *scratch = saved;
            return Ok(Progress::Prepare {
                source,
                dependency,
                continuation,
            });
        }
        let value = queries
            .value(query)?
            .integer()
            .map_err(|e| queries.error(query, e))?;
        let member = queries.typing.facts.members.variants[variant.0 as usize];
        let pending = queries
            .discriminants
            .as_mut()
            .ok_or_else(|| Error::at(super::FileId(0), 0, "missing pending discriminants"))?;
        if !pending.used.insert((member.owner, value)) {
            return Err(queries.error(query, "duplicate enum discriminant"));
        }
        pending.cursor = index + 1;
    }
    *scratch = queries.into_scratch();
    Ok(Progress::Complete)
}

fn visit(facts: &mut Declarations) -> Result<(), Error> {
    facts.constant_work.discriminant_visits = facts
        .constant_work
        .discriminant_visits
        .checked_add(1)
        .filter(|&visits| visits <= 1_000_000)
        .ok_or_else(|| {
            Error::at(
                super::FileId(0),
                0,
                "enum discriminant scheduling budget exceeded",
            )
        })?;
    Ok(())
}

impl Queries<'_> {
    pub(super) fn complete_query(&mut self, query: Query, value: Value) -> Result<(), Error> {
        let value = self.adjusted_reference(query, value)?;
        if let Query::Variant(variant) = query
            && let Some(pending) = &mut self.discriminants
        {
            let result = value.integer().and_then(|bits| {
                pending.complete(variant, bits)?;
                Ok(bits)
            });
            let bits = result.map_err(|e| self.error(query, e))?;
            let member = self.typing.facts.members.variants[variant.0 as usize];
            self.typing
                .facts
                .aggregates
                .set_discriminant(member.owner, member.ordinal, bits)
                .map_err(|e| self.error(query, e))?;
        }
        self.states
            .insert(self.state_key(query), State::Ready(value));
        Ok(())
    }
    pub(super) fn variant_tag(&self, variant: VariantId) -> Result<Primitive, Error> {
        if let Some(pending) = &self.discriminants {
            return pending
                .entry(variant)
                .map(|entry| entry.spec.tag)
                .map_err(|e| self.error(Query::Variant(variant), e));
        }
        let member = self.typing.facts.members.variants[variant.0 as usize];
        self.typing
            .facts
            .aggregates
            .get(member.owner)
            .map(|shape| shape.repr.tag.unwrap_or(Primitive::Isize))
            .ok_or_else(|| self.error(Query::Variant(variant), "missing enum representation"))
    }
    pub(super) fn enter_variant(&mut self, variant: VariantId) -> Result<(), Error> {
        let query = Query::Variant(variant);
        let Some(pending) = &self.discriminants else {
            let member = self.typing.facts.members.variants[variant.0 as usize];
            if !self
                .typing
                .facts
                .aggregates
                .discriminant_ready(member.owner, member.ordinal)
            {
                return Err(self.error(
                    query,
                    "enum discriminant requires completed declaration evaluation",
                ));
            }
            let shape = self
                .typing
                .facts
                .aggregates
                .get(member.owner)
                .ok_or_else(|| self.error(query, "missing completed enum definition"))?;
            let value = self
                .typing
                .facts
                .aggregates
                .variants(shape)
                .get(member.ordinal as usize)
                .ok_or_else(|| self.error(query, "missing completed enum discriminant"))?
                .discriminant;
            return self.complete_query(query, Value::Scalar(Scalar::Integer(value)));
        };
        let entry = pending.entry(variant).map_err(|e| self.error(query, e))?;
        if let Some(value) = entry.value {
            return self.complete_query(query, Value::Scalar(Scalar::Integer(value)));
        }
        let spec = entry.spec;
        if let Some(key) = spec.value {
            let ty = TypeId::primitive(spec.tag);
            let owner = DefId(
                self.typing.facts.members.variants[variant.0 as usize]
                    .owner
                    .0,
            );
            let scope = self.report.resolution.definitions[owner.0 as usize].scope;
            let module = self.report.resolution.scopes[scope.0 as usize].module;
            if let Some(pending) = &mut self.discriminants {
                pending
                    .captures
                    .check(self.report, self.typing.facts, key, module)?;
            }
            self.typing.anonymous_constant(key, ty, module)?;
            self.require_complete_patterns(query)?;
            self.forward(query, Query::Node(key));
            self.prepare(key)?;
        } else if let Some(previous) = spec.previous {
            self.work
                .push(Work::Successor(query, Query::Variant(previous), spec.tag));
            self.work.push(Work::Enter(Query::Variant(previous)));
        } else {
            self.complete_query(query, Value::Scalar(Scalar::Integer(0)))?;
        }
        Ok(())
    }
}
