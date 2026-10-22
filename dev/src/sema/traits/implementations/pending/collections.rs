//! Only proved shape disjointness excludes latent canonical collection headers.
use super::{DefId, Head, Identity, Pending};
use crate::{
    resolve::{expansion::Report, model::DefKind},
    support::iteration::{Collection, Trait},
    types::{
        ItemId, Store,
        infer::{Constructor, Context, Error, TermId},
    },
};
pub(super) struct Heads {
    iterator: ItemId,
    entries: [(DefId, Collection); 3],
    cloning: Option<DefId>,
}
#[derive(Clone, Copy)]
enum Shape {
    Slice,
    Array,
    Iterator,
    Other,
    Unknown,
}
impl Pending {
    pub(in super::super) fn collection_owner(&self, owner: DefId) -> bool {
        self.owners.contains_key(&owner)
            && self.collections.as_deref().is_some_and(|rules| {
                rules.entries.iter().any(|(id, _)| *id == owner) || rules.cloning == Some(owner)
            })
    }
    pub(in super::super) fn only_collections(&self) -> bool {
        self.owners
            .keys()
            .all(|owner| self.collection_owner(*owner))
    }

    pub(in super::super) fn install_collections(
        &mut self,
        report: &Report,
    ) -> Result<(), &'static str> {
        if self.collections.is_some() {
            return Err("collection scheduling rules installed twice");
        }
        let canonical = report
            .resolution
            .builtin_traits
            .collections()
            .ok_or("missing canonical collection catalog")?;
        let iteration = report
            .resolution
            .builtin_traits
            .iteration()
            .ok_or("missing canonical iteration catalog")?;
        let nominal = report
            .resolution
            .library_types
            .get(crate::support::library_types::LibraryType::SliceIter)
            .ok_or("missing canonical slice iterator")?;
        if nominal.definition != canonical.iterator {
            return Err("collection scheduling iterator differs from canonical nominal");
        }
        let mut entries = Vec::new();
        for declaration in canonical.declarations {
            let Some(trait_) = declaration.kind.trait_() else {
                continue;
            };
            let expected = Identity::User(ItemId(iteration.get(trait_).definition.0));
            let head = if trait_ == Trait::Iterator {
                Head::Nominal(ItemId(canonical.iterator.0))
            } else {
                Head::Reference(false)
            };
            if self.owners.get(&declaration.definition) != Some(&(expected, Some(head)))
                || report
                    .resolution
                    .definitions
                    .get(declaration.definition.0 as usize)
                    .is_none_or(|d| d.kind != DefKind::SelfType || d.node.is_some())
            {
                return Err("collection scheduling rule does not match a reserved canonical owner");
            }
            entries.push((declaration.definition, declaration.kind));
        }
        let entries = entries
            .try_into()
            .map_err(|_| "invalid collection scheduling rule count")?;
        let cloning = report.resolution.builtin_traits.cloning().and_then(|c| {
            c.implementation
                .map(|implementation| (c.definition, implementation))
        });
        let cloning = if let Some((definition, implementation)) = cloning {
            if report
                .resolution
                .builtin_traits
                .get(crate::resolve::model::BuiltinTrait::Clone)
                != Some(definition)
                || self.owners.get(&implementation.definition)
                    != Some(&(
                        Identity::User(ItemId(definition.0)),
                        Some(Head::Nominal(ItemId(canonical.iterator.0))),
                    ))
                || report
                    .resolution
                    .definitions
                    .get(implementation.definition.0 as usize)
                    .is_none_or(|d| d.kind != DefKind::SelfType || d.node.is_some())
            {
                return Err("Clone scheduling rule differs from canonical implementation");
            }
            Some(implementation.definition)
        } else {
            None
        };
        self.collections = Some(Box::new(Heads {
            iterator: ItemId(canonical.iterator.0),
            entries,
            cloning,
        }));
        Ok(())
    }
    pub(in super::super) fn filter_collections(
        &self,
        store: &Store,
        context: &mut Context,
        subject: TermId,
        candidates: &mut Vec<DefId>,
    ) -> Result<(), Error> {
        self.filter_collection_shapes(context, candidates, |context, iterator| {
            classify(store, context, subject, iterator)
        })
    }
    pub(in super::super) fn filter_known_collections(
        &self,
        store: &Store,
        context: &mut Context,
        subject: crate::types::TypeId,
        candidates: &mut Vec<DefId>,
    ) -> Result<(), Error> {
        self.filter_collection_shapes(context, candidates, |context, iterator| {
            context.account_work(2)?;
            let root = store.kind(subject).ok_or(Error::InvalidIdentity)?;
            let child = if let crate::types::Kind::Reference {
                mutable: false,
                referent,
                ..
            } = root
            {
                Some(Head::canonical(
                    store.kind(referent).ok_or(Error::InvalidIdentity)?,
                ))
            } else {
                None
            };
            Ok(classify_heads(Some(Head::canonical(root)), child, iterator))
        })
    }
    fn filter_collection_shapes(
        &self,
        context: &mut Context,
        candidates: &mut Vec<DefId>,
        mut classify: impl FnMut(&mut Context, ItemId) -> Result<Shape, Error>,
    ) -> Result<(), Error> {
        let Some(rules) = self.collections.as_deref() else {
            return Ok(());
        };
        let mut shape = None;
        let mut end = 0;
        for index in 0..candidates.len() {
            let owner = candidates[index];
            context.account_work(3)?;
            let kind = rules
                .entries
                .iter()
                .find(|(id, _)| *id == owner)
                .map(|(_, kind)| *kind)
                .or_else(|| (rules.cloning == Some(owner)).then_some(Collection::SliceIterator));
            let keep = if let Some(kind) = kind {
                let actual = if let Some(shape) = shape {
                    shape
                } else {
                    let actual = classify(context, rules.iterator)?;
                    shape = Some(actual);
                    actual
                };
                matches!(
                    (kind, actual),
                    (Collection::SharedSlice, Shape::Slice)
                        | (Collection::SharedArray, Shape::Array)
                        | (Collection::SliceIterator, Shape::Iterator)
                        | (_, Shape::Unknown)
                )
            } else {
                true
            };
            if keep {
                candidates[end] = owner;
                end += 1;
            }
        }
        candidates.truncate(end);
        Ok(())
    }
}

fn head(store: &Store, context: &mut Context, term: TermId) -> Result<Option<Constructor>, Error> {
    context.account_work(1)?;
    match context.head(store, term) {
        Ok(value) => Ok(value),
        Err(Error::Normalization | Error::Unresolved) => Ok(None),
        Err(error) => Err(error),
    }
}
fn classify(
    store: &Store,
    context: &mut Context,
    term: TermId,
    iterator: ItemId,
) -> Result<Shape, Error> {
    let root = head(store, context, term)?.map(Head::inferred);
    let child = if matches!(root, Some(Head::Reference(false))) {
        let child = context.child(store, term, 0)?;
        head(store, context, child)?.map(Head::inferred)
    } else {
        None
    };
    Ok(classify_heads(root, child, iterator))
}
fn classify_heads(root: Option<Head>, child: Option<Head>, iterator: ItemId) -> Shape {
    match root {
        Some(Head::Nominal(item)) if item == iterator => Shape::Iterator,
        Some(Head::Reference(false)) => match child {
            Some(Head::Slice) => Shape::Slice,
            Some(Head::Array) => Shape::Array,
            None | Some(Head::Projection(_)) => Shape::Unknown,
            _ => Shape::Other,
        },
        None | Some(Head::Projection(_)) => Shape::Unknown,
        _ => Shape::Other,
    }
}
