//! Bind prepared written arguments only after the receiver selects a declaration.
use super::{Checker, DefId, Error, NodeId};
use crate::{
    sema::{
        arguments,
        generics::{DeferredArgument, ItemUse, SourceArgument},
    },
    types::arguments::Argument,
};

impl Checker<'_> {
    pub(super) fn complete_method_arguments(
        &mut self,
        node: NodeId,
        owner: DefId,
    ) -> Result<(), Error> {
        let key = self.key(node);
        let use_ = self.facts.generics.method_uses.get(&key);
        let (lifetimes, supplied) =
            use_.map_or((None, Vec::new()), |u| (u.lifetimes, u.arguments.clone()));
        let parameters = self
            .facts
            .generics
            .parameters
            .get(&owner)
            .cloned()
            .unwrap_or_default();
        let offset = arguments::arity(&parameters, supplied.len(), |_| false)
            .map_err(|e| self.error(node, e))?;
        let mut arguments = Vec::with_capacity(supplied.len());
        for (index, source) in supplied.into_iter().enumerate() {
            arguments.push(match (source, parameters[index + offset].argument) {
                (DeferredArgument::Infer, _) => SourceArgument::Infer,
                (DeferredArgument::Type(key), Argument::Type(_)) => SourceArgument::Type(key),
                (DeferredArgument::Const(key), Argument::Const(formal)) => {
                    let ty = self
                        .facts
                        .types
                        .const_type(formal)
                        .ok_or_else(|| self.error(node, "missing const parameter type"))?;
                    let cache = self.method_constants.get_or_insert_with(Default::default);
                    let argument = arguments::Constants {
                        report: self.report,
                        facts: self.facts,
                        captures: &mut cache.captures,
                        scratch: &mut cache.scratch,
                        discriminants: &mut cache.discriminants,
                    }
                    .argument(key, ty, Some(self.module))?;
                    SourceArgument::Const(argument)
                }
                _ => return Err(self.error(node, "method argument has the wrong generic kind")),
            });
        }
        self.facts.generics.item_uses.insert(
            key,
            ItemUse {
                lifetimes,
                arguments,
            },
        );
        Ok(())
    }
}
