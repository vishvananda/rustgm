//! Function lifetime schemes retain source/body slots and callable binding times separately.
use super::{
    BinderId, Checker, DefId, Error, Kind, List, NodeKey, RegionId, Type, TypeId, regions,
};
use crate::support::hash::HashSet;
use crate::{sema::lifetimes::FunctionRegions, types::bound_uses::Uses};

#[derive(Clone, Copy)]
pub(super) struct SignatureTypes {
    pub normal: TypeId,
    pub original: TypeId,
}

impl Checker<'_> {
    pub(super) fn function_bounds(
        &mut self,
        key: NodeKey,
        binder: BinderId,
    ) -> Result<Vec<(RegionId, RegionId)>, Error> {
        let item = self.item(key)?;
        let mut bounds = Vec::new();
        let ast = &self.report.package.unit(key.file).configured.ast;
        for mut node in ast.list(item.generics).iter().copied() {
            if !self.report.package.unit(key.file).configured.active[node.get() as usize] {
                continue;
            }
            while let Kind::Attributed { value, .. } = ast.node(node).kind {
                node = value;
            }
            let Kind::Generic(index) = ast.node(node).kind else {
                return Err(self.error(key, "invalid lifetime parameter syntax"));
            };
            let parameter = ast.generics[index as usize];
            if parameter.flavor != crate::syntax::item::GenericFlavor::Lifetime {
                continue;
            }
            if parameter.bounds.start == parameter.bounds.end {
                continue;
            }
            let definition = self
                .facts
                .node(NodeKey { node, ..key })
                .and_then(|f| f.binding)
                .ok_or_else(|| self.error(key, "missing bounded lifetime parameter"))?;
            let longer = self
                .regions
                .named(Some(binder), definition)
                .map_err(|e| self.error(key, e))?;
            self.lifetime_bounds(key, binder, longer, parameter.bounds, &mut bounds)?;
        }
        for node in ast.list(item.where_clause) {
            let Kind::Where {
                quantified,
                ty,
                bounds: list,
                ..
            } = ast.node(*node).kind
            else {
                return Err(self.error(key, "invalid lifetime where-clause"));
            };
            if !self.lifetime_argument(NodeKey { node: ty, ..key }) {
                // Type clauses are decoded by the generic-bound worklist,
                // which retains their complete lexical quantifiers.
                continue;
            }
            if quantified {
                return Err(self.error(key, "lifetime where-bounds cannot introduce lifetimes"));
            }
            let longer = self.bound_region(NodeKey { node: ty, ..key }, binder)?;
            self.lifetime_bounds(key, binder, longer, list, &mut bounds)?;
        }
        Ok(bounds)
    }

    fn bound_region(&mut self, key: NodeKey, binder: BinderId) -> Result<RegionId, Error> {
        let Kind::Name(span, _) = self.kind(key) else {
            return Err(self.error(key, "type and trait where-bounds require generic semantics"));
        };
        let name = self
            .report
            .package
            .unit(key.file)
            .spelling(span)
            .map_err(|e| self.error(key, e.message))?;
        if name == "'_" || !name.starts_with('\'') {
            return Err(self.error(key, "outlives bounds require named lifetimes"));
        }
        self.region(
            key,
            Some(key.node),
            regions::Context {
                binder: Some(binder),
                position: regions::Position::Output,
            },
        )
    }

    fn lifetime_bounds(
        &mut self,
        key: NodeKey,
        binder: BinderId,
        longer: RegionId,
        list: List,
        bounds: &mut Vec<(RegionId, RegionId)>,
    ) -> Result<(), Error> {
        let ast = &self.report.package.unit(key.file).configured.ast;
        for mut node in ast.list(list).iter().copied() {
            if let Kind::Bound {
                optional: false,
                quantified,
                binders: _,
                value,
            } = ast.node(node).kind
            {
                if quantified {
                    return Err(self.error(key, "outlives bounds cannot introduce lifetimes"));
                }
                node = value;
            }
            bounds.push((longer, self.bound_region(NodeKey { node, ..key }, binder)?));
        }
        Ok(())
    }

    pub(super) fn function_regions(
        &mut self,
        key: NodeKey,
        owner: DefId,
        binder: BinderId,
        declared: List,
        signatures: SignatureTypes,
        mut bounds: Vec<(RegionId, RegionId)>,
    ) -> Result<(), Error> {
        if self.facts.destructors.method(owner).is_some() {
            return self.drop_method_regions(
                key,
                owner,
                binder,
                declared,
                signatures.normal,
                &bounds,
            );
        }
        let captures = self
            .facts
            .associated
            .schema(owner)
            .map_or(0, |schema| schema.captures);
        if let Some(schema) = self.facts.associated.schema(owner) {
            bounds.splice(0..0, schema.bounds.iter().copied());
        }
        let Type::Function { bound_regions, .. } = self
            .facts
            .types
            .kind(signatures.normal)
            .ok_or_else(|| self.error(key, "missing lifetime signature"))?
        else {
            return Err(self.error(key, "invalid lifetime signature"));
        };
        if bound_regions == 0
            && declared.start == declared.end
            && bounds.is_empty()
            && !self.facts.generics.has_parameters(owner)
        {
            return Ok(());
        }
        self.function_binders.insert(key, binder);
        let (inputs, outputs) = self.signature_occurrences(key, signatures.original, true)?;
        let bounded = self.bounded_lifetimes(key, owner, &bounds)?;
        // Projection inputs do not constrain a lifetime. Original output
        // occurrences make it early even when normalization erases the lifetime.
        let declared = self.declared_regions(key, declared)?;
        for definition in &declared {
            self.regions
                .named(Some(binder), *definition)
                .map_err(|e| self.error(key, e))?;
        }
        let mut early = Vec::new();
        let mut has_declared_late = false;
        for (definition, region) in self
            .regions
            .bindings(binder)
            .map_err(|e| self.error(key, e))?
        {
            let (_, index) = region
                .bound_parts()
                .ok_or_else(|| self.error(key, "invalid named lifetime slot"))?;
            if u32::from(index) < captures {
                continue;
            }
            if bounded.contains(&index)
                || (outputs.indices.contains(&index) && !inputs.indices.contains(&index))
            {
                early.push((definition, index));
            } else {
                has_declared_late = true;
            }
        }
        let early_indices: HashSet<_> = early.iter().map(|(_, index)| *index).collect();
        // Late instantiation supplies exactly the slots used by the executable
        // signature, including normalized references from projected inputs.
        let (inputs, outputs) = if signatures.original == signatures.normal {
            (inputs, outputs)
        } else {
            self.signature_occurrences(key, signatures.normal, false)?
        };
        let mut late: Vec<_> = inputs
            .indices
            .union(&outputs.indices)
            .copied()
            .filter(|index| u32::from(*index) >= captures && !early_indices.contains(index))
            .collect();
        late.sort_unstable();
        let scheme = FunctionRegions {
            total: self.regions.count(binder),
            captures,
            early,
            late,
            has_declared_late,
            bounds,
        };
        if !scheme.early.is_empty()
            || captures != 0
            || (self.facts.associated.schema(owner).is_some()
                && self.facts.generics.has_parameters(owner))
        {
            self.callable_template(key, owner, signatures.normal, &scheme)?;
        }
        self.facts.lifetimes.functions.insert(owner, scheme);
        Ok(())
    }

    fn bounded_lifetimes(
        &self,
        key: NodeKey,
        owner: DefId,
        bounds: &[(RegionId, RegionId)],
    ) -> Result<HashSet<u16>, Error> {
        let mut bounded: HashSet<_> = bounds
            .iter()
            .flat_map(|(a, b)| [a, b])
            .filter_map(|r| {
                r.bound_parts()
                    .filter(|(depth, _)| *depth == 0)
                    .map(|(_, index)| index)
            })
            .collect();
        bounded.extend(
            self.facts
                .generics
                .bound_lifetimes(&self.facts.types, &self.facts.unreduced, owner)
                .map_err(|e| self.error(key, e))?,
        );
        Ok(bounded)
    }

    fn signature_occurrences(
        &self,
        key: NodeKey,
        signature: TypeId,
        constraining: bool,
    ) -> Result<(Uses, Uses), Error> {
        let Some(Type::Function {
            parameters, result, ..
        }) = self.facts.types.kind(signature)
        else {
            return Err(self.error(key, "missing lifetime occurrence signature"));
        };
        let mut inputs = if constraining {
            Uses::constraining()
        } else {
            Uses::default()
        };
        for ty in self
            .facts
            .types
            .list(parameters)
            .ok_or_else(|| self.error(key, "missing lifetime parameter types"))?
        {
            inputs
                .collect(&self.facts.types, *ty)
                .map_err(|e| self.error(key, e))?;
        }
        let mut outputs = Uses::default();
        outputs
            .collect(&self.facts.types, result)
            .map_err(|e| self.error(key, e))?;
        Ok((inputs, outputs))
    }

    pub(super) fn callable_template(
        &mut self,
        key: NodeKey,
        owner: DefId,
        signature: TypeId,
        scheme: &FunctionRegions,
    ) -> Result<(), Error> {
        super::comparisons::LibraryContracts {
            report: self.report,
            facts: &mut self.facts,
        }
        .callable_template(key, owner, signature, scheme)
    }
}

