//! One exact caller namespace transports source locals before actual arguments enter.
use super::{CallerInstance, Declarations, NodeKey, catalog::CallerKey};
use crate::support::hash::HashMap;
use crate::{sema::traits::obligations::Reservation, types::lifetime_proof::OwnedTransport};
use std::ops::Range;

struct Entry {
    transport: Option<OwnedTransport>,
    bytes: usize,
}
pub(super) struct Namespaces {
    entries: HashMap<CallerKey, Entry>,
    pub(super) observations:
        HashMap<crate::sema::inline_constants::Context, super::observed_arguments::Observed>,
    pub(super) bytes: usize,
}
impl Namespaces {
    pub(super) fn new() -> Self {
        Self {
            entries: HashMap::default(),
            observations: HashMap::default(),
            bytes: size_of::<Self>(),
        }
    }
}

/// The callback only renames retained identities. Its result carries no permission.
/// The exact cached folder is restored on errors as well as successful returns.
pub(crate) fn with_namespace<T>(
    facts: &mut Declarations,
    source: NodeKey,
    enclosing: CallerInstance,
    universals: u32,
    owned: &[Range<u32>],
    scratch: usize,
    operation: impl FnOnce(&mut Declarations, &mut OwnedTransport) -> Result<T, &'static str>,
) -> Result<T, &'static str> {
    validate_context(facts, source, enclosing)?;
    let Some(computations) = facts.source_computations.as_deref() else {
        return Err("source caller has no completed namespace owner");
    };
    let (mut key, publication) = computations.0.original(source)?;
    publication.validate(facts, source)?;
    if matches!(enclosing, CallerInstance::Original) || key.universals != universals {
        return Err("source namespace does not match its original body binder");
    }
    key.enclosing = enclosing;
    let mut transport = take(facts, key, owned, scratch)?;
    let before = transport.steps.saturating_add(transport.fold.steps);
    let result = (|| {
        transport.tighten_bytes(room(facts, key)?.saturating_sub(scratch))?;
        operation(facts, &mut transport)
    })();
    let (bytes, visits) = transport.retained_bytes();
    facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(
        transport
            .steps
            .saturating_add(transport.fold.steps)
            .saturating_sub(before)
            .saturating_add(visits),
    );
    let restored = restore(facts, key, transport, bytes);
    restored?;
    result
}
fn take(
    facts: &mut Declarations,
    key: CallerKey,
    owned: &[Range<u32>],
    scratch: usize,
) -> Result<OwnedTransport, &'static str> {
    let existing = facts
        .source_computations
        .as_deref_mut()
        .and_then(|c| c.0.namespaces.as_deref_mut())
        .and_then(|n| n.entries.get_mut(&key));
    if let Some(entry) = existing {
        let transport = entry
            .transport
            .as_ref()
            .ok_or("source caller namespace is already borrowed")?;
        if !transport.matches(owned) {
            return Err("source caller owned lifetimes changed after registration");
        }
        return entry
            .transport
            .take()
            .ok_or("missing source caller namespace");
    }
    let reserved = facts
        .body_regions
        .frontier(key.source)
        .map_err(|_| "source namespace identity space exhausted")?;
    super::observed_arguments::prepare(facts, key.enclosing, scratch)?;
    // Renamed locals join the source body's numbering unit after its current
    // identities and after every observed actual, which another unit may have
    // numbered; the reservation below covers any identities skipped between.
    let first = super::observed_arguments::regions(facts, key.enclosing)?
        .iter()
        .filter(|region| region.bound_parts().is_none())
        .map(|region| region.0.saturating_add(1))
        .fold(reserved, u32::max);
    // `owned` comes from the exact original Contract, whose local proof IDs
    // may collide numerically with another body's diagnostic arena. Rename
    // those locals before inserting external actuals; never interpret the
    // original proof interval as global diagnostic ownership.
    let catalog = &facts
        .source_computations
        .as_deref()
        .ok_or("missing source namespace catalog")?
        .0;
    let metadata = crate::support::retained::new_map::<CallerKey, Entry>(1).saturating_add(
        if catalog.namespaces.is_none() {
            size_of::<Namespaces>()
        } else {
            0
        },
    );
    let bytes = catalog
        .namespace_available()
        .saturating_sub(metadata)
        .saturating_sub(scratch);
    let mut transport = OwnedTransport::new_budgeted(owned, &[], key.universals, first, bytes)?;
    transport.check_actuals(super::observed_arguments::regions(facts, key.enclosing)?)?;
    let (bytes, visits) = transport.retained_bytes();
    facts.body_work.canonical_steps = facts
        .body_work
        .canonical_steps
        .saturating_add(transport.steps)
        .saturating_add(visits);
    let catalog = &mut facts
        .source_computations
        .as_deref_mut()
        .ok_or("missing source namespace catalog")?
        .0;
    let current = catalog
        .namespaces
        .as_ref()
        .map_or(size_of::<Namespaces>(), |n| n.bytes);
    let planned = current
        .saturating_add(bytes)
        .saturating_add(crate::support::retained::new_map::<CallerKey, Entry>(1));
    catalog.namespace_budget(planned)?;
    if catalog
        .namespaces
        .as_ref()
        .is_some_and(|n| n.entries.len() >= 1_000_000)
    {
        return Err("source caller namespace count budget exceeded");
    }
    // These are diagnostic/ownership slots, never executable proof publication.
    let interval = transport.interval_range();
    let allocated = if interval.is_empty() {
        reserved..reserved
    } else {
        reserved..interval.end
    };
    Reservation::default().reserve(facts, key.source, key.source, allocated)?;
    let namespaces = facts
        .source_computations
        .as_deref_mut()
        .ok_or("missing source namespace catalog")?
        .0
        .namespaces
        .get_or_insert_with(|| Box::new(Namespaces::new()));
    namespaces.entries.insert(
        key,
        Entry {
            transport: None,
            bytes,
        },
    );
    namespaces.bytes = planned;
    Ok(transport)
}
fn restore(
    facts: &mut Declarations,
    key: CallerKey,
    transport: OwnedTransport,
    bytes: usize,
) -> Result<(), &'static str> {
    let fixed = transport.validate_fixed();
    let catalog = &mut facts
        .source_computations
        .as_deref_mut()
        .ok_or("missing source namespace catalog")?
        .0;
    let namespaces = catalog
        .namespaces
        .as_deref_mut()
        .ok_or("missing source namespace owner")?;
    let entry = namespaces
        .entries
        .get_mut(&key)
        .ok_or("missing source namespace entry")?;
    if entry.transport.is_some() {
        return Err("source namespace was replaced while borrowed");
    }
    namespaces.bytes = namespaces
        .bytes
        .saturating_sub(entry.bytes)
        .saturating_add(bytes);
    entry.bytes = bytes;
    entry.transport = Some(transport);
    let bytes = namespaces.bytes;
    catalog.namespace_budget(bytes)?;
    facts.constant_work.source_context_bytes = catalog.retained_bytes();
    fixed
}

