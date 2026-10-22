//! Complete concrete integer range originals retain their exact canonical owners.
use crate::{
    resolve::model::{BuiltinRangeImplementation, BuiltinRanges},
    sema::model::Declarations,
    types::{
        GenericArgs, ItemId, Kind, RegionId, TypeId, arguments::Argument,
        parameters::predicates::Predicate,
    },
};
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct Original {
    pub declaration: BuiltinRangeImplementation,
    pub element: TypeId,
    pub subject: TypeId,
    pub signature: TypeId,
    pub predicate: Predicate,
}
pub(crate) struct Registry {
    pub canonical: BuiltinRanges,
    pub originals: Vec<Original>,
    pub complete: bool,
}
impl Original {
    pub fn validate(self, facts: &Declarations) -> Result<(), &'static str> {
        let iteration = facts
            .iteration
            .as_deref()
            .ok_or("missing range iteration owner")?;
        let registry = iteration
            .ranges
            .as_deref()
            .ok_or("missing range contracts")?;
        if !registry.complete || !registry.originals.contains(&self) {
            return Err("range original lacks completed canonical registration");
        }
        if !registry
            .canonical
            .implementations
            .contains(&self.declaration)
        {
            return Err("range declaration differs from canonical owner");
        }
        let Some(Kind::Primitive(integer)) = facts.types.kind(self.element) else {
            return Err("range element is not primitive");
        };
        if integer.integer().is_none() {
            return Err("range element is not integer");
        }
        if facts.definitions[self.declaration.primitive.0 as usize] != Some(self.element) {
            return Err("range primitive differs from canonical identity");
        }
        let Some(Kind::Adt(nominal, args)) = facts.types.kind(self.subject) else {
            return Err("range Self is not nominal");
        };
        if nominal != ItemId(registry.canonical.nominal.0)
            || facts
                .types
                .arguments(args)
                .is_none_or(|a| a.collect::<Vec<_>>() != [Argument::Type(self.element)])
        {
            return Err("range Self differs from canonical schema");
        }
        let inventory = facts
            .generics
            .implementations
            .as_ref()
            .ok_or("missing range implementation inventory")?;
        if !inventory.is_validated(self.declaration.definition)
            || inventory
                .header(self.declaration.definition)
                .is_none_or(|h| h.predicate != self.predicate)
            || inventory.method(
                self.declaration.definition,
                iteration
                    .canonical
                    .get(crate::support::iteration::Trait::Iterator)
                    .method,
            ) != Some(self.declaration.method)
        {
            return Err("range original lacks exact selected implementation");
        }
        if inventory
            .header(self.declaration.definition)
            .is_none_or(|header| {
                header.types.as_slice()
                    != [(
                        iteration
                            .canonical
                            .get(crate::support::iteration::Trait::Iterator)
                            .item,
                        self.declaration.item,
                    )]
            })
            || facts.definitions[self.declaration.item.0 as usize] != Some(self.element)
        {
            return Err("range Item definition differs from its declared element");
        }
        self.validate_signature(facts, ItemId(iteration.canonical.option.0))
    }
    fn validate_signature(
        self,
        facts: &Declarations,
        expected_option: ItemId,
    ) -> Result<(), &'static str> {
        let scheme = facts
            .lifetimes
            .functions
            .get(&self.declaration.method)
            .ok_or("missing range function regions")?;
        if scheme.total != 1
            || scheme.captures != 0
            || !scheme.early.is_empty()
            || scheme.late != [0]
            || !scheme.bounds.is_empty()
        {
            return Err("range original has wrong receiver binder");
        }
        let Some(Kind::Function {
            parameters,
            result,
            unsafe_: false,
            convention: crate::types::callable::Convention::RUST,
            bound_regions: 1,
        }) = facts.types.kind(self.signature)
        else {
            return Err("invalid range next signature");
        };
        let [receiver] = facts
            .types
            .list(parameters)
            .ok_or("missing range next input")?
        else {
            return Err("range next arity differs");
        };
        if facts.types.kind(*receiver)
            != Some(Kind::Reference {
                referent: self.subject,
                mutable: true,
                region: RegionId::bound(0, 0)?,
            })
        {
            return Err("range next has different receiver");
        }
        let Some(Kind::Adt(option, args)) = facts.types.kind(result) else {
            return Err("range next result is not Option");
        };
        if option != expected_option
            || facts
                .types
                .arguments(args)
                .is_none_or(|a| a.collect::<Vec<_>>() != [Argument::Type(self.element)])
        {
            return Err("range next has different Item result");
        }
        if facts.definitions[self.declaration.method.0 as usize].and_then(|ty| facts.types.kind(ty))
            != Some(Kind::FunctionItem {
                item: ItemId(self.declaration.method.0),
                signature: self.signature,
                arguments: GenericArgs::EMPTY,
            })
        {
            return Err("range next differs from canonical callable");
        }
        Ok(())
    }
}

pub(in crate::sema) fn requested_owner(
    report: &crate::resolve::expansion::Report,
    store: &crate::types::Store,
    context: &mut crate::types::infer::Context,
    mut subject: crate::types::infer::TermId,
) -> Result<Option<crate::resolve::model::DefId>, crate::types::infer::Error> {
    use crate::types::infer::{Constructor, Error};
    let Some(canonical) = report.resolution.builtin_traits.ranges() else {
        return Ok(None);
    };
    loop {
        context.account_work(1)?;
        let head = match context.head(store, subject) {
            Ok(head) => head,
            Err(Error::Normalization | Error::Unresolved) => return Ok(None),
            Err(error) => return Err(error),
        };
        match head {
            Some(Constructor::Reference { mutable: true, .. }) => {
                subject = context.child(store, subject, 0)?;
            }
            Some(Constructor::Adt(item, _) | Constructor::AdtArgs(item, _))
                if item == ItemId(canonical.nominal.0) =>
            {
                return Ok(Some(canonical.implementations[0].definition));
            }
            _ => return Ok(None),
        }
    }
}
