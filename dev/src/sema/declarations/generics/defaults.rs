//! Default templates have query states independent of alias expansion and parameter identity.
use super::{Argument, Checker, Declaration, DefId, Error, NodeKey, State, Work};
use std::collections::HashMap;

#[derive(Clone, Copy)]
pub(in super::super) struct DefaultId(usize);
#[derive(Clone, Copy)]
struct Entry {
    declaration: Declaration,
    state: State,
}
#[derive(Default)]
pub(in super::super) struct Defaults {
    entries: Vec<Entry>,
    index: HashMap<DefId, DefaultId>,
}
impl Defaults {
    pub(super) fn insert(
        &mut self,
        binding: DefId,
        declaration: Declaration,
    ) -> Result<(), &'static str> {
        let id = DefaultId(self.entries.len());
        if self.index.insert(binding, id).is_some() {
            return Err("generic default already declared");
        }
        self.entries.push(Entry {
            declaration,
            state: State::Unseen,
        });
        Ok(())
    }
    pub(super) fn get(&self, binding: DefId) -> Option<DefaultId> {
        self.index.get(&binding).copied()
    }
}
impl Checker<'_> {
    pub(in super::super) fn generic_default(
        &mut self,
        id: DefaultId,
        finish: bool,
    ) -> Result<(), Error> {
        let Entry { declaration, state } = self.defaults.entries[id.0];
        if state == State::Complete {
            return Ok(());
        }
        let Declaration {
            owner,
            key,
            index,
            context,
            ..
        } = declaration;
        let syntax = self.generic_syntax(key)?;
        let node = syntax
            .default
            .ok_or_else(|| self.error(key, "missing generic default"))?;
        if !finish {
            if state == State::Active {
                return Err(self.error(key, "recursive generic default"));
            }
            self.defaults.entries[id.0].state = State::Active;
            self.work.push(Work::Default(id, true));
            if syntax.flavor == crate::syntax::item::GenericFlavor::Type {
                self.node_work(key.file, node, context);
            } else {
                self.constant_dependencies(NodeKey { node, ..key }, context)?;
            }
            return Ok(());
        }
        let key = NodeKey { node, ..key };
        let argument = self.facts.generics.parameters[&owner][index as usize].argument;
        let default = match argument {
            Argument::Type(_) => Argument::Type(self.ty(key)?),
            Argument::Const(id) => {
                let ty = self
                    .facts
                    .types
                    .const_type(id)
                    .ok_or_else(|| self.error(key, "missing const parameter type"))?;
                Argument::Const(self.generic_constant(key, ty, owner)?)
            }
            Argument::Lifetime(_) => return Err(self.error(key, "lifetimes cannot have defaults")),
        };
        if let Some(original) = self.facts.unreduced.node(key) {
            let parameter = self.facts.generics.parameters[&owner][index as usize].binding;
            self.facts.unreduced.retain_default(parameter, original);
        }
        self.facts
            .generics
            .parameters
            .get_mut(&owner)
            .ok_or_else(|| Error::at(key.file, 0, "missing generic owner"))?[index as usize]
            .default = Some(default);
        self.defaults.entries[id.0].state = State::Complete;
        Ok(())
    }
    pub(in super::super) fn complete_deferred_declarations(
        &mut self,
        steps: &mut usize,
        limit: usize,
    ) -> Result<(), Error> {
        // Explicit applications demand only the body and written arguments.
        // Nevertheless every declared default must be checked, including unused ones.
        let mut next = 0;
        while next < self.defaults.entries.len() {
            self.work.push(Work::Default(DefaultId(next), false));
            self.drain(steps, limit)?;
            next += 1;
        }
        for (owner, context) in std::mem::take(&mut self.deferred_bounds) {
            self.schedule_bounds(owner, context)?;
            self.drain(steps, limit)?;
        }
        Ok(())
    }
}
