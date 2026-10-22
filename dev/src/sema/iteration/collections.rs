//! Exact collection records retain complete impl and callable contracts.
mod contracts;
use crate::{
    resolve::model::{BuiltinCollectionDeclaration, BuiltinCollections, BuiltinIteration, DefId},
    support::iteration::Collection,
    types::{ItemId, TypeId, parameters::predicates::Predicate},
};
#[derive(Clone, Copy)]
pub(crate) struct Original {
    pub declaration: BuiltinCollectionDeclaration,
    pub parameter: TypeId,
    pub subject: TypeId,
    pub item: TypeId,
    pub iterator: TypeId,
    pub signature: TypeId,
    pub predicate: Option<Predicate>,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Status {
    Prepared,
    Complete,
}
pub(crate) struct Registry {
    pub status: Status,
    pub mutable: Option<Box<MutableRegistry>>,
    pub canonical: BuiltinCollections,
    pub originals: [Option<Original>; 4],
    pub prefixes:
        crate::support::hash::HashMap<crate::resolve::unit::NodeKey, crate::resolve::unit::NodeKey>,
}
pub(crate) struct MutableRegistry {
    pub canonical: crate::resolve::model::BuiltinMutableCollections,
    pub status: Status,
    pub originals: [Option<Original>; 4],
}
#[derive(Clone, Copy)]
pub(crate) struct Group<'a> {
    pub iterator: DefId,
    pub declarations: &'a [BuiltinCollectionDeclaration; 4],
    pub originals: &'a [Option<Original>; 4],
    pub status: Status,
}
impl Registry {
    pub fn group(&self, mutable: bool) -> Option<Group<'_>> {
        if mutable {
            self.mutable.as_deref().map(|m| Group {
                iterator: m.canonical.iterator,
                declarations: &m.canonical.declarations,
                originals: &m.originals,
                status: m.status,
            })
        } else {
            Some(Group {
                iterator: self.canonical.iterator,
                declarations: &self.canonical.declarations,
                originals: &self.originals,
                status: self.status,
            })
        }
    }
    pub fn all_originals(&self) -> impl Iterator<Item = &Original> {
        self.originals
            .iter()
            .chain(self.mutable.iter().flat_map(|m| m.originals.iter()))
            .flatten()
    }

    pub fn associated(
        &self,
        iteration: BuiltinIteration,
        owner: DefId,
        item: ItemId,
    ) -> Result<Option<DefId>, &'static str> {
        let Some(original) = self
            .all_originals()
            .find(|o| o.declaration.definition == owner)
        else {
            return Ok(None);
        };
        let Some(trait_) = original.declaration.kind.trait_() else {
            return Ok(None);
        };
        let expected = iteration.get(trait_);
        if item == ItemId(expected.item.0) {
            return original
                .declaration
                .item
                .map(Some)
                .ok_or("missing collection Item definition");
        }
        if expected.into_iter == Some(DefId(item.0)) {
            return original
                .declaration
                .into_iter
                .map(Some)
                .ok_or("missing collection IntoIter definition");
        }
        Err("collection implementation does not own associated member")
    }
    pub fn original(&self, method: DefId) -> Option<Original> {
        self.all_originals()
            .find(|o| o.declaration.method == method)
            .copied()
    }
}
/// Only an exact checked receiver shape requests a canonical header. Unknown
/// projections remain inference work and supply no closed absence or permission.
pub(in crate::sema) fn requested_owner(
    report: &crate::resolve::expansion::Report,
    store: &crate::types::Store,
    context: &mut crate::types::infer::Context,
    term: crate::types::infer::TermId,
) -> Result<Option<DefId>, crate::types::infer::Error> {
    use crate::types::infer::{Constructor, Error};
    let Some(canonical) = report.resolution.builtin_traits.collections() else {
        return Ok(None);
    };
    let mut head = |term| {
        context.account_work(1)?;
        match context.head(store, term) {
            Ok(head) => Ok(head),
            Err(Error::Normalization | Error::Unresolved) => Ok(None),
            Err(error) => Err(error),
        }
    };
    let kind = match head(term)? {
        Some(Constructor::Adt(item, _) | Constructor::AdtArgs(item, _))
            if item == ItemId(canonical.iterator.0) =>
        {
            Some(Collection::SliceIterator)
        }
        Some(Constructor::Adt(item, _) | Constructor::AdtArgs(item, _))
            if canonical
                .mutable
                .as_deref()
                .is_some_and(|m| item == ItemId(m.iterator.0)) =>
        {
            Some(Collection::SliceMutableIterator)
        }
        Some(Constructor::Reference { mutable, .. }) => {
            let term = context.child(store, term, 0)?;
            context.account_work(1)?;
            match context.head(store, term) {
                Ok(Some(Constructor::Slice)) => Some(if mutable {
                    Collection::MutableSlice
                } else {
                    Collection::SharedSlice
                }),
                Ok(Some(
                    Constructor::Array(_) | Constructor::ArrayTerm(_) | Constructor::ArrayInfer(_),
                )) => Some(if mutable {
                    Collection::MutableArray
                } else {
                    Collection::SharedArray
                }),
                Ok(_) | Err(Error::Normalization | Error::Unresolved) => None,
                Err(error) => return Err(error),
            }
        }
        _ => None,
    };
    Ok(kind.and_then(|kind| canonical.declaration(kind).map(|d| d.definition)))
}
impl Original {
    pub fn validate(self, facts: &crate::sema::model::Declarations) -> Result<(), &'static str> {
        let registry = facts
            .iteration
            .as_deref()
            .and_then(|r| r.collections.as_deref())
            .ok_or("missing collection registry")?;
        let group = registry
            .group(self.declaration.kind.mutable())
            .ok_or("missing active collection group")?;
        if group.declarations[self.declaration.kind.index()] != self.declaration {
            return Err("collection original differs from canonical owner");
        }
        if group.status != Status::Complete {
            return Err("collection original requires completed conformance");
        }
        let registered = registry
            .original(self.declaration.method)
            .ok_or("unregistered collection original")?;
        if registered.parameter != self.parameter
            || registered.subject != self.subject
            || registered.item != self.item
            || registered.iterator != self.iterator
            || registered.signature != self.signature
            || registered.predicate != self.predicate
        {
            return Err("collection original differs from registered contract");
        }
        let Some(crate::types::Kind::Parameter(parameter)) = facts.types.kind(self.parameter)
        else {
            return Err("collection element is not its parameter");
        };
        let identity = facts
            .types
            .parameter(parameter)
            .ok_or("missing collection parameter")?;
        if identity.owner != ItemId(self.declaration.definition.0)
            || identity.index != u32::from(self.declaration.lifetime.is_some())
            || identity.kind != (crate::types::parameters::ParameterKind::Type { sized: true })
        {
            return Err("collection element differs from owner schema");
        }
        let canonical = facts.definitions[group.iterator.0 as usize]
            .ok_or("missing canonical iterator type")?;
        let Some(crate::types::Kind::Adt(owner, _)) = facts.types.kind(canonical) else {
            return Err("invalid canonical iterator type");
        };
        let Some(crate::types::Kind::Adt(actual, arguments)) = facts.types.kind(self.iterator)
        else {
            return Err("collection result is not canonical iterator");
        };
        if owner != actual
            || facts
                .types
                .arguments(arguments)
                .is_none_or(|a| a.count() != 2)
        {
            return Err("collection iterator identity/schema differs");
        }
        if let Some(predicate) = self.predicate {
            let inventory = facts
                .generics
                .implementations
                .as_ref()
                .ok_or("missing collection implementation inventory")?;
            if !inventory.is_validated(self.declaration.definition)
                || inventory
                    .header(self.declaration.definition)
                    .is_none_or(|h| h.predicate != predicate)
            {
                return Err("collection requires exact checked implementation");
            }
        } else if !self.declaration.kind.method() {
            return Err("collection impl has no predicate");
        }
        if facts.definitions[self.declaration.method.0 as usize].and_then(|t|facts.types.kind(t)).is_none_or(|k|!matches!(k,crate::types::Kind::FunctionItem {item,signature,..} if item==ItemId(self.declaration.method.0)&&signature==self.signature)) {return Err("collection method differs from original callable");}
        self.validate_contract(facts)
    }
}
