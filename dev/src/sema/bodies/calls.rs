//! Callable instantiation separates item arguments, late binders and caller universals.
use super::{Checker, Constructor, DefId, Error, NodeId, TermId, Type, TypeId};
use crate::{
    sema::model::{BodyRegionOrigin, Outlives},
    types::{GenericArgs, RegionId, infer, substitute::Bound},
};
use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub(super) struct CallTypes {
    pub obligations: Vec<(NodeId, crate::types::infer::predicates::Predicate)>,
    pub assumed_obligations: HashSet<usize>,
    pub generic: Vec<(NodeId, DefId, TermId)>,
    pub generic_constructors: bool,
    pub active: HashMap<NodeId, TermId>,
    pub method_owners: HashMap<NodeId, (DefId, TermId)>,
    pub methods: Vec<(NodeId, super::dot_calls::Pending)>,
    pub nominal_receivers: HashMap<TermId, super::nominals::Fields>,
    pub nominal_order: Vec<TermId>,
    signatures: HashMap<TermId, TypeId>,
    universal_regions: Vec<RegionId>,
    universals: Option<Bound>,
}
impl CallTypes {
    pub fn insert_nominal(&mut self, term: TermId, fields: super::nominals::Fields) {
        if self.nominal_receivers.insert(term, fields).is_none() {
            self.nominal_order.push(term);
        }
    }
    pub fn universal_arguments(&self) -> Vec<RegionId> {
        self.universal_regions.clone()
    }
    pub fn clear(&mut self) {
        self.obligations.clear();
        self.assumed_obligations.clear();
        self.generic.clear();
        self.generic_constructors = false;
        self.active.clear();
        self.method_owners.clear();
        self.methods.clear();
        self.nominal_receivers.clear();
        self.nominal_order.clear();
        self.signatures.clear();
        self.universal_regions.clear();
        self.universals = None;
    }
}

