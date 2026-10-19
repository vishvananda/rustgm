//! Generic declaration environments are elaborated once by the declaration worklist.
mod annotations;
mod arguments;
pub(super) mod bindings;
pub(super) mod bounds;
pub(super) use crate::sema::generics::captures::Captures;
mod constants;
mod default_validation;
mod defaults;
pub(super) use defaults::{DefaultId, Defaults};
mod functions;
mod validation;
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
    pub(super) fn declaration_parameters(
        &mut self,
        owner: DefId,
        key: NodeKey,
        context: regions::Context,
    ) -> Result<(), Error> {
        let item = self.item(key)?;
        let function = matches!(item.kind, crate::syntax::item::ItemKind::Function { .. });
        let implementation = matches!(item.kind, crate::syntax::item::ItemKind::Impl { .. });
        let family = matches!(item.kind, crate::syntax::item::ItemKind::TypeAlias { .. })
            && self.facts.associated.owner(owner).is_some();
        if family && self.facts.generics.parameters.contains_key(&owner) {
            return Ok(());
        }
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
        self.prepare_bounds(owner, key, &nodes, context)?;
        if !(nodes.iter().any(|key| {
            self.generic_syntax(*key)
                .is_ok_and(|p| p.flavor != GenericFlavor::Lifetime)
        }) || (!nodes.is_empty()
            && (family || matches!(item.kind, crate::syntax::item::ItemKind::Trait { .. }))))
        {
            if !function && self.bound_syntax.contains_key(&owner) {
                self.deferred_bounds.push((owner, context));
            }
            return Ok(());
        }
        if self.facts.destructors.method(owner).is_some() {
            return Err(self.error(
                key,
                "type/const parameters and lifetime bounds require later semantic assignments",
            ));
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
            if function && parameter.default.is_some() {
                return Err(self.error(*node, "function generic parameters cannot have defaults"));
            }
            if implementation && parameter.default.is_some() {
                return Err(self.error(*node, "impl generic parameters cannot have defaults"));
            }
            if family && parameter.default.is_some() {
                return Err(self.error(*node, "associated type parameters cannot have defaults"));
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
        if !function && self.bound_syntax.contains_key(&owner) {
            self.deferred_bounds.push((owner, context));
        }
        for (index, key) in nodes.into_iter().enumerate().rev() {
            // Function lifetime slots are assigned by signature first use, then
            // classified as early/late. Do not preassign them in source order.
            if function && self.generic_syntax(key)?.flavor == GenericFlavor::Lifetime {
                continue;
            }
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
        let binding = self.generic_binding(key)?;
        let argument = if syntax.flavor == GenericFlavor::Lifetime {
            Argument::Lifetime(
                self.regions
                    .named(context.binder, binding)
                    .map_err(|e| self.error(key, e))?,
            )
        } else {
            let kind = if syntax.flavor == GenericFlavor::Type {
                ParameterKind::Type {
                    sized: !self.relaxed.contains(&binding),
                }
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
        if syntax.default.is_some() {
            self.defaults
                .insert(binding, declaration)
                .map_err(|e| self.error(key, e))?;
        }
        Ok(())
    }
    pub(super) fn validate_alias(
        &mut self,
        owner: DefId,
        key: NodeKey,
        ty: TypeId,
    ) -> Result<(), Error> {
        if self.facts.associated.owner(owner).is_some() {
            return Ok(());
        }
        let Some(parameters) = self.facts.generics.parameters.get(&owner) else {
            return Ok(());
        };
        self.parameter_uses
            .collect(
                &self.facts.types,
                self.facts.unreduced.definition(owner).unwrap_or(ty),
            )
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
