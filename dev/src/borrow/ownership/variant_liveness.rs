//! Fresh variant decisions exclude inactive payloads from possible cleanup only.
mod index;
mod locate;
mod parts;
mod picked;
mod qterm;
mod query;
pub(crate) use picked::Selected;
mod tags;
mod trans;
use super::initialization::Maybe;
use super::{PathId, Paths, paths::Access, state::State};
use crate::types::{ItemId, aggregate::Definitions};
use crate::{
    body::{
        drops::{MemberPosition, MemberQuery},
        ir::{Action, BlockId, Body, Rvalue, Terminator},
    },
    support::hash::HashMap,
    types::{Kind, Store, TypeId},
};

#[derive(Clone, Copy)]
pub(super) struct EligibilityInput<'a> {
    pub definitions: &'a Definitions,
    pub custom: &'a dyn Fn(ItemId) -> bool,
    pub requirements: Option<&'a crate::body::drops::Requirements>,
}
#[derive(Clone, Copy)]
struct Rule {
    parent: PathId,
    variant: u32,
    yes: BlockId,
    no: BlockId,
}
pub(super) struct Refinements {
    rules: HashMap<usize, Rule>,
    groups: HashMap<(PathId, u32), (usize, usize)>,
    ranges: Vec<(u32, u32)>,
    pub steps: usize,
    byte_limit: usize,
    partition: Option<Box<index::Partition>>,
    physical: bool,
}
impl Refinements {
    pub(super) fn new(
        body: &Body,
        paths: &Paths,
        types: &Store,
        position: &impl Fn(TypeId, MemberQuery) -> Result<MemberPosition, &'static str>,
        byte_limit: usize,
        eligibility: Option<EligibilityInput<'_>>,
        physical: bool,
    ) -> Result<Self, &'static str> {
        let mut result = Self {
            rules: HashMap::default(),
            groups: HashMap::default(),
            ranges: Vec::new(),
            steps: 0,
            byte_limit,
            partition: None,
            physical,
        };
        let seeded = result.index_rules(
            body,
            paths,
            types,
            position,
            eligibility.map(|input| input.definitions),
            eligibility.and_then(|input| input.requirements),
        )?;
        if physical && !result.rules.is_empty() {
            result.index_fields(paths, types, position)?;
        }
        if seeded
            && let Some(EligibilityInput {
                definitions,
                custom,
                ..
            }) = eligibility
        {
            let partition = index::Partition::new(
                index::Inputs {
                    types,
                    definitions,
                    paths,
                    body,
                    limit: byte_limit,
                },
                custom,
                position,
            )?;
            let prior = result.steps;
            result.charge(partition.tags.parts.steps.get())?;
            // All later capture/substitution work shares this same combined cap.
            partition.tags.parts.charge(prior)?;
            result.partition = Some(Box::new(partition));
        }
        Ok(result)
    }
    fn index_rules(
        &mut self,
        body: &Body,
        paths: &Paths,
        types: &Store,
        position: &impl Fn(TypeId, MemberQuery) -> Result<MemberPosition, &'static str>,
        definitions: Option<&Definitions>,
        requirements: Option<&crate::body::drops::Requirements>,
    ) -> Result<bool, &'static str> {
        let mut seeded = false;
        self.charge(body.blocks.len())?;
        for (index, block) in body.blocks.iter().enumerate() {
            if let Some(definitions) = definitions {
                for statement in &body.statements[block.start as usize..block.end as usize] {
                    self.charge(1)?;
                    let ty = match statement.action {
                        Action::StorageLive(local) => {
                            body.locals.get(local.0 as usize).map(|local| local.ty)
                        }
                        Action::Evaluate(value) => {
                            body.values.get(value.0 as usize).map(|value| value.ty)
                        }
                        _ => None,
                    };
                    seeded|=ty.is_some_and(|ty|matches!(types.kind(ty),Some(Kind::Adt(item,_))
                        if definitions.get(item).is_some_and(|d|d.kind==crate::types::aggregate::Kind::Enum)));
                    if let (Action::StorageLive(id), Some(requirements)) =
                        (statement.action, requirements)
                    {
                        seeded |= body.locals.get(id.0 as usize).is_some_and(|local| {
                            matches!(
                                local.kind,
                                crate::body::ir::LocalKind::Parameter(_)
                                    | crate::body::ir::LocalKind::Variadic
                            ) && requirements.locals.get(id.0 as usize).copied() == Some(true)
                                && structural_type(types, definitions, local.ty)
                        });
                    }
                    if let (Action::Evaluate(value), Some(requirements)) =
                        (statement.action, requirements)
                    {
                        seeded |= structural_seed(types, definitions, body, requirements, value);
                    }
                }
            }
            let Terminator::Branch { condition, yes, no } = block.terminator else {
                continue;
            };
            // A cached boolean can outlive a write or call that changes its input.
            // Require the genuine test to be the final statement at this edge.
            if yes == no
                || block.start == block.end
                || !matches!(body.statements[(block.end - 1) as usize].action,
                Action::Evaluate(value) if value == condition)
            {
                continue;
            }
            let Some(Rvalue::IsVariant { place, variant }) =
                body.values.get(condition.0 as usize).map(|v| v.kind)
            else {
                continue;
            };
            let info = paths.place(place)?;
            let Some(parent) = info
                .path
                .filter(|_| info.access == Access::Owned && !info.indexed && !info.union)
            else {
                continue;
            };
            let ty = paths.entries[parent.0 as usize].ty;
            if !matches!(types.kind(ty), Some(Kind::Adt(_, _))) {
                continue;
            }
            self.charge(4)?;
            let MemberPosition::Variant(variant) = position(ty, MemberQuery::Variant(variant))?
            else {
                return Err("cleanup variant query lost its actual nominal coordinates");
            };
            self.reserve_rule()?;
            self.rules.insert(
                index,
                Rule {
                    parent,
                    variant,
                    yes,
                    no,
                },
            );
        }
        Ok(seeded)
    }
    fn index_fields(
        &mut self,
        paths: &Paths,
        types: &Store,
        position: &impl Fn(TypeId, MemberQuery) -> Result<MemberPosition, &'static str>,
    ) -> Result<(), &'static str> {
        let count = paths.entries.len();
        self.charge(count)?;
        self.check_storage(count)?;
        let mut fields = Vec::new();
        fields
            .try_reserve_exact(count)
            .map_err(|_| "cleanup refinement field allocation failed")?;
        for path in &paths.entries {
            let (Some(parent), Some(super::Component::Field(field))) =
                (path.parent, path.component)
            else {
                continue;
            };
            let ty = paths.entries[parent.0 as usize].ty;
            if !matches!(types.kind(ty), Some(Kind::Adt(_, _))) {
                continue;
            }
            self.charge(2)?;
            let MemberPosition::Field { variant, .. } = position(ty, MemberQuery::Field(field))?
            else {
                return Err("cleanup refinement field query changed its actual coordinates");
            };
            fields.push((parent, variant, path.start, path.end));
        }
        self.charge(
            fields
                .len()
                .saturating_mul(fields.len().max(1).ilog2() as usize + 1),
        )?;
        fields.sort_unstable_by_key(|&(parent, variant, start, _)| (parent.0, variant, start));
        self.ranges
            .try_reserve_exact(fields.len())
            .map_err(|_| "cleanup refinement interval allocation failed")?;
        self.groups
            .try_reserve(fields.len())
            .map_err(|_| "cleanup refinement group allocation failed")?;
        for (parent, variant, start, end) in fields {
            self.charge(1)?;
            let first = self.ranges.len();
            self.ranges.push((start, end));
            self.groups
                .entry((parent, variant))
                .or_insert((first, first))
                .1 = first + 1;
        }
        Ok(())
    }
    fn charge(&mut self, amount: usize) -> Result<(), &'static str> {
        self.steps = self.steps.saturating_add(amount);
        if self.steps > 32_000_000 {
            return Err("cleanup refinement work budget exceeded");
        }
        Ok(())
    }
    fn check_storage(&self, fields: usize) -> Result<(), &'static str> {
        let bytes = size_of::<Self>()
            .saturating_add(crate::support::retained::map(&self.rules))
            .saturating_add(
                fields
                    .saturating_mul(size_of::<(PathId, u32, u32, u32)>() + size_of::<(u32, u32)>()),
            )
            .saturating_add(crate::support::retained::new_map::<
                (PathId, u32),
                (usize, usize),
            >(fields));
        if bytes > self.byte_limit {
            return Err("cleanup refinement exceeds 64 MB budget");
        }
        Ok(())
    }
    fn reserve_rule(&mut self) -> Result<(), &'static str> {
        if crate::support::retained::new_map::<usize, Rule>(self.rules.len() + 1) > self.byte_limit
        {
            return Err("cleanup refinement exceeds 64 MB budget");
        }
        if self.rules.len() == self.rules.capacity() {
            self.charge(self.rules.len())?;
        }
        self.rules
            .try_reserve(1)
            .map_err(|_| "cleanup refinement rule allocation failed")
    }
    pub(super) fn is_empty(&self) -> bool {
        (!self.physical || self.rules.is_empty()) && self.partition.is_none()
    }
    pub(super) fn has(&self, block: usize) -> bool {
        self.rules.contains_key(&block)
    }
    pub(super) fn apply<I: Maybe>(
        &self,
        paths: &Paths,
        block: usize,
        target: BlockId,
        state: &mut State<I>,
    ) -> Result<(), &'static str> {
        state.charge(1)?;
        let Some(rule) = self.rules.get(&block) else {
            return Ok(());
        };
        if let Some(partition) = &self.partition {
            partition.edge(rule.parent, rule.variant, target == rule.yes, state)?;
        }
        if !self.physical {
            return Ok(());
        }
        let ranges = self
            .groups
            .get(&(rule.parent, rule.variant))
            .map_or(&[][..], |&(start, end)| &self.ranges[start..end]);
        if target == rule.no {
            for &(start, end) in ranges {
                state.charge(1)?;
                state.exclude(start, end)?;
            }
        } else if target == rule.yes {
            let parent = &paths.entries[rule.parent.0 as usize];
            let mut cursor = parent.start + 1;
            for &(start, end) in ranges {
                state.charge(1)?;
                state.exclude(cursor, start)?;
                cursor = end;
            }
            state.exclude(cursor, parent.end)?;
        } else {
            return Err("cleanup refinement belongs to another successor");
        }
        Ok(())
    }
}

