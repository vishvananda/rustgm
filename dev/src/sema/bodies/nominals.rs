//! Nominal uses retain shared inference variables across constructors and fields.
#[cfg(test)]
#[path = "../../../tests/invariants/returned_fields.rs"]
mod tests;
use super::{Checker, Constructor, DefId, Error, NodeId, TermId, Type, TypeId};
use crate::{
    sema::{
        generics::SourceArgument,
        members::{FieldId, VariantId},
        model::BodyRegionOrigin,
    },
    types::{
        ItemId,
        arguments::{Argument as Canonical, ConstKind},
        infer::{Instantiation, arguments::Argument},
        substitute::Bound,
    },
};

pub(super) use crate::sema::annotations::Fields;
impl Checker<'_> {
    pub(super) fn nominal_term(
        &mut self,
        node: NodeId,
        path: NodeId,
        owner: ItemId,
        expected: Option<TermId>,
    ) -> Result<TermId, Error> {
        if let Some(&use_) = self.facts.generics.constructor_uses.get(&self.key(path)) {
            return match use_ {
                crate::sema::generics::ConstructorUse::Prefix(prefix) => {
                    self.annotation(prefix.node)
                }
                crate::sema::generics::ConstructorUse::SelfType(ty) => self.known(node, ty),
            };
        }
        if let Some(super::Target::Definition(id)) =
            self.facts.node(self.key(path)).and_then(|f| f.reference)
            && self.report.resolution.definitions[id.0 as usize].kind
                == crate::resolve::model::DefKind::TypeAlias
        {
            return self.annotation(path);
        }
        let Some(parameters) = self.facts.generics.parameters.get(&DefId(owner.0)).cloned() else {
            let ty = self.nominal_type(node, path, owner)?;
            return self.known(node, ty);
        };
        let written = self
            .facts
            .generics
            .item_uses
            .get(&self.key(path))
            .ok_or_else(|| self.error(node, "nominal arguments were not elaborated"))?;
        let supplied = written.arguments.clone();
        let lifetimes = written.lifetimes;
        let count = parameters
            .iter()
            .take_while(|p| matches!(p.argument, Canonical::Lifetime(_)))
            .count();
        let mut regions = Vec::with_capacity(count);
        if let Some(list) = lifetimes {
            let nodes = self
                .report
                .package
                .unit(self.state.file)
                .configured
                .ast
                .list(list);
            if nodes.len() != count {
                return Err(self.error(node, "incorrect number of lifetime arguments"));
            }
            for &argument in nodes {
                let region = self.local_region(argument)?;
                regions.push(self.alias_region(argument, region)?);
            }
        } else if let Some(context) =
            self.constructor_regions(node, owner, expected.filter(|_| count != 0))?
        {
            if context.len() != count {
                return Err(self.error(node, "incorrect contextual lifetime argument count"));
            }
            regions = context;
        } else {
            for _ in 0..count {
                regions.push(self.body_region(node, BodyRegionOrigin::ItemArgument)?);
            }
        }
        let mut arguments = regions
            .iter()
            .copied()
            .map(Argument::Lifetime)
            .collect::<Vec<_>>();
        let mut bindings = Vec::with_capacity(parameters.len() - count);
        for (index, parameter) in parameters.iter().skip(count).enumerate() {
            let source = supplied
                .get(index)
                .copied()
                .unwrap_or(SourceArgument::Infer);
            let binding = self.inferred_argument(node, parameter.argument, source)?;
            arguments.push(binding.1);
            bindings.push(binding);
        }
        let mut fields = Fields {
            regions: Bound::new(regions),
            parameters: Instantiation::pooled(
                &mut self.state.infer,
                &self.facts.types,
                &bindings,
                &mut self.state.call_types.instantiations,
            )
            .map_err(|e| self.error(node, e))?,
        };
        if !supplied.is_empty() {
            for index in count + supplied.len()..parameters.len() {
                let default = parameters[index]
                    .default
                    .ok_or_else(|| self.error(node, "missing generic default"))?;
                fields.default(self, self.key(node), default, arguments[index])?;
            }
        }
        let term = self
            .state
            .infer
            .nominal(owner, &arguments)
            .map_err(|e| self.error(node, e))?;
        self.state.call_types.insert_nominal(term, fields);
        Ok(term)
    }

    /// A known expected nominal supplies its own input arguments. In particular,
    /// opaque associated output equality must not infer these lifetime inputs.
    fn constructor_regions(
        &mut self,
        node: NodeId,
        owner: ItemId,
        expected: Option<TermId>,
    ) -> Result<Option<Vec<crate::types::RegionId>>, Error> {
        let Some(expected) = expected.filter(|_| self.facts.aggregates.has_projected_templates())
        else {
            return Ok(None);
        };
        if !matches!(
            self.contextual_head(node, expected)?,
            Some(Constructor::Adt(item, _) | Constructor::AdtArgs(item, _)) if item == owner
        ) {
            return Ok(None);
        }
        let arguments = self
            .state
            .infer
            .item_arguments(&self.facts.types, expected)
            .map_err(|e| self.error(node, e))?;
        let mut regions = Vec::new();
        for argument in arguments {
            if let Argument::Lifetime(region) = argument {
                regions.push(self.alias_region(node, region)?);
            }
        }
        Ok(Some(regions))
    }

    pub(super) fn generic_constructor(
        &mut self,
        node: NodeId,
        ty: TypeId,
        variant: VariantId,
    ) -> Result<TermId, Error> {
        let variant = self.facts.members.variants[variant.0 as usize];
        let nominal = self.nominal_term(node, node, variant.owner, None)?;
        let Some(Type::FunctionItem { item, .. }) = self.facts.types.kind(ty) else {
            return Ok(nominal);
        };
        self.state.call_types.generic_constructors = true;
        self.facts.generics.referenced_constructors.insert(item);
        let mut children = Vec::with_capacity((variant.end - variant.start) as usize + 1);
        for index in variant.start..variant.end {
            children.push(self.instance_field(node, nominal, FieldId(index))?);
        }
        children.push(nominal);
        let signature = self
            .state
            .infer
            .construct(
                Constructor::Function {
                    bound_regions: 0,
                    parameters: variant.end - variant.start,
                    unsafe_: false,
                    convention: crate::types::callable::Convention::RUST,
                },
                &children,
            )
            .map_err(|e| self.error(node, e))?;
        let arguments = self
            .state
            .infer
            .item_arguments(&self.facts.types, nominal)
            .map_err(|e| self.error(node, e))?;
        self.state
            .infer
            .function_item(item, signature, &arguments)
            .map_err(|e| self.error(node, e))
    }
    pub(super) fn inferred_field(
        &mut self,
        node: NodeId,
        receiver: TermId,
        owner: ItemId,
        template: TypeId,
    ) -> Result<TermId, Error> {
        let root = self
            .state
            .infer
            .root(receiver)
            .map_err(|e| self.error(node, e))?;
        if !self.state.call_types.nominal_receivers.contains_key(&root) {
            let fields = self.receiver_fields(node, root, owner)?;
            self.state.call_types.insert_nominal(root, fields);
        }
        let key = self.key(node);
        let report = self.report;
        let fields = self
            .state
            .call_types
            .nominal_receivers
            .get_mut(&root)
            .ok_or_else(|| {
                super::Declarations::error(report, key, "missing nominal inference environment")
            })?;
        let ty = fields
            .regions
            .apply(&mut self.facts.types, template)
            .map_err(|e| super::Declarations::error(report, key, e))?;
        let term = self
            .state
            .infer
            .instantiate_with(&mut self.facts.types, ty, &mut fields.parameters)
            .map_err(|e| self.error(node, e))?;
        if self.facts.types.contains_projections(template) == Some(true)
            || self.state.projections.active()
        {
            self.defer_normalization(node, crate::types::lifetime_proof::ScopeId::ROOT, term)?;
        }
        Ok(term)
    }
    fn receiver_fields(
        &mut self,
        node: NodeId,
        receiver: TermId,
        owner: ItemId,
    ) -> Result<Fields, Error> {
        if !matches!(self.head(node, receiver)?,
            Some(Constructor::Adt(item, _) | Constructor::AdtArgs(item, _)) if item == owner)
        {
            return Err(self.error(node, "field receiver belongs to another nominal owner"));
        }
        let arguments = self
            .state
            .infer
            .item_arguments(&self.facts.types, receiver)
            .map_err(|e| self.error(node, e))?;
        let Some(parameters) = self.facts.generics.parameters.get(&DefId(owner.0)) else {
            let count = self
                .facts
                .lifetimes
                .parameters
                .get(&DefId(owner.0))
                .map_or(0, Vec::len);
            if arguments.len() != count {
                return Err(self.error(node, "invalid nominal lifetime argument count"));
            }
            let regions = arguments
                .into_iter()
                .map(|argument| match argument {
                    Argument::Lifetime(region) => Ok(region),
                    _ => Err(self.error(node, "nominal lifetime argument kind mismatch")),
                })
                .collect::<Result<Vec<_>, _>>()?;
            return Ok(Fields {
                regions: Bound::new(regions),
                parameters: Instantiation::new(&mut self.state.infer, &self.facts.types, &[])
                    .map_err(|e| self.error(node, e))?,
            });
        };
        if parameters.len() != arguments.len() {
            return Err(self.error(node, "invalid nominal argument count"));
        }
        let mut regions = Vec::new();
        let mut bindings = Vec::new();
        for (parameter, argument) in parameters.iter().zip(arguments) {
            let id = match (parameter.argument, argument) {
                (Canonical::Lifetime(_), Argument::Lifetime(region)) => {
                    regions.push(region);
                    continue;
                }
                (Canonical::Type(ty), Argument::Type(_)) => {
                    let Some(Type::Parameter(id)) = self.facts.types.kind(ty) else {
                        return Err(self.error(node, "missing nominal type parameter"));
                    };
                    id
                }
                (Canonical::Const(id), Argument::Const(_)) => {
                    let Some(ConstKind::Parameter(id)) = self.facts.types.const_kind(id) else {
                        return Err(self.error(node, "missing nominal const parameter"));
                    };
                    id
                }
                _ => return Err(self.error(node, "nominal argument kind mismatch")),
            };
            bindings.push((id, argument));
        }
        Ok(Fields {
            regions: Bound::new(regions),
            parameters: Instantiation::new(&mut self.state.infer, &self.facts.types, &bindings)
                .map_err(|e| self.error(node, e))?,
        })
    }
}
