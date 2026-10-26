//! A caller-budget entry shares the ordinary shape fold without changing its state.
use super::{Action, Bound, Key, Kind, Store, TypeId};
use crate::types::arguments::Argument;
#[derive(Clone, Copy)]
pub(super) struct Budget {
    bytes: usize,
    steps: usize,
}
impl Budget {
    pub(super) const fn new(bytes: usize, steps: usize) -> Self {
        Self { bytes, steps }
    }
    pub(super) fn tightened(self, bytes: usize) -> Self {
        Self {
            bytes: self.bytes.min(bytes),
            ..self
        }
    }
    pub(super) const fn ordinary() -> Self {
        Self {
            bytes: usize::MAX,
            steps: usize::MAX,
        }
    }
    pub(super) fn check_steps(&self, steps: usize) -> Result<(), &'static str> {
        if steps > self.steps {
            Err("bounded shape fold work budget exceeded")
        } else {
            Ok(())
        }
    }
    pub(super) fn check_bytes(&self, bytes: usize) -> Result<(), &'static str> {
        if bytes > self.bytes {
            Err("bounded shape fold storage budget exceeded")
        } else {
            Ok(())
        }
    }
    pub(super) fn prepare(
        &self,
        fold: &mut Bound,
        store: &Store,
        kind: Kind,
        finish: bool,
    ) -> Result<(), &'static str> {
        let needs = needs(store, kind)?;
        self.check_steps(fold.steps.saturating_add(needs.arguments))?;
        let extra = if finish {
            crate::support::retained::new_map::<Key, TypeId>(fold.completed.len().saturating_add(1))
                .saturating_add(growth::<TypeId>(needs.list))
                .saturating_add(growth::<super::RegionId>(needs.arguments))
                .saturating_add(growth::<Argument>(needs.arguments))
        } else {
            growth::<(Key, bool)>(
                fold.work
                    .len()
                    .saturating_add(needs.edges)
                    .saturating_add(1),
            )
        };
        self.check_bytes(fold.budget_retained_bytes()?.saturating_add(extra))?;
        if finish {
            fold.completed
                .try_reserve(1)
                .map_err(|_| "bounded shape cache allocation failed")?;
            fold.children
                .try_reserve(needs.list.saturating_sub(fold.children.len()))
                .map_err(|_| "bounded shape field scratch allocation failed")?;
            fold.regions
                .try_reserve(needs.arguments.saturating_sub(fold.regions.len()))
                .map_err(|_| "bounded shape region scratch allocation failed")?;
            fold.generic
                .try_reserve(needs.arguments.saturating_sub(fold.generic.len()))
                .map_err(|_| "bounded shape argument scratch allocation failed")?;
        } else {
            fold.work
                .try_reserve(needs.edges.saturating_add(1))
                .map_err(|_| "bounded shape pending allocation failed")?;
        }
        Ok(())
    }
}
pub(super) fn growth<T>(length: usize) -> usize {
    length
        .saturating_add(4)
        .saturating_mul(4)
        .saturating_mul(size_of::<T>())
}
struct Needs {
    edges: usize,
    list: usize,
    arguments: usize,
}
fn needs(store: &Store, kind: Kind) -> Result<Needs, &'static str> {
    let mut result = Needs {
        edges: 0,
        list: 0,
        arguments: 0,
    };
    let args = |id| -> Result<usize, &'static str> {
        Ok(store
            .arguments(id)
            .ok_or("invalid bounded shape arguments")?
            .len())
    };
    let list = match kind {
        Kind::Tuple(list) => Some(list),
        Kind::Function { parameters, .. } => {
            result.edges = 1;
            Some(parameters)
        }
        Kind::Array { .. }
        | Kind::ArrayTerm { .. }
        | Kind::Slice(_)
        | Kind::Reference { .. }
        | Kind::Pattern { .. }
        | Kind::Pointer { .. } => {
            result.edges = 1;
            None
        }
        Kind::FunctionItem { arguments, .. } => {
            result.edges = 1;
            result.arguments = args(arguments)?;
            None
        }
        Kind::Closure { arguments, .. } => {
            result.edges = 2;
            result.arguments = args(arguments)?;
            None
        }
        Kind::Adt(_, arguments) | Kind::Opaque(_, arguments) | Kind::Dynamic { arguments, .. } => {
            result.arguments = args(arguments)?;
            None
        }
        Kind::Projection(application) => {
            let parent = store
                .trait_application(application.trait_application)
                .ok_or("invalid bounded shape projection parent")?;
            result.edges = 1;
            result.arguments = args(parent.arguments)?.saturating_add(args(application.arguments)?);
            None
        }
        Kind::Never | Kind::Foreign(_) | Kind::Primitive(_) | Kind::Parameter(_) => None,
    };
    if let Some(list) = list {
        result.list = store.list(list).ok_or("invalid bounded shape list")?.len();
    }
    result.edges = result
        .edges
        .saturating_add(result.arguments)
        .saturating_add(result.list);
    Ok(result)
}
/// Growth payment is separate from actual traversal-step reconciliation. Store
/// interning retains canonical data under its existing arena limits.
pub(super) fn charge_prepare(
    fold: &Bound,
    store: &Store,
    kind: Kind,
    finish: bool,
    charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
) -> Result<(), &'static str> {
    use crate::support::retained::Growth;
    let needs = needs(store, kind)?;
    let mut growth = Growth::default();
    if finish || matches!(kind, Kind::Parameter(_)) {
        growth.include(Growth::map::<Key, TypeId>(
            fold.completed.len(),
            fold.completed.capacity(),
            1,
        ));
        growth.include(Growth::vector::<TypeId>(
            fold.children.len(),
            fold.children.capacity(),
            needs.list,
        ));
        growth.include(Growth::vector::<super::RegionId>(
            fold.regions.len(),
            fold.regions.capacity(),
            needs.arguments,
        ));
        growth.include(Growth::vector::<Argument>(
            fold.generic.len(),
            fold.generic.capacity(),
            needs.arguments,
        ));
    } else {
        growth.include(Growth::vector::<(Key, bool)>(
            fold.work.len(),
            fold.work.capacity(),
            needs.edges.saturating_add(1),
        ));
    }
    charge(growth.work, growth.bytes)
}