impl super::comparisons::LibraryContracts<'_> {
    pub(super) fn callable_template(
        &mut self,
        key: NodeKey,
        owner: DefId,
        signature: TypeId,
        scheme: &FunctionRegions,
    ) -> Result<(), Error> {
        if scheme.early.is_empty()
            && scheme.captures == 0
            && !self.facts.generics.has_parameters(owner)
        {
            return Ok(());
        }
        let mut work = 0;
        let mapping = scheme
            .callable_mapping(&mut work)
            .map_err(|e| self.error(key, e));
        self.facts.body_work.canonical_steps += work;
        let mapping = mapping?;
        let bound = |region: RegionId| -> Result<RegionId, Error> {
            if region == RegionId::STATIC {
                return Ok(region);
            }
            let Some((0, index)) = region.bound_parts() else {
                return Err(self.error(key, "invalid callable bound parameter"));
            };
            let Some((1, early)) = mapping
                .get(usize::from(index))
                .and_then(|r| r.bound_parts())
            else {
                return Err(self.error(key, "callable bounds require early parameters"));
            };
            RegionId::bound(0, early).map_err(|e| self.error(key, e))
        };
        let bounds = scheme
            .bounds
            .iter()
            .map(|(a, b)| Ok((bound(*a)?, bound(*b)?)))
            .collect::<Result<Vec<_>, Error>>()?;
        let late = u32::try_from(scheme.late.len())
            .map_err(|_| self.error(key, "too many late parameters"))?;
        let early = u32::try_from(scheme.captures as usize + scheme.early.len())
            .map_err(|_| self.error(key, "too many early parameters"))?;
        let signature = crate::types::substitute::Bound::new(mapping)
            .signature(&mut self.facts.types, signature, late)
            .map_err(|e| self.error(key, e))?;
        let (owners, parameters) = self
            .facts
            .generics
            .callable_parameters(&self.facts.types, owner)
            .map_err(|e| self.error(key, e))?;
        self.facts
            .lifetimes
            .callables
            .insert_scoped(
                &self.facts.types,
                &owners,
                signature,
                early,
                &bounds,
                &parameters,
            )
            .map_err(|e| self.error(key, e))
    }
}