pub(super) fn contains(facts: &Declarations, mut key: CallerKey) -> bool {
    let enclosing = key.enclosing;
    let Some(catalog) = facts.source_computations.as_deref().map(|c| &c.0) else {
        return false;
    };
    let Ok((original, _)) = catalog.original(key.source) else {
        return false;
    };
    key.universals = original.universals;
    key.enclosing = enclosing;
    catalog.namespaces.as_ref().is_some_and(|n| {
        n.entries.get(&key).is_some_and(|e| {
            e.transport
                .as_ref()
                .is_some_and(|t| t.validate_fixed().is_ok())
        })
    })
}

pub(crate) fn with_registered_namespace<T>(
    facts: &mut Declarations,
    source: NodeKey,
    enclosing: CallerInstance,
    operation: impl FnOnce(&mut Declarations, &mut OwnedTransport) -> Result<T, &'static str>,
) -> Result<T, &'static str> {
    validate_context(facts, source, enclosing)?;
    let catalog = &facts
        .source_computations
        .as_deref()
        .ok_or("source namespace catalog is missing")?
        .0;
    let (mut key, _) = catalog.original(source)?;
    key.enclosing = enclosing;
    let entry = facts
        .source_computations
        .as_deref_mut()
        .and_then(|c| c.0.namespaces.as_deref_mut())
        .and_then(|n| n.entries.get_mut(&key))
        .ok_or("source body has no registered namespace")?;
    let mut transport = entry
        .transport
        .take()
        .ok_or("source namespace is already borrowed")?;
    let before = transport.steps.saturating_add(transport.fold.steps);
    let result = (|| {
        transport.tighten_bytes(room(facts, key)?)?;
        operation(facts, &mut transport)
    })();
    let (bytes, visits) = transport.retained_bytes();
    facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(
        transport
            .steps
            .saturating_add(transport.fold.steps)
            .saturating_sub(before)
            .saturating_add(visits),
    );
    restore(facts, key, transport, bytes)?;
    result
}
pub(crate) fn available(facts: &Declarations) -> usize {
    facts
        .source_computations
        .as_deref()
        .map_or(0, |c| c.0.namespace_available())
}

