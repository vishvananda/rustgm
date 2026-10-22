//! Discover named backing through completed values before publishing storage roots.
mod values;
use super::{Declarations, DefId, Error, Report, error, records};
use crate::{
    sema::{
        foreign,
        model::InitializerKind,
        values::{Constant, LocationId, Origin, Reader, View},
    },
    support::{
        crates::{CrateIdentity, ExternalDef, ExternalKind, Symbol},
        linkage::{Linkage, Naming},
    },
    syntax::item::Safety,
    types::TypeId,
};
use std::collections::{HashMap, HashSet};

pub(super) struct Entry {
    pub definition: DefId,
    pub external: ExternalDef,
    pub ty: TypeId,
    pub symbol: Symbol,
    pub public: bool,
    pub mutable: bool,
    pub origin: OriginValue,
}
pub(super) enum OriginValue {
    Defined(Constant),
    Foreign { unsafe_: bool },
}
#[derive(Default)]
pub(super) struct Prepared {
    pub entries: Vec<Entry>,
}

pub(super) fn prepare(
    report: &Report,
    facts: &mut Declarations,
    identity: &CrateIdentity,
    public: &[DefId],
) -> Result<Prepared, Error> {
    let public_set: HashSet<_> = public.iter().copied().collect();
    let mut definitions = public.to_vec();
    let mut seen = HashSet::new();
    let mut values = HashSet::new();
    let mut locations = HashMap::new();
    let mut prepared = Prepared::default();
    let mut steps = 0usize;
    while let Some(definition) = definitions.pop() {
        if !seen.insert(definition) {
            continue;
        }
        if seen.len() > 100_000 {
            return Err(error("metadata static discovery exceeds profile budget"));
        }
        let entry = entry(
            report,
            facts,
            identity,
            definition,
            public_set.contains(&definition),
        )?;
        if let OriginValue::Defined(value) = entry.origin {
            let reader = Reader::new(facts);
            let mut work = vec![value];
            while let Some(value) = work.pop() {
                if !values.insert(value) {
                    continue;
                }
                steps = steps.saturating_add(1);
                if steps > 1_000_000 {
                    return Err(error(
                        "metadata stored value discovery exceeds profile budget",
                    ));
                }
                match reader.view(value).map_err(error)? {
                    View::Product { length, .. } => {
                        if length > 1_000_000 - steps as u64 {
                            return Err(error("metadata product exceeds profile budget"));
                        }
                        steps += usize::try_from(length)
                            .map_err(|_| error("metadata product length exceeds host index"))?;
                        for index in 0..length {
                            work.push(reader.field(value, index).map_err(error)?);
                        }
                    }
                    View::Reference(location) => {
                        definitions.push(static_root(
                            reader,
                            location,
                            &mut locations,
                            &mut steps,
                        )?);
                    }
                    View::Scalar(_) => {}
                    _ => {
                        return Err(error(
                            "static initializer representation is outside the metadata profile",
                        ));
                    }
                }
            }
        }
        prepared.entries.push(entry);
    }
    prepared
        .entries
        .sort_unstable_by(|left, right| left.external.cmp(&right.external));
    Ok(prepared)
}

fn entry(
    report: &Report,
    facts: &mut Declarations,
    identity: &CrateIdentity,
    definition: DefId,
    public: bool,
) -> Result<Entry, Error> {
    let ty = facts.definitions[definition.0 as usize]
        .ok_or_else(|| error("missing exported static type"))?;
    let previous = facts
        .foreign
        .as_deref()
        .and_then(|registry| registry.global(definition));
    let imported = previous.is_some_and(|global| global.linkage == Linkage::Import);
    let mutable = if let Some(global) = previous {
        global.mutable
    } else {
        let index = facts
            .initializer_index(definition)
            .ok_or_else(|| error("missing exported static initializer"))?;
        let InitializerKind::Static { mutable } = facts.initializers[index].kind else {
            return Err(error("metadata storage root is not a static"));
        };
        mutable
    };
    let origin = if imported {
        OriginValue::Foreign {
            unsafe_: previous.is_some_and(|global| global.safety != Safety::Safe),
        }
    } else {
        OriginValue::Defined(
            Reader::new(facts)
                .definition(definition)
                .ok_or_else(|| error("metadata static initializer is not complete"))?,
        )
    };
    let external = records::path(report, identity, definition, ExternalKind::Static)?;
    let registry = facts.foreign.get_or_insert_default();
    let naming = previous.map_or(Naming::Rust, |global| global.naming);
    let symbol = match naming {
        Naming::Rust => Symbol::Rust,
        Naming::Exact(name) => Symbol::Exact(
            registry
                .names()
                .get(name)
                .ok_or_else(|| error("missing static export symbol"))?
                .into(),
        ),
    };
    if !imported {
        registry
            .insert_global(
                definition,
                foreign::Global {
                    ty,
                    mutable,
                    safety: previous.map_or(Safety::Default, |g| g.safety),
                    naming,
                    linkage: Linkage::Export,
                },
            )
            .map_err(error)?;
    }
    facts
        .crates
        .get_or_insert_default()
        .symbols
        .insert(definition, external.rust_symbol());
    Ok(Entry {
        definition,
        external,
        ty,
        symbol,
        public,
        mutable,
        origin,
    })
}

fn static_root(
    reader: Reader<'_>,
    mut location: LocationId,
    cache: &mut HashMap<LocationId, DefId>,
    steps: &mut usize,
) -> Result<DefId, Error> {
    let mut path = Vec::new();
    let definition = loop {
        if let Some(&definition) = cache.get(&location) {
            break definition;
        }
        *steps = steps.saturating_add(1);
        if *steps > 1_000_000 {
            return Err(error("metadata static ancestry exceeds profile budget"));
        }
        path.push(location);
        match reader.location(location).map_err(error)?.origin {
            Origin::Static(definition) => break definition,
            Origin::Projection { base, .. } => location = base,
            _ => {
                return Err(error(
                    "unnamed or reinterpreted storage is outside the metadata profile",
                ));
            }
        }
    };
    for location in path {
        cache.insert(location, definition);
    }
    Ok(definition)
}
