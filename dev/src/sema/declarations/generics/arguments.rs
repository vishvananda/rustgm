//! Instantiate alias paths from resolved argument syntax and completed parameter facts.
use super::{Argument, Checker, DefId, Error, Kind, NodeKey, TypeId, regions};
use crate::{
    syntax::ast::{List, NodeId},
    types::GenericArgs,
};
impl Checker<'_> {
    fn supplied_arguments(
        &self,
        key: NodeKey,
        owner: DefId,
    ) -> Result<(Option<List>, usize), Error> {
        let parameters = &self.facts.generics.parameters[&owner];
        let list = crate::sema::lifetimes::arguments(self.report, key, false)?;
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
            .any(|p| p.default.is_none())
        {
            return Err(self.error(key, "missing required generic argument"));
        }
        Ok((list, omitted))
    }
    pub(super) fn lifetime_argument(&self, key: NodeKey) -> bool {
        matches!(self.kind(key), Kind::Name(span) if self.report.package.unit(key.file).spelling(span).is_ok_and(|name| name.starts_with('\'')))
    }
    pub(in super::super) fn path_arguments(
        &mut self,
        key: NodeKey,
        owner: DefId,
        context: regions::Context,
    ) -> Result<(), Error> {
        if !self.facts.generics.parameters.contains_key(&owner) {
            return Ok(());
        }
        let (list, omitted) = self.supplied_arguments(key, owner)?;
        let Some(list) = list else { return Ok(()) };
        let ast = &self.report.package.unit(key.file).configured.ast;
        for (index, node) in ast.list(list).iter().copied().enumerate().rev() {
            match self.facts.generics.parameters[&owner][index + omitted].argument {
                Argument::Type(_) => self.node_work(key.file, node, context),
                Argument::Const(_) => self.constant_dependencies(NodeKey { node, ..key })?,
                Argument::Lifetime(_) => {}
            }
        }
        Ok(())
    }
    pub(in super::super) fn generic_path(
        &mut self,
        key: NodeKey,
        owner: DefId,
        context: regions::Context,
    ) -> Result<TypeId, Error> {
        let (supplied, omitted) = self.supplied_arguments(key, owner)?;
        let count = self.facts.generics.parameters[&owner].len();
        let mut arguments = Vec::with_capacity(count);
        for index in 0..count {
            let parameter = self.facts.generics.parameters[&owner][index];
            let node = index.checked_sub(omitted).and_then(|i| {
                supplied.and_then(|list| {
                    self.report
                        .package
                        .unit(key.file)
                        .configured
                        .ast
                        .list(list)
                        .get(i)
                        .copied()
                })
            });
            let argument = if let Some(node) = node {
                self.source_argument(NodeKey { node, ..key }, owner, parameter.argument, context)?
            } else if index < omitted {
                let region = self.region(key, None, context)?;
                self.regions.record(context, region);
                Argument::Lifetime(region)
            } else {
                let default = parameter
                    .default
                    .ok_or_else(|| self.error(key, "missing generic default"))?;
                let prefix = self
                    .facts
                    .types
                    .intern_arguments(&arguments)
                    .map_err(|e| self.error(key, e))?;
                self.facts
                    .generics
                    .default_argument(&mut self.facts.types, owner, prefix, default)
                    .map_err(|e| self.error(key, e))?
            };
            arguments.push(argument);
        }
        let args: GenericArgs = self
            .facts
            .types
            .intern_arguments(&arguments)
            .map_err(|e| self.error(key, e))?;
        let template = self.facts.definitions[owner.0 as usize]
            .ok_or_else(|| self.error(key, "missing alias template"))?;
        let result = self
            .facts
            .generics
            .substitute(&mut self.facts.types, owner, args, template)
            .map_err(|e| self.error(key, e))?;
        Ok(result)
    }
    fn source_argument(
        &mut self,
        key: NodeKey,
        owner: DefId,
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
                Argument::Const(self.generic_constant(key, ty, owner)?)
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
            length: super::super::array_length(self.report, length_key)?,
        })
    }
}
