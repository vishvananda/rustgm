//! Function lifetime schemes retain source/body slots and callable binding times separately.
use super::{
    BinderId, Checker, DefId, Error, Kind, List, NodeKey, RegionId, Type, TypeId, regions,
};
use crate::{sema::lifetimes::FunctionRegions, types::bound_uses::Uses};
use std::collections::HashSet;

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
            if !self.report.package.unit(key.file).configured.active[node.0 as usize] {
                continue;
            }
            while let Kind::Attributed { value, .. } = ast.node(node).kind {
                node = value;
            }
            let Kind::Generic(index) = ast.node(node).kind else {
                return Err(self.error(key, "invalid lifetime parameter syntax"));
            };
            let parameter = ast.generics[index as usize];
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
                binders,
                ty,
                bounds: list,
            } = ast.node(*node).kind
            else {
                return Err(self.error(key, "invalid lifetime where-clause"));
            };
            if binders.start != binders.end {
                return Err(self.error(
                    key,
                    "higher-ranked where-clauses require advanced generic relationships",
                ));
            }
            let longer = self.bound_region(NodeKey { node: ty, ..key }, binder)?;
            self.lifetime_bounds(key, binder, longer, list, &mut bounds)?;
        }
        Ok(bounds)
    }

    fn bound_region(&mut self, key: NodeKey, binder: BinderId) -> Result<RegionId, Error> {
        let Kind::Name(span) = self.kind(key) else {
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
                binders,
                value,
            } = ast.node(node).kind
            {
                if binders.start != binders.end {
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
        signature: TypeId,
        bounds: Vec<(RegionId, RegionId)>,
    ) -> Result<(), Error> {
        if self.facts.destructors.method(owner).is_some() {
            return self.drop_method_regions(key, owner, binder, declared, signature, &bounds);
        }
        let Type::Function {
            bound_regions,
            parameters,
            result,
            ..
        } = self
            .facts
            .types
            .kind(signature)
            .ok_or_else(|| self.error(key, "missing lifetime signature"))?
        else {
            return Err(self.error(key, "invalid lifetime signature"));
        };
        if bound_regions == 0 && declared.start == declared.end && bounds.is_empty() {
            return Ok(());
        }
        self.function_binders.insert(key, binder);
        let mut inputs = Uses::default();
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
        let bounded: HashSet<_> = bounds
            .iter()
            .flat_map(|(a, b)| [a, b])
            .filter_map(|r| {
                r.bound_parts()
                    .filter(|(depth, _)| *depth == 0)
                    .map(|(_, index)| index)
            })
            .collect();
        // An unused declared lifetime is late, even when an erased alias once
        // mentioned it. An unconstrained lifetime occurring in the output is early.
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
            if bounded.contains(&index)
                || (outputs.indices.contains(&index) && !inputs.indices.contains(&index))
            {
                early.push((definition, index));
            } else {
                has_declared_late = true;
            }
        }
        let early_indices: HashSet<_> = early.iter().map(|(_, index)| *index).collect();
        let mut late: Vec<_> = inputs
            .indices
            .union(&outputs.indices)
            .copied()
            .filter(|index| !early_indices.contains(index))
            .collect();
        late.sort_unstable();
        let scheme = FunctionRegions {
            total: self.regions.count(binder),
            early,
            late,
            has_declared_late,
            bounds,
        };
        self.callable_template(key, owner, signature, &scheme)?;
        self.facts.lifetimes.functions.insert(owner, scheme);
        Ok(())
    }

    fn callable_template(
        &mut self,
        key: NodeKey,
        owner: DefId,
        signature: TypeId,
        scheme: &FunctionRegions,
    ) -> Result<(), Error> {
        if scheme.early.is_empty() {
            return Ok(());
        }
        let mut mapping = vec![RegionId::STATIC; scheme.total as usize];
        for (depth, indices) in [
            (0, scheme.late.clone()),
            (1, scheme.early.iter().map(|(_, index)| *index).collect()),
        ] {
            for (ordinal, index) in indices.into_iter().enumerate() {
                mapping[usize::from(index)] = RegionId::bound(
                    depth,
                    u16::try_from(ordinal)
                        .map_err(|_| self.error(key, "too many callable parameters"))?,
                )
                .map_err(|e| self.error(key, e))?;
            }
        }
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
        let early = u32::try_from(scheme.early.len())
            .map_err(|_| self.error(key, "too many early parameters"))?;
        let signature = crate::types::substitute::Bound::new(mapping)
            .signature(&mut self.facts.types, signature, late)
            .map_err(|e| self.error(key, e))?;
        self.facts
            .lifetimes
            .callables
            .insert(
                &self.facts.types,
                crate::types::ItemId(owner.0),
                signature,
                early,
                &bounds,
            )
            .map_err(|e| self.error(key, e))
    }
}