impl Refinements {
    pub(super) fn bits(&self, physical: u32) -> u32 {
        self.partition.as_ref().map_or(physical, |p| p.tags.bits)
    }
    pub(super) fn scratch(&self) -> Result<Vec<u8>, &'static str> {
        self.partition
            .as_ref()
            .map_or_else(|| Ok(Vec::new()), |p| p.scratch())
    }
    pub(super) fn before<I: Maybe>(
        &self,
        location: crate::body::storage::Location,
        state: &mut State<I>,
        scratch: &mut [u8],
    ) -> Result<(), &'static str> {
        self.partition
            .as_ref()
            .map_or(Ok(()), |p| p.before(location, state, scratch))
    }
    pub(super) fn after<I: Maybe>(
        &self,
        location: crate::body::storage::Location,
        state: &mut State<I>,
        scratch: &[u8],
    ) -> Result<(), &'static str> {
        self.partition
            .as_ref()
            .map_or(Ok(()), |p| p.after(location, state, scratch))
    }
    pub(super) fn transition<I: Maybe>(
        &self,
        event: super::initialization::Transition,
        state: &mut State<I>,
    ) -> Result<(), &'static str> {
        self.partition
            .as_ref()
            .map_or(Ok(()), |p| p.transition(event, state))
    }
}

