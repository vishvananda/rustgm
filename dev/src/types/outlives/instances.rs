//! Prepare canonical projected nominal requirements before immutable region proofs.
use super::{
    Bounds, Budget, GenericArgs, HashMap, HashSet, ItemId, Kind, Projection, ProjectionPair, Scan,
    Store, TypeId, Use, children,
};
use crate::types::substitute::Bound;

#[derive(Default)]
pub(super) struct Mapping {
    captures: super::TypeComponents,
    lowering: HashMap<u16, Bound>,
    substitutions: HashMap<(ItemId, GenericArgs), Bound>,
}
impl Mapping {
    pub fn lift(
        &mut self,
        types: &mut Store,
        projection: Projection,
        budget: &mut Budget,
    ) -> Result<Option<Projection>, &'static str> {
        if projection.depth == 0 {
            return Ok(Some(projection));
        }
        let before = self.captures.steps();
        let captured = self.captures.captures(types, projection);
        budget.work(self.captures.steps().saturating_sub(before))?;
        if captured? {
            return Ok(None);
        }
        let fold = self
            .lowering
            .entry(projection.depth)
            .or_insert_with(|| Bound::lower(projection.depth));
        let before = fold.steps;
        let result = fold.apply(types, projection.ty);
        budget.work(fold.steps.saturating_sub(before))?;
        Ok(Some(Projection {
            ty: result?,
            depth: 0,
        }))
    }
    pub fn apply(
        &mut self,
        types: &mut Store,
        use_: Use,
        projection: Projection,
        budget: &mut Budget,
    ) -> Result<Projection, &'static str> {
        if projection.depth != 0 {
            return Err("nominal projected requirement was not lifted");
        }
        let key = (use_.item, use_.arguments);
        let fold = match self.substitutions.entry(key) {
            std::collections::hash_map::Entry::Occupied(entry) => entry.into_mut(),
            std::collections::hash_map::Entry::Vacant(entry) => {
                budget.work(
                    types
                        .arguments(use_.arguments)
                        .ok_or("invalid nominal arguments")?
                        .len(),
                )?;
                entry.insert(Bound::joint(types, use_.item, use_.arguments)?)
            }
        };
        let before = fold.steps;
        let result = fold.apply(types, projection.ty);
        budget.work(fold.steps.saturating_sub(before))?;
        Ok(Projection {
            ty: result?,
            depth: use_.depth,
        })
    }
}

#[derive(Default)]
pub(super) struct Projected {
    pub templates: HashMap<ItemId, Vec<ProjectionPair>>,
    instances: HashMap<(ItemId, GenericArgs), Vec<ProjectionPair>>,
    prepared: HashSet<TypeId>,
    mapping: Mapping,
    scan: Scan,
    work: Vec<(TypeId, bool)>,
}
impl Bounds {
    #[must_use]
    pub fn projections(&self, item: ItemId) -> &[ProjectionPair] {
        self.projected
            .as_ref()
            .and_then(|p| p.templates.get(&item))
            .map_or(&[], Vec::as_slice)
    }
    /// Prepare only requested structural roots, never expanding nominal fields.
    /// Completed templates and successful mappings remain immutable afterwards.
    /// Ordinary nominal contracts allocate no projected preparation state.
    /// # Errors
    /// Rejects malformed canonical inputs and exhausted substitution/traversal work.
    pub fn prepare(&mut self, types: &mut Store, root: TypeId) -> Result<(), &'static str> {
        let Some(projected) = &mut self.projected else {
            return Ok(());
        };
        projected.prepare(types, root)
    }
    #[must_use]
    pub fn preparation_steps(&self) -> usize {
        self.projected.as_ref().map_or(0, |p| p.scan.budget.0)
    }
    pub(super) fn instance(
        &self,
        item: ItemId,
        arguments: GenericArgs,
    ) -> Result<&[ProjectionPair], &'static str> {
        let Some(projected) = &self.projected else {
            return Ok(&[]);
        };
        if !projected.templates.contains_key(&item) {
            return Ok(&[]);
        }
        projected
            .instances
            .get(&(item, arguments))
            .map(Vec::as_slice)
            .ok_or("nominal projected requirements were not prepared")
    }
}
impl Projected {
    fn prepare(&mut self, types: &mut Store, root: TypeId) -> Result<(), &'static str> {
        if self.prepared.contains(&root) {
            return Ok(());
        }
        self.work.clear();
        self.work.push((root, false));
        while let Some((ty, finish)) = self.work.pop() {
            self.scan.budget.charge()?;
            if self.prepared.contains(&ty) {
                continue;
            }
            if finish {
                self.prepared.insert(ty);
                continue;
            }
            let kind = types
                .kind(ty)
                .ok_or("invalid nominal requirement preparation type")?;
            if let Kind::Adt(item, arguments) = kind {
                self.instantiate(types, item, arguments)?;
            }
            self.work.push((ty, true));
            self.scan.work.clear();
            if let Kind::Projection(application) = kind {
                self.scan.projected_inputs(types, application, 0)?;
            } else {
                children(types, ty, 0, &mut self.scan.work, &mut self.scan.budget)?;
            }
            self.scan
                .budget
                .pending(self.work.len().saturating_add(self.scan.work.len()))?;
            self.work
                .extend(self.scan.work.iter().map(|&(ty, _)| (ty, false)));
        }
        Ok(())
    }
    fn instantiate(
        &mut self,
        types: &mut Store,
        item: ItemId,
        arguments: GenericArgs,
    ) -> Result<(), &'static str> {
        let key = (item, arguments);
        if self.instances.contains_key(&key) {
            return Ok(());
        }
        let Some(templates) = self.templates.get(&item) else {
            return Ok(());
        };
        self.scan.budget.work(templates.len())?;
        let mut mapped = Vec::with_capacity(templates.len());
        let use_ = Use {
            item,
            arguments,
            depth: 0,
            callable: false,
        };
        for &(projection, shorter) in templates {
            let projection = self
                .mapping
                .apply(types, use_, projection, &mut self.scan.budget)?;
            let shorter = super::contracts::region(types, use_, shorter)?
                .ok_or("nominal lifetime escaped its root")?;
            mapped.push((projection, shorter));
        }
        self.instances.insert(key, mapped);
        Ok(())
    }
}
