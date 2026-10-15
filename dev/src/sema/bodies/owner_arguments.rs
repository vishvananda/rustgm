//! Explicit trait-prefix arguments constrain the same callable variables as ordinary inference.
use super::{Checker, Error, NodeId};
use crate::{
    sema::{annotations::Fields, generics::SourceArgument},
    types::{
        infer::{Instantiation, arguments::Argument},
        parameters::ParameterId,
        substitute::Bound,
    },
};

impl Checker<'_> {
    pub(super) fn bind_trait_arguments(
        &mut self,
        node: NodeId,
        arguments: &[Argument],
        bindings: &[(ParameterId, Argument)],
    ) -> Result<(), Error> {
        let Some((owner, use_)) = self.facts.generics.trait_uses.get(&self.key(node)) else {
            return Ok(());
        };
        let owner = *owner;
        let supplied = use_.arguments.clone();
        let parameters = self
            .facts
            .generics
            .parameters
            .get(&owner)
            .ok_or_else(|| self.error(node, "missing trait prefix parameters"))?
            .iter()
            .filter(|p| !matches!(p.argument, crate::types::arguments::Argument::Lifetime(_)))
            .copied()
            .collect::<Vec<_>>();
        let actual = bindings
            .iter()
            .filter(|(id, _)| {
                self.facts
                    .types
                    .parameter(*id)
                    .is_some_and(|p| p.owner.0 == owner.0)
            })
            .map(|(_, value)| *value)
            .collect::<Vec<_>>();
        if parameters.len() != actual.len() {
            return Err(self.error(node, "trait callable parameter count mismatch"));
        }
        let mut defaults = Vec::new();
        for (index, (parameter, actual)) in parameters.iter().zip(actual).enumerate() {
            match (supplied.get(index).copied(), actual) {
                (Some(SourceArgument::Type(source)), Argument::Type(term)) => {
                    let supplied = self.annotation(source.node)?;
                    let supplied = self.alias_argument(node, supplied)?;
                    self.equate(node, term, supplied)?;
                }
                (Some(SourceArgument::Const(id)), Argument::Const(term)) => {
                    let supplied = self
                        .infer
                        .known_const(&self.facts.types, id)
                        .map_err(|e| self.error(node, e))?;
                    self.infer
                        .equate_consts(term, supplied)
                        .map_err(|e| self.error(node, e))?;
                }
                (Some(SourceArgument::Infer), _) => {}
                (None, _) => {
                    if !supplied.is_empty()
                        && let Some(default) = parameter.default
                    {
                        defaults.push((default, actual));
                    }
                }
                _ => return Err(self.error(node, "trait prefix argument kind mismatch")),
            }
        }
        if defaults.is_empty() {
            return Ok(());
        }
        let regions = arguments
            .iter()
            .take_while(|argument| matches!(argument, Argument::Lifetime(_)))
            .filter_map(|argument| {
                if let Argument::Lifetime(region) = *argument {
                    Some(region)
                } else {
                    None
                }
            })
            .collect();
        let mut fields = Fields {
            regions: Bound::new(regions),
            parameters: Instantiation::new(&mut self.infer, &self.facts.types, bindings)
                .map_err(|e| self.error(node, e))?,
        };
        for (default, actual) in defaults {
            fields.default(self, self.key(node), default, actual)?;
        }
        Ok(())
    }
}