impl Refinements {
    pub(super) fn visit_selected(
        &self,
        facts: &super::initialization::Facts<'_>,
        mut visitor: impl FnMut(
            super::initialization::Transition,
            &mut super::initialization::Point<'_>,
        ) -> Result<
            Option<crate::borrow::destruction::schedule::Cleanup>,
            &'static str,
        >,
    ) -> Result<Option<Box<Selected>>, &'static str> {
        let Some(partition) = &self.partition else {
            facts.visit_transfers(|event, point| visitor(event, point).map(|_| ()))?;
            return Ok(None);
        };
        let mut selected = Selected::new(&partition.tags, &facts.ownership.paths, facts.body())?;
        facts.visit_transfers(|event, point| {
            let snapshot = if let super::initialization::Transition::Observe(
                super::initialization::Event::Cleanup(candidate),
            ) = event
            {
                Some(selected.snapshot(
                    &partition.tags,
                    &facts.ownership.paths,
                    facts.body(),
                    candidate,
                    point,
                    partition.queries.as_deref(),
                )?)
            } else {
                None
            };
            // Snapshot is captured before the callback can inspect/mutate Point.
            let row = visitor(event, point)?;
            match (snapshot, row) {
                (Some(snapshot), Some(row)) => {
                    selected.complete(&partition.tags.parts, &snapshot, row)
                }
                (None, None) => Ok(()),
                _ => Err("selected replay visitor changed actual cleanup coverage"),
            }
        })?;
        Ok(Some(Box::new(selected)))
    }
    pub(super) fn install_selected(
        &mut self,
        body: &Body,
        selected: Box<Selected>,
    ) -> Result<(), &'static str> {
        let partition = self
            .partition
            .as_mut()
            .ok_or("selected cleanup lost its actual partition")?;
        if partition.selected.is_some()
            || selected.body != body.key()
            || selected.source != body.source
        {
            return Err("selected cleanup owner is stale or belongs to another body");
        }
        partition.selected = Some(selected);
        Ok(())
    }
    pub(super) fn selected(&self) -> Option<&Selected> {
        self.partition.as_ref()?.selected.as_deref()
    }
    pub(super) fn resolve_selected(
        &mut self,
        types: &mut Store,
        definitions: &Definitions,
        body: &Body,
    ) -> Result<(), &'static str> {
        let Some(partition) = &mut self.partition else {
            return Ok(());
        };
        if let Some(selected) = &mut partition.selected {
            selected.resolve(
                types,
                definitions,
                body,
                &partition.tags.parts,
                &partition.transfers.candidates,
                partition
                    .queries
                    .as_deref()
                    .map(|q| q.coordinates.as_slice()),
            )?;
        }
        Ok(())
    }
}

