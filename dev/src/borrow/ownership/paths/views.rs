//! Resolve internal view aliases into canonical physical paths before numbering.
use super::{
    Access, Action, Body, Component, LocalId, LocalKind, PathId, Paths, Place, PlaceInfo, Root,
    Store, arrays,
};
use std::collections::{HashMap, HashSet};

struct Context<'a> {
    types: &'a Store,
    fields: &'a mut HashMap<(PathId, Component), PathId>,
    arrays: &'a mut arrays::Arrays,
    remaining: usize,
    suffix: Vec<(Component, crate::types::TypeId)>,
}
impl Context<'_> {
    fn charge(&mut self, cost: usize) -> Result<(), &'static str> {
        self.remaining = self
            .remaining
            .checked_sub(cost)
            .ok_or("guard alias work budget exceeded")?;
        Ok(())
    }
}
impl Paths {
    pub(super) fn guard_views(
        &mut self,
        body: &Body,
        types: &Store,
        fields: &mut HashMap<(PathId, Component), PathId>,
        arrays: &mut arrays::Arrays,
    ) -> Result<(), &'static str> {
        let mut sources: HashMap<LocalId, Vec<Place>> = HashMap::new();
        for statement in &body.statements {
            if let Action::BindView { view, place } = statement.action {
                if let Place::Temporary(id) = place {
                    self.temporary(body, id)?;
                }
                sources.entry(view).or_default().push(place);
            }
        }
        let mut order: Vec<_> = sources.keys().copied().collect();
        order.sort_unstable_by_key(|id| id.0);
        let mut context = Context {
            types,
            fields,
            arrays,
            remaining: 1_000_000,
            suffix: Vec::new(),
        };
        for view in order {
            let mut targets = Vec::new();
            let mut seen = HashSet::new();
            for source in &sources[&view] {
                let info = self.place(*source)?;
                // Enclosing views precede their nested bindings. Enforce that
                // order instead of accepting cyclic or speculative aliases.
                if self
                    .view_root(body, info.path)
                    .is_some_and(|base| base.0 >= view.0)
                {
                    return Err("guard view alias does not precede its binding");
                }
                for target in self.expand_view(body, info, &mut context)? {
                    if seen.insert(target) {
                        targets.push(target);
                    }
                }
            }
            context.charge(targets.len())?;
            self.views.insert(view, targets);
        }
        for &place in &body.guard_places {
            if let Place::Temporary(id) = place {
                self.temporary(body, id)?;
            }
            let info = self.place(place)?;
            let targets = self.expand_view(body, info, &mut context)?;
            self.guard_places.push(targets);
        }
        Ok(())
    }
    fn expand_view(
        &mut self,
        body: &Body,
        info: PlaceInfo,
        context: &mut Context<'_>,
    ) -> Result<Vec<PlaceInfo>, &'static str> {
        context.charge(1)?;
        let Some(base_view) = self.view_root(body, info.path) else {
            return Ok(vec![info]);
        };
        context.suffix.clear();
        let mut path = info.path;
        while let Some(id) = path {
            context.charge(1)?;
            let entry = &self.entries[id.0 as usize];
            if let Some(component) = entry.component {
                context.suffix.push((component, entry.ty));
            }
            path = entry.parent;
        }
        let bases = self
            .views
            .get(&base_view)
            .ok_or("missing enclosing guard view")?;
        context.charge(bases.len())?;
        let mut targets = bases.clone();
        for target in &mut targets {
            context.charge(context.suffix.len())?;
            for &(component, ty) in context.suffix.iter().rev() {
                if !target.indexed
                    && target.access == Access::Owned
                    && let Some(parent) = target.path
                {
                    target.path = Some(self.component(
                        context.types,
                        context.fields,
                        context.arrays,
                        parent,
                        component,
                        ty,
                    )?);
                }
            }
            target.empty |= info.empty;
            target.indexed |= info.indexed;
            if info.access != Access::Owned {
                target.access = if info.access == Access::Shared
                    || target.access == Access::Shared
                    || target.path.is_none()
                {
                    Access::Shared
                } else {
                    Access::Mutable
                };
            }
        }
        Ok(targets)
    }
    /// Empty rests read no elements, but their internal references still require
    /// and protect the containing array's shallow state. Indexed origins retain
    /// the ordinary conservative whole-array path.
    pub(in crate::borrow::ownership) fn guard_target(
        &self,
        info: PlaceInfo,
    ) -> (Option<PathId>, bool) {
        let shallow = info.empty && !info.indexed && info.access == Access::Owned;
        let path = info.path.map(|id| {
            if shallow {
                self.entries[id.0 as usize].parent.unwrap_or(id)
            } else {
                id
            }
        });
        (path, shallow)
    }
    pub(in crate::borrow::ownership) fn view_root(
        &self,
        body: &Body,
        path: Option<PathId>,
    ) -> Option<LocalId> {
        let Root::Local(local) = self.entries.get(path?.0 as usize)?.root else {
            return None;
        };
        matches!(body.locals.get(local.0 as usize)?.kind, LocalKind::Guard(_)).then_some(local)
    }
}
