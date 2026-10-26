//! Structural layout readiness is data, never trait or source-call permission.
use super::Layouts;
use crate::{
    support::hash::HashSet,
    types::{
        Kind, Store, TypeId,
        aggregate::{Definitions, ShapeReadiness},
    },
};
impl Layouts {
    /// Prepay dense-state initialization and possible existing-buffer copying.
    /// A prepared layout needs neither this setup nor another ABI demand.
    pub(crate) fn pending_state_work(&self, types: &Store, ty: TypeId) -> usize {
        if self.get(ty).is_some() {
            return 0;
        }
        types
            .len()
            .saturating_sub(self.states.len())
            .saturating_add(if types.len() > self.states.capacity() {
                self.states.capacity()
            } else {
                0
            })
    }
    /// Check prepared fields before querying the stable ABI cache. Pending
    /// declarations must not become permanent failed-layout cache entries.
    /// The caller owns the counter, including a failed attempt's actual work.
    pub(crate) fn storage_readiness(
        &self,
        types: &Store,
        definitions: &Definitions,
        ty: TypeId,
        work: &mut usize,
        limit: usize,
    ) -> Result<(), &'static str> {
        charge(work, limit)?;
        if self.get(ty).is_some() {
            return Ok(());
        }
        let mut pending = vec![ty];
        let mut seen = HashSet::default();
        let mut first = true;
        while let Some(next) = pending.pop() {
            if first {
                first = false;
            } else {
                charge(work, limit)?;
            }
            if self.get(next).is_some() || !seen.insert(next) {
                continue;
            }
            let fields = match types.kind(next).ok_or("missing storage type")? {
                Kind::Tuple(fields) => Some(fields),
                Kind::Pattern { base, .. } => {
                    pending.push(base);
                    None
                }
                Kind::Opaque(..) => {
                    pending.push(types.storage_head(next)?);
                    None
                }
                Kind::Foreign(_) => {
                    return Err("foreign type has unknown value size and alignment");
                }
                Kind::Parameter(_) | Kind::Projection(_) | Kind::ArrayTerm { .. } => {
                    return Err("constant storage layout requires a closed normalized type");
                }
                Kind::Array { element, .. } => {
                    pending.push(element);
                    None
                }
                Kind::Closure { environment, .. } => {
                    pending.push(environment);
                    None
                }
                Kind::Adt(item, _) => {
                    let ShapeReadiness::Ready(shape) = definitions.try_shape(types, next)? else {
                        return Err("constant storage layout requires completed fields");
                    };
                    if shape.definition.kind == crate::types::aggregate::Kind::Enum
                        && !definitions.discriminants_ready(item)
                    {
                        return Err(
                            "constant storage layout requires completed enum discriminants",
                        );
                    }
                    for variant in shape.variants() {
                        let fields = types.list(variant.fields).ok_or("missing storage fields")?;
                        if work
                            .saturating_add(pending.len())
                            .saturating_add(fields.len())
                            > limit
                        {
                            return Err("constant storage layout work budget exceeded");
                        }
                        pending.extend(fields.iter().copied());
                    }
                    None
                }
                _ => None,
            };
            if let Some(fields) = fields {
                let fields = types.list(fields).ok_or("missing storage fields")?;
                if work
                    .saturating_add(pending.len())
                    .saturating_add(fields.len())
                    > limit
                {
                    return Err("constant storage layout work budget exceeded");
                }
                pending.extend(fields.iter().copied());
            }
        }
        Ok(())
    }
}
fn charge(work: &mut usize, limit: usize) -> Result<(), &'static str> {
    let Some(next) = work.checked_add(1) else {
        return Err("constant storage layout work budget exceeded");
    };
    *work = next;
    if *work > limit {
        return Err("constant storage layout work budget exceeded");
    }
    Ok(())
}