impl Refinements {
    pub(super) fn has_partition(&self) -> bool {
        self.partition.is_some()
    }
    pub(super) fn prepare_queries(
        &mut self,
        types: &mut Store,
        definitions: &Definitions,
        body: &Body,
        paths: &Paths,
    ) -> Result<Option<&[TypeId]>, &'static str> {
        self.partition
            .as_deref_mut()
            .map(|p| p.prepare_queries(types, definitions, body, paths))
            .transpose()
    }
    pub(super) fn accept_queries(
        &mut self,
        body: &Body,
        types: &Store,
        definitions: &Definitions,
        regions: &crate::borrow::regions::Body,
        answers: &crate::body::defining_cleanups::Qualification<'_>,
    ) -> Result<(), &'static str> {
        self.partition
            .as_deref_mut()
            .ok_or("const qualification lost its partition")?
            .accept_queries(body, types, definitions, regions, answers)
    }
    pub(super) fn apply_queries(&mut self) -> Result<(), &'static str> {
        self.partition
            .as_deref_mut()
            .and_then(|p| p.queries.as_deref_mut())
            .ok_or("const qualification refinement was already completed")?
            .apply()
    }
}

fn structural_seed(
    types: &Store,
    definitions: &Definitions,
    body: &Body,
    requirements: &crate::body::drops::Requirements,
    value: crate::body::ir::ValueId,
) -> bool {
    let Some(value) = body
        .values
        .get(value.0 as usize)
        .filter(|_| requirements.values.get(value.0 as usize).copied() == Some(true))
    else {
        return false;
    };
    if !matches!(
        value.kind,
        Rvalue::Aggregate { .. } | Rvalue::Repeat { .. } | Rvalue::Call { .. } | Rvalue::Read(_)
    ) {
        return false;
    }
    structural_type(types, definitions, value.ty)
}
fn structural_type(types: &Store, definitions: &Definitions, ty: TypeId) -> bool {
    match types.kind(ty) {
        Some(Kind::Tuple(_) | Kind::Array { .. } | Kind::ArrayTerm { .. }) => true,
        Some(Kind::Adt(item, _)) => definitions
            .get(item)
            .is_some_and(|d| d.kind == crate::types::aggregate::Kind::Struct),
        _ => false,
    }
}
