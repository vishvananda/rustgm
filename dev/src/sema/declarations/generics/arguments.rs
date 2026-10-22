//! Instantiate alias paths from resolved argument syntax and completed parameter facts.
use super::{Argument, Checker, DefId, Error, Kind, NodeKey, TypeId, regions};
use crate::{
    syntax::ast::{List, NodeId},
    types::{GenericArgs, ItemId, substitute::Bound},
};
impl Checker<'_> {
    pub(in super::super) fn supplied_arguments(
        &self,
        key: NodeKey,
        owner: DefId,
    ) -> Result<(Option<List>, usize), Error> {
        let parameters = self.facts.generics.parameters.get(&owner).ok_or_else(|| {
            self.error(
                key,
                "type arguments require prepared parameter declarations",
            )
        })?;
        let list = if let Some(positional) = self.associated_bindings.positional(key) {
            Some(positional)
        } else if let Some(binding) = self.report.resolution.associated_bindings.get(&key) {
            binding.arguments
        } else {
            crate::sema::lifetimes::arguments(self.report, key, false)?
        };
        let ast = &self.report.package.unit(key.file).configured.ast;
        let supplied = list.map_or(&[][..], |list| ast.list(list));
        let lifetime_count = parameters
            .iter()
            .take_while(|p| matches!(p.argument, Argument::Lifetime(_)))
            .count();
        let explicit_lifetimes = supplied
            .iter()
            .take_while(|node| {
                self.lifetime_argument(NodeKey {
                    node: **node,
                    ..key
                })
            })
            .count();
        let omitted = if explicit_lifetimes == 0 {
            lifetime_count
        } else {
            0
        };
        if explicit_lifetimes != 0 && explicit_lifetimes != lifetime_count {
            return Err(self.error(key, "incorrect number of lifetime arguments"));
        }
        if supplied.len() + omitted > parameters.len() {
            return Err(self.error(key, "too many generic arguments"));
        }
        if parameters
            .iter()
            .skip(supplied.len() + omitted)
            .any(|p| p.default.is_none() && self.defaults.get(p.binding).is_none())
        {
            return Err(self.error(key, "missing required generic argument"));
        }
        Ok((list, omitted))
    }
    pub(in super::super) fn lifetime_argument(&self, key: NodeKey) -> bool {
        matches!(self.kind(key), Kind::Name(span, _) if self.report.package.unit(key.file).spelling(span).is_ok_and(|name| name.starts_with('\'')))
    }
    pub(in super::super) fn path_arguments(
        &mut self,
        key: NodeKey,
        owner: DefId,
        context: regions::Context,
    ) -> Result<(), Error> {
        if matches!(
            self.report.resolution.builtin_traits.identify(owner),
            Some(crate::resolve::model::BuiltinTrait::Callable(_))
        ) {
            return self.schedule_call_arguments(key, context);
        }
        if !self.facts.generics.parameters.contains_key(&owner)
            || self.report.resolution.definitions[owner.0 as usize].kind
                == crate::resolve::model::DefKind::SelfType
        {
            return Ok(());
        }
        let (list, omitted) = self.supplied_arguments(key, owner)?;
        let supplied = list.map_or(0, |list| {
            self.report
                .package
                .unit(key.file)
                .configured
                .ast
                .list(list)
                .len()
        });
        for parameter in self.facts.generics.parameters[&owner]
            .iter()
            .skip(supplied + omitted)
            .filter(|parameter| parameter.default.is_none())
            .rev()
        {
            if let Some(id) = self.defaults.get(parameter.binding) {
                self.work.push(super::Work::Default(id, false));
            }
        }
        let Some(list) = list else { return Ok(()) };
        let count = self
            .report
            .package
            .unit(key.file)
            .configured
            .ast
            .list(list)
            .len();
        for index in (0..count).rev() {
            let node = self.report.package.unit(key.file).configured.ast.list(list)[index];
            if self
                .facts
                .partial_annotations
                .contains_key(&NodeKey { node, ..key })
            {
                continue;
            }
            let parameter = self.facts.generics.parameters[&owner][index + omitted];
            match parameter.argument {
                Argument::Type(_) => {
                    self.object_argument_default(
                        key,
                        owner,
                        parameter.binding,
                        NodeKey { node, ..key },
                        context,
                    )?;
                    self.node_work(key.file, node, context);
                }
                Argument::Const(id) => {
                    self.schedule_constant_argument(NodeKey { node, ..key }, id, owner, context)?;
                }
                Argument::Lifetime(_) => {}
            }
        }
        Ok(())
    }
    pub(in super::super) fn application_arguments(
        &mut self,
        key: NodeKey,
        owner: DefId,
        context: regions::Context,
    ) -> Result<GenericArgs, Error> {
        if matches!(
            self.report.resolution.builtin_traits.identify(owner),
            Some(crate::resolve::model::BuiltinTrait::Callable(_))
        ) {
            return self.call_application_arguments(key);
        }
        let (supplied, omitted) = self.supplied_arguments(key, owner)?;
        let count = self.facts.generics.parameters[&owner].len();
        let mut arguments = Vec::with_capacity(count);
        let mut defaults = None;
        let mut originals = super::super::unreduced::Arguments::default();
        let mut required_defaults = Vec::new();
        for index in 0..count {
            let parameter = self.facts.generics.parameters[&owner][index];
            let ast = &self.report.package.unit(key.file).configured.ast;
            let node = index
                .checked_sub(omitted)
                .and_then(|i| supplied.and_then(|list| ast.list(list).get(i).copied()));
            let (argument, original) =
                if let Some(region) = self.object_application_lifetime(key, index) {
                    (Argument::Lifetime(region), Argument::Lifetime(region))
                } else if let Some(node) = node {
                    let node = NodeKey { node, ..key };
                    let argument = self.source_argument(node, parameter.argument, context)?;
                    let original = if matches!(argument, Argument::Type(_)) {
                        self.facts
                            .unreduced
                            .node(node)
                            .map_or(argument, Argument::Type)
                    } else {
                        argument
                    };
                    (argument, original)
                } else if index < omitted {
                    let region = self.region(key, None, context)?;
                    self.regions.record(context, region);
                    (Argument::Lifetime(region), Argument::Lifetime(region))
                } else {
                    let default = parameter
                        .default
                        .ok_or_else(|| self.error(key, "missing generic default"))?;
                    required_defaults.push(parameter.binding);
                    if defaults.is_none() {
                        let mut prefix = Bound::prefix(ItemId(owner.0));
                        for &argument in &arguments {
                            prefix
                                .push_argument(&self.facts.types, argument)
                                .map_err(|e| self.error(key, e))?;
                        }
                        defaults = Some(prefix);
                    }
                    let argument = defaults
                        .as_mut()
                        .ok_or_else(|| self.error(key, "missing default substitution"))?
                        .apply_argument(&mut self.facts.types, default)
                        .map_err(|e| self.error(key, e))?;
                    let template = self
                        .facts
                        .unreduced
                        .default_type(parameter.binding)
                        .map_or(default, Argument::Type);
                    let original = originals
                        .apply_default(
                            &mut self.facts.types,
                            ItemId(owner.0),
                            &arguments,
                            template,
                            argument,
                        )
                        .map_err(|e| self.error(key, e))?;
                    (argument, original)
                };
            originals
                .push(&self.facts.types, &arguments, argument, original)
                .map_err(|e| self.error(key, e))?;
            if let Some(prefix) = &mut defaults {
                prefix
                    .push_argument(&self.facts.types, argument)
                    .map_err(|e| self.error(key, e))?;
            }
            arguments.push(argument);
        }
        let args: GenericArgs = self
            .facts
            .types
            .intern_arguments(&arguments)
            .map_err(|e| self.error(key, e))?;
        if let Some(prefix) = defaults {
            self.facts.generics.retain_arguments(owner, args, prefix);
        }
        if let Some(originals) = originals
            .finish(&mut self.facts.types)
            .map_err(|e| self.error(key, e))?
        {
            self.facts.unreduced.retain_arguments(key, originals);
        }
        for binding in required_defaults {
            self.alias_use(key, binding, args, context)?;
        }
        Ok(args)
    }
    pub(in super::super) fn generic_path(
        &mut self,
        key: NodeKey,
        owner: DefId,
        context: regions::Context,
    ) -> Result<TypeId, Error> {
        let args = self.application_arguments(key, owner, context)?;
        let template = self.facts.definitions[owner.0 as usize]
            .ok_or_else(|| self.error(key, "missing alias template"))?;
        let result = self
            .facts
            .generics
            .substitute(&mut self.facts.types, owner, args, template)
            .map_err(|e| self.error(key, e))?;
        let original_template = self.facts.unreduced.definition(owner);
        let original_args = self.facts.unreduced.arguments(key);
        if original_template.is_some() || original_args.is_some() {
            let original = self
                .facts
                .generics
                .substitute(
                    &mut self.facts.types,
                    owner,
                    original_args.unwrap_or(args),
                    original_template.unwrap_or(template),
                )
                .map_err(|e| self.error(key, e))?;
            if self.facts.types.contains_projections(original) == Some(true) {
                self.facts.unreduced.retain_node(key, original);
            }
        }
        self.alias_path_use(key, owner, args, context)?;
        Ok(result)
    }
    fn source_argument(
        &mut self,
        key: NodeKey,
        parameter: Argument,
        context: regions::Context,
    ) -> Result<Argument, Error> {
        Ok(match parameter {
            Argument::Lifetime(_) => {
                if !self.lifetime_argument(key) {
                    return Err(self.error(key, "expected lifetime argument"));
                }
                let region = self.region(key, Some(key.node), context)?;
                self.regions.record(context, region);
                Argument::Lifetime(region)
            }
            Argument::Type(_) => Argument::Type(self.ty(key)?),
            Argument::Const(id) => {
                let ty = self
                    .facts
                    .types
                    .const_type(id)
                    .ok_or_else(|| self.error(key, "missing const argument type"))?;
                Argument::Const(self.generic_constant(key, ty)?)
            }
        })
    }
    pub(in super::super) fn array_type(
        &mut self,
        key: NodeKey,
        element: TypeId,
        node: NodeId,
    ) -> Result<crate::types::Kind, Error> {
        let length_key = NodeKey { node, ..key };
        if let Some(length) = self.const_parameter(length_key) {
            return Ok(crate::types::Kind::ArrayTerm { element, length });
        }
        Ok(crate::types::Kind::Array {
            element,
            length: if let Some(&length) = self.facts.array_lengths.get(&length_key) {
                length
            } else {
                super::super::array_length(self.report, length_key)?
            },
        })
    }
}
