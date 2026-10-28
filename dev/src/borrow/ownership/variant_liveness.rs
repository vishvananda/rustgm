//! Fresh variant decisions exclude inactive payloads from possible cleanup only.
use super::initialization::Maybe;
use super::{PathId, Paths, paths::Access, state::State};
use crate::{
    body::{
        drops::{MemberPosition, MemberQuery},
        ir::{Action, BlockId, Body, Rvalue, Terminator},
    },
    support::hash::HashMap,
    types::{Kind, Store, TypeId},
};

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
}
impl Refinements {
    pub(super) fn new(
        body: &Body,
        paths: &Paths,
        types: &Store,
        position: &impl Fn(TypeId, MemberQuery) -> Result<MemberPosition, &'static str>,
        byte_limit: usize,
    ) -> Result<Self, &'static str> {
        let mut result = Self {
            rules: HashMap::default(),
            groups: HashMap::default(),
            ranges: Vec::new(),
            steps: 0,
            byte_limit,
        };
        result.index_rules(body, paths, types, position)?;
        if result.rules.is_empty() {
            return Ok(result);
        }
        result.index_fields(paths, types, position)?;
        Ok(result)
    }
    fn index_rules(
        &mut self,
        body: &Body,
        paths: &Paths,
        types: &Store,
        position: &impl Fn(TypeId, MemberQuery) -> Result<MemberPosition, &'static str>,
    ) -> Result<(), &'static str> {
        self.charge(body.blocks.len())?;
        for (index, block) in body.blocks.iter().enumerate() {
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
        Ok(())
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
        self.rules.is_empty()
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
