//! Structural layout readiness is data, never trait or source-call permission.
use super::Layouts;
use crate::{
    support::hash::HashSet,
    types::{
        Kind, Store, TypeId,
        aggregate::{Definitions, ShapeReadiness},
    },
};

pub(crate) const PENDING_STORAGE_FIELDS: &str = "constant storage layout requires completed fields";
pub(crate) const STORAGE_WORK_LIMIT: &str = "constant storage layout work budget exceeded";

/// Resume at the exact nominal whose structural field instance is missing.
/// The caller may supply those fields before the walk reaches the ABI cache.
pub(crate) struct StorageReadiness {
    root: Option<TypeId>,
    pending: Vec<TypeId>,
    seen: HashSet<TypeId>,
}

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
        if StorageReadiness::new(ty)
            .advance(self, types, definitions, work, limit)?
            .is_some()
        {
            return Err(PENDING_STORAGE_FIELDS);
        }
        Ok(())
    }
}
impl StorageReadiness {
    pub(crate) fn new(ty: TypeId) -> Self {
        Self {
            root: Some(ty),
            pending: Vec::new(),
            seen: HashSet::default(),
        }
    }

    pub(crate) fn advance(
        &mut self,
        layouts: &Layouts,
        types: &Store,
        definitions: &Definitions,
        work: &mut usize,
        limit: usize,
    ) -> Result<Option<TypeId>, &'static str> {
        while let Some(next) = self.root.take().or_else(|| self.pending.pop()) {
            charge(work, limit)?;
            if layouts.get(next).is_some() || !self.seen.insert(next) {
                continue;
            }
            let fields = match types.kind(next).ok_or("missing storage type")? {
                Kind::Tuple(fields) => Some(fields),
                Kind::Future {
                    identity,
                    arguments,
                } => Some(types.future_application(identity, arguments)?.storage),
                Kind::Pattern { base, .. } => {
                    self.pending.push(base);
                    None
                }
                Kind::Opaque(..) => {
                    self.pending.push(types.storage_head(next)?);
                    None
                }
                Kind::Foreign(_) => {
                    return Err("foreign type has unknown value size and alignment");
                }
                Kind::Parameter(_) | Kind::Projection(_) | Kind::ArrayTerm { .. } => {
                    return Err("constant storage layout requires a closed normalized type");
                }
                Kind::Array { element, .. } => {
                    self.pending.push(element);
                    None
                }
                Kind::Closure { environment, .. } => {
                    self.pending.push(environment);
                    None
                }
                Kind::Adt(item, _) => {
                    let ShapeReadiness::Ready(shape) = definitions.try_shape(types, next)? else {
                        self.seen.remove(&next);
                        self.root = Some(next);
                        return Ok(Some(next));
                    };
                    if shape.definition.kind == crate::types::aggregate::Kind::Enum
                        && !definitions.discriminants_ready(item)
                    {
                        return Err(
                            "constant storage layout requires completed enum discriminants",
                        );
                    }
                    for variant in shape.variants() {
                        self.push_fields(types, variant.fields, *work, limit)?;
                    }
                    None
                }
                _ => None,
            };
            if let Some(fields) = fields {
                self.push_fields(types, fields, *work, limit)?;
            }
        }
        Ok(None)
    }

    fn push_fields(
        &mut self,
        types: &Store,
        fields: crate::types::ListId,
        work: usize,
        limit: usize,
    ) -> Result<(), &'static str> {
        let fields = types.list(fields).ok_or("missing storage fields")?;
        if work
            .saturating_add(self.pending.len())
            .saturating_add(fields.len())
            > limit
        {
            return Err(STORAGE_WORK_LIMIT);
        }
        self.pending.extend_from_slice(fields);
        Ok(())
    }
}
fn charge(work: &mut usize, limit: usize) -> Result<(), &'static str> {
    let Some(next) = work.checked_add(1) else {
        return Err(STORAGE_WORK_LIMIT);
    };
    *work = next;
    if *work > limit {
        return Err(STORAGE_WORK_LIMIT);
    }
    Ok(())
}