pub(crate) fn published(facts: &Declarations, source: NodeKey) -> bool {
    facts
        .source_computations
        .as_deref()
        .is_some_and(|c| c.0.original(source).is_ok())
}

fn validate_context(
    facts: &Declarations,
    source: NodeKey,
    context: CallerInstance,
) -> Result<(), &'static str> {
    use crate::types::Kind;
    let valid = match context {
        CallerInstance::Callable(instance) => match facts.types.kind(instance.ty()) {
            Some(Kind::FunctionItem { item, .. }) => {
                facts
                    .function_indices
                    .get(&crate::resolve::model::DefId(item.0))
                    .and_then(|&n| facts.functions.get(n))
                    .and_then(|f| f.body)
                    == Some(source)
            }
            Some(Kind::Closure { identity, .. }) => facts
                .closures
                .identity(identity)
                .is_some_and(|c| c.semantic_source() == source),
            _ => false,
        },
        CallerInstance::Initializer(instance) => {
            instance.selected_arguments(facts)?;
            facts
                .initializer_index(instance.definition)
                .and_then(|n| facts.initializers.get(n))
                .is_some_and(|i| i.semantic_source() == source)
        }
        CallerInstance::Inline(instance) => {
            instance.source() == source
                && crate::sema::inline_constants::InlineInstance::select(
                    facts,
                    source,
                    instance.enclosing(),
                )? == instance
        }
        CallerInstance::Original => false,
    };
    if !valid {
        return Err("source caller namespace belongs to another semantic owner");
    }
    Ok(())
}

fn room(facts: &Declarations, key: CallerKey) -> Result<usize, &'static str> {
    let catalog = &facts
        .source_computations
        .as_deref()
        .ok_or("missing source namespace catalog")?
        .0;
    let previous = catalog
        .namespaces
        .as_ref()
        .and_then(|n| n.entries.get(&key))
        .ok_or("missing source namespace entry")?
        .bytes;
    Ok(catalog.namespace_available().saturating_add(previous))
}

#[cfg(test)]
#[path = "../../../../tests/invariants/parked_callers.rs"]
mod tests;

/// The body owner supplies only its completed, exactly covered normalization delta.
/// Registration reserves diagnostics; its proof remains in the caller Contract.
pub(crate) fn reserve_normalization(
    facts: &mut Declarations,
    source: NodeKey,
    enclosing: CallerInstance,
    allocated: Range<u32>,
) -> Result<(), &'static str> {
    with_registered_namespace(facts, source, enclosing, |facts, _| {
        Reservation::default().reserve(facts, source, source, allocated)
    })
}
