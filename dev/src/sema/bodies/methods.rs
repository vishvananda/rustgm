//! Selected impl arguments share the ordinary callable inference graph.
use super::{Checker, Error, NodeId};
use crate::types::{infer::arguments::Argument, parameters::ParameterId, substitute::Bound};

impl Checker<'_> {
    pub(super) fn bind_method_owner(
        &mut self,
        node: NodeId,
        arguments: &[Argument],
        bindings: &[(ParameterId, Argument)],
    ) -> Result<(), Error> {
        let Some(selected) = self.facts.methods.qualified.get(&self.key(node)).copied() else {
            return Ok(());
        };
        let schema = self
            .facts
            .associated
            .schema(selected.implementation)
            .ok_or_else(|| self.error(node, "missing selected impl schema"))?;
        let target = schema.self_type;
        let captures = schema.captures as usize;
        let mut regions = Vec::with_capacity(captures);
        for argument in arguments.iter().take(captures) {
            let Argument::Lifetime(region) = argument else {
                return Err(self.error(node, "missing selected impl lifetime argument"));
            };
            regions.push(*region);
        }
        if regions.len() != captures {
            return Err(self.error(node, "incomplete impl lifetime arguments"));
        }
        let target = Bound::new(regions)
            .apply(&mut self.facts.types, target)
            .map_err(|e| self.error(node, e))?;
        let target = self
            .infer
            .instantiate(&mut self.facts.types, target, bindings)
            .map_err(|e| self.error(node, e))?;
        let prefix = if selected.annotation
            || self
                .facts
                .generics
                .annotation_uses
                .contains_key(&selected.prefix)
            || self
                .facts
                .node(selected.prefix)
                .is_some_and(|f| f.ty.is_some())
        {
            self.annotation(selected.prefix.node)?
        } else {
            self.nominal_term(node, selected.prefix.node, selected.nominal)?
        };
        let prefix = self.alias_argument(node, prefix)?;
        self.equate(node, target, prefix)
    }
}