impl Checker<'_> {
    pub(super) fn callable_shape(
        &mut self,
        node: NodeId,
        actual: TermId,
        expected: TermId,
    ) -> Result<(), Error> {
        let generic = !self.call_types.generic.is_empty()
            || self.call_types.generic_constructors
            || self.projections.active();
        let partial = !self.annotation_checks.is_empty()
            || (generic
                && self
                    .infer
                    .contains_late_callables(&self.facts.types, actual)
                    .map_err(|e| self.error(node, e))?);
        let source = if partial {
            self.annotation_source(node, actual, expected)?
        } else {
            actual
        };
        self.shape_equation(node, source, expected)
    }
    fn annotation_source(
        &mut self,
        node: NodeId,
        actual: TermId,
        expected: TermId,
    ) -> Result<TermId, Error> {
        match self.infer.resolved(&mut self.facts.types, expected) {
            Ok(_) => return Ok(actual),
            Err(infer::Error::Unresolved) => {}
            Err(e) => return Err(self.error(node, e)),
        }
        let ty = match self.infer.resolved(&mut self.facts.types, actual) {
            Ok(ty) => ty,
            Err(infer::Error::Unresolved) => return Ok(actual),
            Err(e) => return Err(self.error(node, e)),
        };
        if self.facts.types.contains_late_callables(ty) != Some(true) {
            return Ok(actual);
        }
        let ty = self.alias_type(node, ty)?;
        let first = u32::try_from(self.facts.body_regions.len())
            .ok()
            .and_then(|n| n.checked_add(1))
            .ok_or_else(|| self.error(node, "body region identity space exhausted"))?;
        let specialized = self
            .infer
            .specialize_shape(&mut self.facts.types, ty, expected, first)
            .map_err(|e| self.error(node, e))?;
        let region = crate::sema::model::BodyRegion {
            body: self.body,
            source: self.key(node),
            origin: BodyRegionOrigin::Inference,
        };
        self.facts
            .body_regions
            .extend(std::iter::repeat_n(region, specialized.regions as usize));
        self.known(node, specialized.ty)
    }
    pub(super) fn prepare_universals(&mut self, node: NodeId) -> Result<(), Error> {
        if self.call_types.universals.is_some() {
            return Ok(());
        }
        for index in 0..self.body_binders {
            let bound = RegionId::bound(
                0,
                u16::try_from(index)
                    .map_err(|_| self.error(node, "too many body lifetime parameters"))?,
            )
            .map_err(|e| self.error(node, e))?;
            let alias = self.body_region(node, BodyRegionOrigin::UniversalAlias)?;
            self.call_types.universal_regions.push(alias);
            self.infer.region_equalities.push((alias, bound));
        }
        self.call_types.universals = Some(Bound::new(self.call_types.universal_regions.clone()));
        Ok(())
    }
    pub(super) fn alias_region(
        &mut self,
        node: NodeId,
        region: RegionId,
    ) -> Result<RegionId, Error> {
        let Some((depth, index)) = region.bound_parts() else {
            return Ok(region);
        };
        if depth != 0 {
            return Err(self.error(
                node,
                "item lifetime argument is outside the current body binder",
            ));
        }
        self.prepare_universals(node)?;
        self.call_types
            .universal_regions
            .get(usize::from(index))
            .copied()
            .ok_or_else(|| self.error(node, "missing body lifetime parameter"))
    }
    pub(super) fn alias_type(&mut self, node: NodeId, ty: TypeId) -> Result<TypeId, Error> {
        if self.facts.types.escaping_binders(ty) == Some(0) {
            return Ok(ty);
        }
        self.prepare_universals(node)?;
        let Some(substitution) = self.call_types.universals.as_mut() else {
            return Err(self.error(node, "missing universal substitution"));
        };
        substitution
            .apply(&mut self.facts.types, ty)
            .map_err(|e| self.error(node, e))
    }

    pub(super) fn alias_predicate(
        &mut self,
        node: NodeId,
        predicate: crate::types::parameters::predicates::Predicate,
    ) -> Result<crate::types::parameters::predicates::Predicate, Error> {
        self.prepare_universals(node)?;
        let Some(substitution) = self.call_types.universals.as_mut() else {
            return Err(self.error(node, "missing universal predicate substitution"));
        };
        substitution
            .predicate(&mut self.facts.types, predicate)
            .map_err(|e| self.error(node, e))
    }

    pub(super) fn call_signature(&mut self, node: NodeId, term: TermId) -> Result<TermId, Error> {
        let Some(Constructor::Function { bound_regions, .. }) = self.head(node, term)? else {
            return Err(self.error(node, "call requires a known function type"));
        };
        let root = self.infer.root(term).map_err(|e| self.error(node, e))?;
        let ty = if let Some(ty) = self.call_types.signatures.get(&root) {
            *ty
        } else {
            let ty = match self.infer.resolved(&mut self.facts.types, root) {
                Ok(ty) => ty,
                // Without a late binder, argument checking can retain the graph.
                // Otherwise instantiate its regions without solving type holes.
                Err(infer::Error::Unresolved) if bound_regions == 0 => return Ok(term),
                Err(infer::Error::Unresolved) => return self.instantiate_call_term(node, term),
                Err(error) => return Err(self.error(node, error)),
            };
            self.call_types.signatures.insert(root, ty);
            ty
        };
        if bound_regions == 0 && self.facts.types.escaping_binders(ty) == Some(0) {
            return Ok(term);
        }
        let signature = self.instantiate_call(node, ty)?;
        self.known(node, signature)
    }
    fn instantiate_call(&mut self, node: NodeId, signature: TypeId) -> Result<TypeId, Error> {
        let Some(Type::Function {
            bound_regions,
            parameters,
            result,
            unsafe_,
        }) = self.facts.types.kind(signature)
        else {
            return Err(self.error(node, "missing callable signature"));
        };
        let mut arguments = Vec::with_capacity(bound_regions as usize);
        for _ in 0..bound_regions {
            arguments.push(self.body_region(node, BodyRegionOrigin::CallArgument)?);
        }
        let mut substitution = Bound::new(arguments);
        let count = self
            .facts
            .types
            .list(parameters)
            .ok_or_else(|| self.error(node, "missing call parameters"))?
            .len();
        let mut children = Vec::with_capacity(count);
        for index in 0..count {
            let ty = self
                .facts
                .types
                .list(parameters)
                .ok_or_else(|| self.error(node, "missing callable parameters"))?[index];
            let ty = substitution
                .apply(&mut self.facts.types, ty)
                .map_err(|e| self.error(node, e))?;
            children.push(self.alias_type(node, ty)?);
        }
        let result = substitution
            .apply(&mut self.facts.types, result)
            .map_err(|e| self.error(node, e))?;
        let result = self.alias_type(node, result)?;
        let parameters = self
            .facts
            .types
            .intern_list(&children)
            .map_err(|e| self.error(node, e))?;
        self.facts
            .types
            .intern(Type::Function {
                bound_regions: 0,
                parameters,
                result,
                unsafe_,
            })
            .map_err(|e| self.error(node, e))
    }

    pub(super) fn function_item(
        &mut self,
        node: NodeId,
        owner: DefId,
        ty: TypeId,
    ) -> Result<TypeId, Error> {
        if self.facts.destructors.method(owner).is_some() {
            return Err(self.error(
                node,
                "explicit destructor calls or function values are forbidden",
            ));
        }
        if let Some(&variant) = self.facts.members.constructors.get(&owner) {
            return self.constructor_item(node, ty, variant);
        }
        let Some(Type::FunctionItem {
            item, signature, ..
        }) = self.facts.types.kind(ty)
        else {
            return Ok(ty);
        };
        let explicit = if self.facts.generics.item_uses.contains_key(&self.key(node)) {
            self.facts
                .generics
                .item_uses
                .get(&self.key(node))
                .and_then(|u| u.lifetimes)
        } else {
            crate::sema::lifetimes::arguments(self.report, self.key(node), false)?
        };
        let Some(scheme) = self.facts.lifetimes.functions.get(&owner) else {
            if explicit.is_some_and(|args| args.start != args.end) {
                return Err(self.error(node, "function does not accept lifetime arguments"));
            }
            return Ok(ty);
        };
        let count = scheme.early.len();
        let captures = scheme.captures as usize;
        if captures != 0
            && self.facts.associated.trait_owner(owner).is_none()
            && !self.facts.methods.qualified.contains_key(&self.key(node))
            && !self.call_types.method_owners.contains_key(&node)
        {
            return Err(self.error(node, "method lifetime captures require a selected owner"));
        }
        let mut mapping = vec![RegionId::STATIC; scheme.total as usize];
        for (ordinal, index) in scheme.late.iter().copied().enumerate() {
            mapping[usize::from(index)] = RegionId::bound(
                0,
                u16::try_from(ordinal).map_err(|_| self.error(node, "too many late lifetimes"))?,
            )
            .map_err(|e| self.error(node, e))?;
        }
        let early: Vec<_> = scheme.early.iter().map(|(_, index)| *index).collect();
        let late = u32::try_from(scheme.late.len())
            .map_err(|_| self.error(node, "too many late lifetime slots"))?;
        let bounds = scheme.bounds.clone();
        let mut arguments = Vec::with_capacity(captures + count);
        let explicit_captures = self
            .facts
            .generics
            .trait_uses
            .get(&self.key(node))
            .and_then(|use_| use_.arguments.as_ref())
            .and_then(|arguments| arguments.lifetimes);
        let owner_regions = self.early_item_arguments(node, explicit_captures, captures)?;
        for (slot, region) in mapping.iter_mut().take(captures).zip(owner_regions) {
            *slot = region;
            arguments.push(region);
        }
        arguments.extend(self.early_item_arguments(node, explicit, count)?);
        for (index, argument) in early
            .into_iter()
            .zip(arguments.iter().copied().skip(captures))
        {
            mapping[usize::from(index)] = argument;
        }
        self.item_bounds(node, &mapping, &bounds)?;
        let arguments = self
            .facts
            .types
            .intern_regions(&arguments)
            .map_err(|e| self.error(node, e))?;
        let signature = if let Some(template) = self.facts.lifetimes.callables.get(item) {
            self.facts
                .lifetimes
                .substitute(&mut self.facts.types, arguments, template.signature)
                .map_err(|e| self.error(node, e))?
        } else {
            Bound::new(mapping)
                .signature(&mut self.facts.types, signature, late)
                .map_err(|e| self.error(node, e))?
        };
        self.facts
            .types
            .intern(Type::FunctionItem {
                item,
                arguments,
                signature,
            })
            .map_err(|e| self.error(node, e))
    }
    fn early_item_arguments(
        &mut self,
        node: NodeId,
        explicit: Option<crate::syntax::ast::List>,
        count: usize,
    ) -> Result<Vec<RegionId>, Error> {
        let mut arguments = Vec::with_capacity(count);
        if let Some(list) = explicit {
            let nodes = self
                .report
                .package
                .unit(self.file)
                .configured
                .ast
                .list(list);
            if nodes.len() != count {
                return Err(self.error(
                    node,
                    "explicit lifetime arguments must match early-bound parameters",
                ));
            }
            for node in nodes {
                let region = self.local_region(*node)?;
                arguments.push(self.alias_region(*node, region)?);
            }
        } else {
            for _ in 0..count {
                arguments.push(self.body_region(node, BodyRegionOrigin::ItemArgument)?);
            }
        }
        Ok(arguments)
    }
    fn item_bounds(
        &mut self,
        node: NodeId,
        mapping: &[RegionId],
        bounds: &[(RegionId, RegionId)],
    ) -> Result<(), Error> {
        for &(longer, shorter) in bounds {
            let replace = |region: RegionId| -> Result<RegionId, Error> {
                if let Some((0, index)) = region.bound_parts() {
                    mapping
                        .get(usize::from(index))
                        .copied()
                        .ok_or_else(|| self.error(node, "invalid item bound parameter"))
                } else {
                    Ok(region)
                }
            };
            let (longer, shorter) = (replace(longer)?, replace(shorter)?);
            if longer != shorter && longer != RegionId::STATIC {
                self.facts.outlives.push(Outlives {
                    body: self.body,
                    source: self.key(node),
                    longer,
                    shorter,
                });
            }
        }
        Ok(())
    }
    fn constructor_item(
        &mut self,
        node: NodeId,
        ty: TypeId,
        variant: crate::sema::members::VariantId,
    ) -> Result<TypeId, Error> {
        let owner = self.facts.members.variants[variant.0 as usize].owner;
        let nominal = self.nominal_type(node, node, owner)?;
        let Some(Type::FunctionItem {
            item, signature, ..
        }) = self.facts.types.kind(ty)
        else {
            return Ok(nominal);
        };
        let Some(Type::Adt(_, arguments)) = self.facts.types.kind(nominal) else {
            return Err(self.error(node, "missing constructor instance"));
        };
        if arguments == GenericArgs::EMPTY {
            return Ok(ty);
        }
        let regions = self
            .facts
            .types
            .region_args(arguments)
            .ok_or_else(|| self.error(node, "missing constructor lifetime arguments"))?
            .to_vec();
        let mut aliases = Vec::with_capacity(regions.len());
        for region in regions {
            aliases.push(self.alias_region(node, region)?);
        }
        let signature = Bound::new(aliases)
            .signature(&mut self.facts.types, signature, 0)
            .map_err(|e| self.error(node, e))?;
        self.facts
            .types
            .intern(Type::FunctionItem {
                item,
                arguments,
                signature,
            })
            .map_err(|e| self.error(node, e))
    }
}
