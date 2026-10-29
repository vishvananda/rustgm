//! Apply completed nominal requirements using the consuming formation policy.
use super::{
    Argument, Bounds, ComponentKind, Projection, RegionId, Scan, Store, TypeId, contracts, free,
};

pub(super) trait Demand {
    fn require(
        &mut self,
        scan: &mut Scan<'_>,
        types: &Store,
        ty: TypeId,
        shorter: RegionId,
        depth: u16,
        kind: ComponentKind,
    ) -> Result<(), &'static str>;

    fn projection(
        &mut self,
        scan: &mut Scan<'_>,
        types: &Store,
        projection: Projection,
        shorter: RegionId,
    ) -> Result<(), &'static str>;
}

pub(super) struct Ordinary;
impl Demand for Ordinary {
    fn require(
        &mut self,
        scan: &mut Scan<'_>,
        types: &Store,
        ty: TypeId,
        shorter: RegionId,
        depth: u16,
        kind: ComponentKind,
    ) -> Result<(), &'static str> {
        scan.component_bound(types, ty, shorter, depth, kind)
    }

    fn projection(
        &mut self,
        scan: &mut Scan<'_>,
        _: &Store,
        projection: Projection,
        shorter: RegionId,
    ) -> Result<(), &'static str> {
        let pair = (projection, shorter);
        if scan.unique_projections.insert(pair) {
            scan.projection_pairs.push(pair);
        }
        Ok(())
    }
}

impl Scan<'_> {
    pub(super) fn demands(
        &mut self,
        types: &Store,
        bounds: &Bounds,
        callables: &crate::types::callable::Templates,
        demand: &mut impl Demand,
    ) -> Result<(), &'static str> {
        for index in 0..self.uses.len() {
            let use_ = self.uses[index];
            let pairs = if use_.callable {
                let template = callables
                    .get(use_.item)
                    .ok_or("missing callable lifetime template")?;
                if types.arguments(use_.arguments).is_none_or(|args| {
                    args.filter(|arg| matches!(arg, Argument::Lifetime(_)))
                        .count()
                        != template.parameters as usize
                }) {
                    return Err("callable lifetime argument arity mismatch");
                }
                callables
                    .bounds(use_.item)
                    .ok_or("missing callable lifetime bounds")?
            } else {
                bounds
                    .get(use_.item)
                    .ok_or("missing nominal outlives facts")?
            };
            for pair in pairs {
                self.budget.charge()?;
                if let Some(pair) = contracts::instantiate(types, use_, *pair)? {
                    self.record(pair)?;
                }
            }
            if !use_.callable {
                for &pair in bounds
                    .parameters(use_.item)
                    .ok_or("missing nominal parameter requirements")?
                {
                    self.parameter_with(types, use_, pair, demand)?;
                }
                for &(projection, shorter) in bounds.instance(use_.item, use_.arguments)? {
                    self.budget.charge()?;
                    if let Some(shorter) = free(shorter, use_.depth)? {
                        demand.projection(
                            self,
                            types,
                            Projection {
                                depth: use_.depth,
                                ..projection
                            },
                            shorter,
                        )?;
                    }
                }
            }
        }
        self.family_demands(types, bounds, demand)
    }
}
