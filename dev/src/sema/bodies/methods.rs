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
        if self.bind_library_owner(node, bindings)? {
            return Ok(());
        }
        let selected = self.facts.methods.qualified.get(&self.key(node)).copied();
        let dot = self.state.call_types.method_owners.get(&node).copied();
        let Some(implementation) = selected
            .map(|s| s.implementation)
            .or_else(|| dot.map(|d| d.0))
        else {
            return Ok(());
        };

        let schema = self
            .facts
            .associated
            .schema(implementation)
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
            .state
            .infer
            .instantiate(&mut self.facts.types, target, bindings)
            .map_err(|e| self.error(node, e))?;
        if let Some((_, prefix)) = dot {
            let prefix = self.alias_argument(node, prefix)?;
            return self.equate(node, target, prefix);
        }
        let selected = selected.ok_or_else(|| self.error(node, "missing selected method owner"))?;
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
            self.nominal_term(node, selected.prefix.node, selected.nominal, None)?
        };
        let prefix = self.alias_argument(node, prefix)?;
        self.equate(node, target, prefix)
    }
    fn bind_library_owner(
        &mut self,
        node: NodeId,
        bindings: &[(ParameterId, Argument)],
    ) -> Result<bool, Error> {
        if let Some(prefix) = self.facts.methods.length_prefix(self.key(node)) {
            let length = self
                .facts
                .methods
                .demanded_length()
                .ok_or_else(|| self.error(node, "missing selected slice method"))?;
            let prefix = self.annotation(prefix.node)?;
            let prefix = self.alias_argument(node, prefix)?;
            if !matches!(self.head(node, prefix)?, Some(super::Constructor::Slice)) {
                return Err(self.error(node, "slice method prefix is not a slice"));
            }
            let element = self.child(node, prefix, 0)?;
            let target = bindings
                .iter()
                .find_map(|(id, arg)| {
                    if *id == length.parameter {
                        if let Argument::Type(ty) = arg {
                            Some(*ty)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .ok_or_else(|| self.error(node, "missing inherited slice element argument"))?;
            self.equate(node, element, target)?;
            return Ok(true);
        }
        if let Some((prefix, method)) = self.facts.methods.pointer_prefix(self.key(node)) {
            let prefix = self.annotation(prefix.node)?;
            let prefix = self.alias_argument(node, prefix)?;
            let Some(super::Constructor::Pointer { mutable }) = self.head(node, prefix)? else {
                return Err(self.error(node, "pointer method prefix is not a pointer"));
            };
            if !matches!(self.facts.types.kind(method.receiver), Some(crate::types::Kind::Pointer { mutable: expected, .. }) if mutable == expected)
            {
                return Err(self.error(node, "pointer method prefix has wrong mutability"));
            }
            let element = self.child(node, prefix, 0)?;
            let target = bindings
                .iter()
                .find_map(|(id, arg)| match arg {
                    Argument::Type(ty) if *id == method.parameter => Some(*ty),
                    _ => None,
                })
                .ok_or_else(|| self.error(node, "missing inherited pointer element argument"))?;
            self.equate(node, element, target)?;
            return Ok(true);
        }
        Ok(false)
    }
}