#[cfg(test)]
#[path = "../../../tests/invariants/bounded_shapes.rs"]
mod tests;
impl Bound {
    /// Budget only this fold's owned state and scratch; canonical Store storage is separate.
    /// This entry accepts only a fresh no-assumption runtime shape environment.
    #[allow(
        dead_code,
        reason = "shared internal prerequisite; production budget owners land in separate checkpoints"
    )]
    pub(crate) fn apply_bounded(
        &mut self,
        store: &mut Store,
        ty: TypeId,
        work_left: usize,
        bytes: usize,
    ) -> Result<TypeId, &'static str> {
        let retained = self.shape_retained_bytes()?;
        let budget = Budget {
            bytes,
            steps: self.steps.saturating_add(work_left),
        };
        budget.check_bytes(retained)?;
        let key = (ty, 0);
        if let Some(value) = self.result(store, key)? {
            return Ok(value);
        }
        budget.check_bytes(
            retained.saturating_add(growth::<(Key, bool)>(self.work.len().saturating_add(1))),
        )?;
        self.work
            .try_reserve(1)
            .map_err(|_| "bounded shape initial allocation failed")?;
        self.work.push((key, false));
        let result = self
            .drain::<true>(store, &budget)
            .and_then(|()| self.result(store, key)?.ok_or("missing bounded shape root"));
        self.work.clear();
        result
    }
    pub(super) fn budget_retained_bytes(&self) -> Result<usize, &'static str> {
        if self.region_budget()?.is_some() {
            self.region_retained_bytes()
        } else {
            self.shape_retained_bytes()
        }
    }
    pub(crate) fn shape_retained_bytes(&self) -> Result<usize, &'static str> {
        if !matches!(
            self.action,
            Action::Shape {
                preserve_static: false
            }
        ) || self.parameters.is_some()
            || self.applications.is_some()
            || !self.arguments.is_empty()
        {
            return Err("bounded shape fold requires a no-assumption shape environment");
        }
        Ok(crate::support::retained::sum([
            crate::support::retained::vector(&self.arguments),
            crate::support::retained::map(&self.completed).saturating_mul(4),
            crate::support::retained::vector(&self.work),
            crate::support::retained::vector(&self.children),
            crate::support::retained::vector(&self.regions),
            crate::support::retained::vector(&self.generic),
            size_of::<Self>(),
        ]))
    }
}
