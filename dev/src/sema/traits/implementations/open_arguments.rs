//! An open header's rigid trait argument can exclude a candidate before inference.
use super::bucket_growth;
use crate::{
    support::{
        hash::HashMap,
        retained::{Growth, map, vector},
    },
    types::{ItemId, Kind, Store, arguments::Argument, parameters::predicates::Predicate},
};

/// A nominal argument can only unify with the same constructor. Projections,
/// parameters and all other shapes stay in the fallback lane.
pub(super) fn nominal(store: &Store, goal: Predicate) -> (Option<ItemId>, usize) {
    let Some(arguments) = goal
        .bound
        .trait_parts()
        .and_then(|(application, _)| store.trait_application(application))
        .and_then(|application| store.arguments(application.arguments))
    else {
        return (None, 1);
    };
    let mut work = 1_usize;
    for argument in arguments {
        work = work.saturating_add(1);
        match argument {
            Argument::Lifetime(_) => {}
            Argument::Type(ty) => {
                let key = match store.kind(ty) {
                    Some(Kind::Adt(item, _)) => Some(item),
                    _ => None,
                };
                return (key, work);
            }
            Argument::Const(_) => return (None, work),
        }
    }
    (None, work)
}

#[derive(Default)]
pub(super) struct Bucket {
    all: Vec<usize>,
    arguments: Option<Box<Arguments>>,
}

#[derive(Default)]
struct Arguments {
    nominal: HashMap<ItemId, Vec<usize>>,
    fallback: Vec<usize>,
}

impl Bucket {
    pub(super) fn all(&self) -> &[usize] {
        &self.all
    }

    pub(super) fn candidates(&self, argument: Option<ItemId>) -> [&[usize]; 2] {
        match (argument, self.arguments.as_deref()) {
            (Some(argument), Some(rows)) => [
                rows.nominal.get(&argument).map_or(&[], Vec::as_slice),
                &rows.fallback,
            ],
            _ => [&self.all, &[]],
        }
    }

    pub(super) fn push(&mut self, index: usize, argument: Option<ItemId>) {
        if self.arguments.is_none() && argument.is_some() {
            self.arguments = Some(Box::new(Arguments {
                fallback: self.all.clone(),
                ..Arguments::default()
            }));
        }
        if let Some(rows) = &mut self.arguments {
            if let Some(nominal) = argument {
                rows.nominal.entry(nominal).or_default().push(index);
            } else {
                rows.fallback.push(index);
            }
        }
        self.all.push(index);
    }

    pub(super) fn growth(&self, argument: Option<ItemId>) -> Growth {
        let mut growth = Growth::vector::<usize>(self.all.len(), self.all.capacity(), 1);
        let index = match (self.arguments.as_deref(), argument) {
            (Some(rows), Some(key)) => bucket_growth(&rows.nominal, &key),
            (Some(rows), None) => {
                Growth::vector::<usize>(rows.fallback.len(), rows.fallback.capacity(), 1)
            }
            (None, Some(key)) => {
                let mut index = bucket_growth(&HashMap::default(), &key);
                index.include(Growth {
                    bytes: size_of::<Arguments>()
                        .saturating_add(self.all.len().saturating_mul(size_of::<usize>())),
                    work: self.all.len(),
                });
                index
            }
            (None, None) => Growth::default(),
        };
        growth.include(index);
        growth
    }

    pub(super) fn allocation(&self, argument: Option<ItemId>) -> usize {
        let indexed = self.arguments.as_deref().map_or(0, |rows| {
            size_of::<Arguments>()
                .saturating_add(map(&rows.nominal))
                .saturating_add(vector(&rows.fallback))
                .saturating_add(
                    argument
                        .and_then(|key| rows.nominal.get(&key))
                        .map_or(0, vector),
                )
        });
        vector(&self.all).saturating_add(indexed)
    }
}
