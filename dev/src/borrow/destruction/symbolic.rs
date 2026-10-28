//! Possible cleanup accesses under fixed assumptions, without executable recipes.
#[cfg(test)]
#[path = "../../../tests/invariants/matched_payloads.rs"]
mod tests;
use super::{
    Event,
    flags::Recorder,
    schedule::{Cleanup, Schedule},
};
use crate::support::hash::HashMap;
use crate::{
    body::ir::Selection,
    borrow::ownership::{
        Component, Error, PathId, Paths,
        initialization::{self, Point, Status},
    },
    types::{ItemId, Kind, Store, TypeId, aggregate::Definitions, destruction::NeedsDrop},
};

#[derive(Clone, Copy)]
enum Tracking {
    Path(PathId),
    Bit(u32),
    External,
}
/// Abstract lifetime/access obligations cannot be lowered as native cleanup.
#[derive(Default)]
pub struct Plan {
    cleanups: Vec<Cleanup>,
    events: Vec<Event>,
    pub steps: usize,
}
impl Plan {
    pub(crate) fn schedule(&self) -> Schedule<'_> {
        Schedule::symbolic(&self.events, &self.cleanups)
    }
}
struct Builder<'a, C> {
    types: &'a Store,
    definitions: &'a Definitions,
    needed: &'a mut NeedsDrop,
    custom: C,
    paths: &'a Paths,
    fields: HashMap<(PathId, u32, u32), PathId>,
    work: Vec<(TypeId, Tracking)>,
    plan: Plan,
    retained: usize,
}
/// Reuse proved initialization and sparse move paths to select possible accesses.
/// `position` maps a selected field to its declaration's variant/field ordinals.
/// # Errors
/// Rejects invalid field moves, missing typed identities and exhausted work.
pub fn select(
    types: &Store,
    definitions: &Definitions,
    needed: &mut NeedsDrop,
    facts: &mut initialization::Facts<'_>,
    custom: impl Fn(ItemId) -> bool,
    position: impl Fn(
        TypeId,
        crate::body::drops::MemberQuery,
    ) -> Result<crate::body::drops::MemberPosition, &'static str>,
) -> Result<Plan, Error> {
    let refinement_steps = facts
        .refine_variants(types, &position)
        .map_err(|message| Error {
            node: facts.body().source.into(),
            message,
        })?;
    facts.destructor_moves(types, &custom)?;
    let error = |message| Error {
        node: facts.body().source.into(),
        message,
    };
    let mut builder = Builder {
        types,
        definitions,
        needed,
        custom,
        paths: &facts.ownership.paths,
        fields: HashMap::default(),
        work: Vec::new(),
        plan: Plan::default(),
        retained: 0,
    };
    builder.charge(refinement_steps).map_err(error)?;
    for (index, path) in builder.paths.entries.iter().enumerate() {
        builder.charge(1).map_err(error)?;
        if let (Some(parent), Some(Component::Field(field))) = (path.parent, path.component) {
            let ty = builder.paths.entries[parent.0 as usize].ty;
            let (variant, ordinal) = match (types.kind(ty), field) {
                (Some(Kind::Tuple(_) | Kind::Closure { .. }), Selection::Tuple(ordinal)) => {
                    (0, ordinal)
                }
                (Some(Kind::Adt(_, _)), Selection::Nominal(_)) => {
                    let crate::body::drops::MemberPosition::Field { variant, field } =
                        position(ty, crate::body::drops::MemberQuery::Field(field))
                            .map_err(error)?
                    else {
                        return Err(error("cleanup field query lost its exact coordinates"));
                    };
                    (variant, field)
                }
                _ => return Err(error("invalid abstract cleanup field owner")),
            };
            builder
                .retain(2 * size_of::<((PathId, u32, u32), PathId)>())
                .map_err(error)?;
            let id = PathId(u32::try_from(index).map_err(|_| error("too many cleanup paths"))?);
            if builder
                .fields
                .insert((parent, variant, ordinal), id)
                .is_some()
            {
                return Err(error("duplicate abstract cleanup field"));
            }
        }
    }
    let mut recording = super::flags::Recording::for_body(facts.body(), Vec::new());
    facts
        .visit_transfers(|transition, point| {
            recording.push(transition);
            let initialization::Transition::Observe(event) = transition else {
                return Ok(());
            };
            builder.charge(1)?;
            if let initialization::Event::Cleanup(candidate) = event {
                let body = facts.body();
                let ty = body
                    .place_type(candidate.place)
                    .ok_or("invalid abstract cleanup place")?;
                let tracked = candidate.path.map_or(Tracking::External, Tracking::Path);
                // Initialization is already proved. A moved value has no
                // cleanup access, so it demands no recursive type property.
                let live = builder.status(tracked, point)? != Status::Never
                    && builder.needed.demand(types, definitions, ty)?;
                let access = live && builder.access(ty, tracked, point)?;
                builder.retain(size_of::<Cleanup>())?;
                builder.plan.cleanups.push(Cleanup {
                    candidate,
                    access,
                    live_type: live.then_some(ty),
                });
            }
            Ok(())
        })
        .map_err(error)?;
    Recorder {
        events: &mut builder.plan.events,
        flags: &[],
        steps: &mut builder.plan.steps,
        retained: builder.retained,
    }
    .record_from(
        facts,
        &mut recording,
        builder.plan.cleanups.iter().map(|c| c.candidate),
    )
    .map_err(error)?;
    Ok(builder.plan)
}
impl<C: Fn(ItemId) -> bool> Builder<'_, C> {
    fn charge(&mut self, count: usize) -> Result<(), &'static str> {
        self.plan.steps = self
            .plan
            .steps
            .checked_add(count)
            .filter(|n| *n <= 32_000_000)
            .ok_or("abstract destruction work budget exceeded")?;
        Ok(())
    }
    fn retain(&mut self, bytes: usize) -> Result<(), &'static str> {
        self.retained = self
            .retained
            .checked_add(bytes)
            .filter(|n| *n <= 64_000_000)
            .ok_or("abstract destruction exceeds 64 MB budget")?;
        Ok(())
    }
    fn status(&self, tracked: Tracking, point: &mut Point<'_>) -> Result<Status, &'static str> {
        match tracked {
            Tracking::Path(id) => point.status(
                self.paths
                    .entries
                    .get(id.0 as usize)
                    .ok_or("invalid abstract cleanup path")?,
            ),
            Tracking::Bit(bit) => point.bit(bit),
            Tracking::External => Ok(Status::Always),
        }
    }
    fn push(&mut self, ty: TypeId, tracked: Tracking) -> Result<(), &'static str> {
        self.charge(1)?;
        if self.work.len() >= 1_000_000 {
            return Err("abstract destruction pending work exceeded");
        }
        self.work.push((ty, tracked));
        Ok(())
    }
    fn access(
        &mut self,
        ty: TypeId,
        tracked: Tracking,
        point: &mut Point<'_>,
    ) -> Result<bool, &'static str> {
        self.work.clear();
        self.push(ty, tracked)?;
        while let Some((ty, tracked)) = self.work.pop() {
            self.charge(1)?;
            let status = self.status(tracked, point)?;
            if status == Status::Never || !self.needed.demand(self.types, self.definitions, ty)? {
                continue;
            }
            if status == Status::Always || !matches!(tracked, Tracking::Path(_)) {
                return Ok(true);
            }
            let Tracking::Path(path) = tracked else {
                return Err("missing abstract cleanup path");
            };
            let bit = self.paths.entries[path.0 as usize].start;
            let kind = self.types.kind(ty).ok_or("invalid abstract cleanup type")?;
            if matches!(kind, Kind::Parameter(_))
                || matches!(kind, Kind::Adt(item, _) if (self.custom)(item))
            {
                if point.bit(bit)? == Status::Never {
                    return Err("partly initialized object has no initialized structure");
                }
                return Ok(true);
            }
            match kind {
                Kind::Tuple(list) => self.fields(path, 0, list)?,
                Kind::Closure { environment, .. } => {
                    let Some(Kind::Tuple(list)) = self.types.kind(environment) else {
                        return Err("abstract closure cleanup requires a tuple environment");
                    };
                    self.fields(path, 0, list)?;
                }
                Kind::Adt(_, _) => {
                    for (ordinal, variant) in self
                        .definitions
                        .shape(self.types, ty)?
                        .variants()
                        .enumerate()
                    {
                        self.fields(
                            path,
                            u32::try_from(ordinal).map_err(|_| "too many cleanup variants")?,
                            variant.fields,
                        )?;
                    }
                }
                Kind::Array { element, .. } => {
                    if let Some(segments) = self.paths.array_segments(path) {
                        for segment in segments {
                            self.push(
                                element,
                                segment
                                    .element
                                    .map_or(Tracking::Bit(segment.bit), Tracking::Path),
                            )?;
                        }
                    } else if point.bit(bit)? != Status::Never {
                        return Ok(true);
                    }
                }
                Kind::ArrayTerm { .. } | Kind::Slice(_) => {
                    // Unknown-length elements cannot have independently moved paths yet.
                    if point.bit(bit)? != Status::Never {
                        return Ok(true);
                    }
                }
                _ => return Err("unexpected abstract destructible type"),
            }
        }
        Ok(false)
    }
    fn fields(
        &mut self,
        path: PathId,
        variant: u32,
        list: crate::types::ListId,
    ) -> Result<(), &'static str> {
        let fields = self
            .types
            .list(list)
            .ok_or("invalid abstract cleanup fields")?;
        let shallow = Tracking::Bit(self.paths.entries[path.0 as usize].start);
        for (ordinal, &ty) in fields.iter().enumerate() {
            let ordinal = u32::try_from(ordinal).map_err(|_| "too many cleanup fields")?;
            let tracked = self
                .fields
                .get(&(path, variant, ordinal))
                .map_or(shallow, |p| Tracking::Path(*p));
            self.push(ty, tracked)?;
        }
        Ok(())
    }
}
