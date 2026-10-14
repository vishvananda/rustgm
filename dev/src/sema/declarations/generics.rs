//! Alias declaration environments are elaborated once by the declaration worklist.
mod arguments;
mod constants;
use super::{
    Checker, DefId, Error, GenericFlavor, ItemId, Kind, NodeKey, State, Type, TypeId, Work, regions,
};
use crate::{
    sema::generics::Parameter,
    syntax::item::GenericParameter,
    types::{
        arguments::{Argument, ConstKind},
        parameters::{Parameter as TypeParameter, ParameterKind},
    },
};

#[derive(Clone, Copy)]
pub(super) enum Phase {
    Begin,
    Declare,
    Default,
}
#[derive(Clone, Copy)]
pub(super) struct Declaration {
    owner: DefId,
    key: NodeKey,
    index: u32,
    context: regions::Context,
    phase: Phase,
}
impl Checker<'_> {
    pub(super) fn alias_parameters(
        &mut self,
        owner: DefId,
        key: NodeKey,
        context: regions::Context,
    ) -> Result<(), Error> {
        let item = self.item(key)?;
        let unit = self.report.package.unit(key.file);
        let nodes: Vec<_> = unit
            .configured
            .ast
            .list(item.generics)
            .iter()
            .copied()
            .filter(|node| unit.configured.active[node.0 as usize])
            .map(|node| self.unattributed(NodeKey { node, ..key }))
            .collect();
        if !nodes.iter().any(|key| {
            self.generic_syntax(*key)
                .is_ok_and(|p| p.flavor != GenericFlavor::Lifetime)
        }) {
            return Ok(());
        }
        let mut names = std::collections::HashSet::new();
        let mut non_lifetime = false;
        let mut defaulted = false;
        for node in &nodes {
            let parameter = self.generic_syntax(*node)?;
            let binding = self.generic_binding(*node)?;
            let name = self.report.resolution.definitions[binding.0 as usize].name;
            let lifetime = parameter.flavor == GenericFlavor::Lifetime;
            if !names.insert((lifetime, name)) {
                return Err(self.error(*node, "duplicate generic parameter"));
            }
            if lifetime && non_lifetime {
                return Err(self.error(
                    *node,
                    "lifetime parameters must precede type and const parameters",
                ));
            }
            non_lifetime |= !lifetime;
            if !lifetime && parameter.bounds.start != parameter.bounds.end {
                return Err(self.error(*node, "type parameter bounds require generic obligations"));
            }
            if !lifetime && defaulted && parameter.default.is_none() {
                return Err(self.error(*node, "generic parameters with defaults must be trailing"));
            }
            defaulted |= parameter.default.is_some();
        }
        self.facts
            .generics
            .parameters
            .insert(owner, Vec::with_capacity(nodes.len()));
        for (index, key) in nodes.into_iter().enumerate().rev() {
            self.work.push(Work::Generic(Declaration {
                owner,
                key,
                context,
                index: u32::try_from(index)
                    .map_err(|_| self.error(key, "too many generic parameters"))?,
                phase: Phase::Begin,
            }));
        }
        Ok(())
    }
    fn generic_syntax(&self, key: NodeKey) -> Result<GenericParameter, Error> {
        let Kind::Generic(index) = self.kind(key) else {
            return Err(self.error(key, "missing generic parameter syntax"));
        };
        Ok(self.report.package.unit(key.file).configured.ast.generics[index as usize])
    }
    fn generic_binding(&self, key: NodeKey) -> Result<DefId, Error> {
        self.facts
            .node(key)
            .and_then(|f| f.binding)
            .ok_or_else(|| self.error(key, "missing generic parameter binding"))
    }
    pub(super) fn generic_declaration(&mut self, declaration: Declaration) -> Result<(), Error> {
        let Declaration {
            owner,
            key,
            index,
            context,
            phase,
        } = declaration;
        let syntax = self.generic_syntax(key)?;
        if matches!(phase, Phase::Begin) && syntax.flavor == GenericFlavor::Const {
            self.work.push(Work::Generic(Declaration {
                phase: Phase::Declare,
                ..declaration
            }));
            self.node_work(
                key.file,
                syntax
                    .ty
                    .ok_or_else(|| self.error(key, "const parameter requires a type"))?,
                regions::Context::ALIAS,
            );
            return Ok(());
        }
        if matches!(phase, Phase::Default) {
            return self.generic_default(declaration, syntax);
        }
        let binding = self.generic_binding(key)?;
        let argument = if syntax.flavor == GenericFlavor::Lifetime {
            Argument::Lifetime(
                self.regions
                    .named(context.binder, binding)
                    .map_err(|e| self.error(key, e))?,
            )
        } else {
            let kind = if syntax.flavor == GenericFlavor::Type {
                ParameterKind::Type { sized: true }
            } else {
                ParameterKind::Const {
                    ty: self.ty(NodeKey {
                        node: syntax
                            .ty
                            .ok_or_else(|| self.error(key, "missing const parameter type"))?,
                        ..key
                    })?,
                }
            };
            let id = self
                .facts
                .types
                .intern_parameter(TypeParameter {
                    owner: ItemId(owner.0),
                    index,
                    kind,
                })
                .map_err(|e| self.error(key, e))?;
            match kind {
                ParameterKind::Type { .. } => {
                    let ty = self.intern(key, Type::Parameter(id))?;
                    self.complete(binding, key, ty)?;
                    Argument::Type(ty)
                }
                ParameterKind::Const { ty } => {
                    let term = self
                        .facts
                        .types
                        .intern_const(ConstKind::Parameter(id))
                        .map_err(|e| self.error(key, e))?;
                    self.facts.generics.constants.insert(binding, term);
                    self.facts.definitions[binding.0 as usize] = Some(ty);
                    self.states[binding.0 as usize] = State::Complete;
                    Argument::Const(term)
                }
            }
        };
        self.facts
            .generics
            .parameters
            .get_mut(&owner)
            .ok_or_else(|| Error::at(key.file, 0, "missing generic owner"))?
            .push(Parameter {
                binding,
                argument,
                default: None,
            });
        if let Some(node) = syntax.default {
            self.work.push(Work::Generic(Declaration {
                phase: Phase::Default,
                ..declaration
            }));
            if syntax.flavor == GenericFlavor::Type {
                self.node_work(key.file, node, context);
            } else {
                self.constant_dependencies(NodeKey { node, ..key })?;
            }
        }
        Ok(())
    }
    fn generic_default(
        &mut self,
        declaration: Declaration,
        syntax: GenericParameter,
    ) -> Result<(), Error> {
        let Declaration {
            owner, key, index, ..
        } = declaration;
        let node = syntax
            .default
            .ok_or_else(|| self.error(key, "missing generic default"))?;
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
            Argument::Lifetime(_) => {
                return Err(self.error(key, "lifetimes cannot have defaults"));
            }
        };
        self.facts
            .generics
            .parameters
            .get_mut(&owner)
            .ok_or_else(|| Error::at(key.file, 0, "missing generic owner"))?[index as usize]
            .default = Some(default);
        Ok(())
    }
    pub(super) fn validate_alias(
        &mut self,
        owner: DefId,
        key: NodeKey,
        ty: TypeId,
    ) -> Result<(), Error> {
        let Some(parameters) = self.facts.generics.parameters.get(&owner) else {
            return Ok(());
        };
        self.parameter_uses
            .collect(&self.facts.types, ty)
            .map_err(|e| self.error(key, e))?;
        for parameter in parameters {
            if let Argument::Type(ty) = parameter.argument
                && let Some(Type::Parameter(id)) = self.facts.types.kind(ty)
                && !self.parameter_uses.parameters.contains(&id)
            {
                return Err(self.error(key, "unused type parameter in alias"));
            }
        }
        // Rust defers underlying well-formedness until a type is required in a
        // signature/body. Even an unused alias with an unsized array is accepted.
        Ok(())
    }
}
