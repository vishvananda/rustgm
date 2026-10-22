//! Owned rvalues leave the caller through typed operands, stores and returns.
use super::{
    PathId, Paths,
    initialization::{Maybe, Transition},
    state::{self, State},
};
use crate::support::hash::HashMap;
use crate::{
    body::{
        ir::{Action, BlockId, Body, Rvalue, ValueId},
        storage::Location,
    },
    types::{Kind, Store},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Disposition {
    /// Ownership passes to an aggregate, callee, destination or returned result.
    Transfer,
    /// An ordinary zero-length repeat destroys its evaluated operand immediately.
    Discard,
    /// A constant repeat instantiates elements without owning a prototype object.
    Instantiate,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Moment {
    /// Transfer the result before this ordinal in the body's storage exits.
    Return(u32),
    /// Consume operands as part of the statement, after earlier storage events.
    Statement,
}
pub struct Consumption {
    pub value: ValueId,
    pub path: PathId,
    pub location: Location,
    pub moment: Moment,
    pub disposition: Disposition,
}
#[derive(Default)]
pub struct Effects {
    entries: Vec<Consumption>,
    points: HashMap<Location, (usize, usize)>,
}
impl Effects {
    #[must_use]
    pub fn entries(&self) -> &[Consumption] {
        &self.entries
    }
    #[must_use]
    pub fn at(&self, location: Location) -> &[Consumption] {
        self.points
            .get(&location)
            .map_or(&[], |&(start, end)| &self.entries[start..end])
    }
    pub(super) fn build(body: &Body, types: &Store, paths: &Paths) -> Result<Self, &'static str> {
        let mut result = Self::default();
        let Some(owned) = &body.owned else {
            return Ok(result);
        };
        let mut remaining = 32_000_000_usize;
        for (index, block) in body.blocks.iter().enumerate() {
            let block_id =
                BlockId(u32::try_from(index).map_err(|_| "too many consumption blocks")?);
            for before in block.start..block.end {
                let action = body.statements[before as usize].action;
                let (values, disposition) = operands(body, types, &action)?;
                remaining = remaining
                    .checked_sub(values.len() + 1)
                    .ok_or("owned consumption work budget exceeded")?;
                for &value in values {
                    if owned.creation(value).is_some() {
                        result.insert(
                            paths,
                            value,
                            Location::At {
                                block: block_id,
                                before,
                            },
                            Moment::Statement,
                            disposition,
                        )?;
                    }
                }
            }
        }
        for release in &owned.returned {
            let Location::At { block, before } = release.location else {
                return Err("owned return requires a statement point");
            };
            let block = body
                .blocks
                .get(block.0 as usize)
                .ok_or("invalid owned return block")?;
            if before < block.start
                || before > block.end
                || release.exits_before as usize
                    > body.storage.as_ref().map_or(0, |s| s.exits.len())
            {
                return Err("invalid owned return order");
            }
            result.insert(
                paths,
                release.value,
                release.location,
                Moment::Return(release.exits_before),
                Disposition::Transfer,
            )?;
        }
        let count = result.entries.len();
        remaining
            .checked_sub(count.saturating_mul(count.max(1).ilog2() as usize + 2))
            .ok_or("owned consumption work budget exceeded")?;
        result.entries.sort_by_key(|e| match e.location {
            Location::At { block, before } => (block.0, before, e.moment),
            Location::Edge { block, .. } => (block.0, u32::MAX, e.moment),
        });
        for (index, entry) in result.entries.iter().enumerate() {
            result
                .points
                .entry(entry.location)
                .or_insert((index, index))
                .1 = index + 1;
        }
        Ok(result)
    }
    fn insert(
        &mut self,
        paths: &Paths,
        value: ValueId,
        location: Location,
        moment: Moment,
        disposition: Disposition,
    ) -> Result<(), &'static str> {
        if self.entries.len() >= 1_000_000 {
            return Err("owned consumption exceeds one million effects");
        }
        let path = paths
            .temporary_path(value)
            .ok_or("owned operand lacks an initialization path")?;
        self.entries.push(Consumption {
            value,
            path,
            location,
            moment,
            disposition,
        });
        Ok(())
    }
}
impl Consumption {
    pub(super) fn transfer<I: Maybe>(
        &self,
        paths: &Paths,
        state: &mut State<I>,
        inspect: bool,
        visitor: impl FnMut(Transition, &mut State<I>) -> Result<(), &'static str>,
    ) -> Result<(), &'static str> {
        let path = &paths.entries[self.path.0 as usize];
        if inspect && !state.initialized(path)? {
            return Err("owned operand may be uninitialized or already consumed");
        }
        state::change(state, self.location, path, false, visitor)
    }
}

fn operands<'a>(
    body: &'a Body,
    types: &Store,
    action: &'a Action,
) -> Result<(&'a [ValueId], Disposition), &'static str> {
    let one = |id| (std::slice::from_ref(id), Disposition::Transfer);
    if let Action::Store { value, .. } = action {
        return Ok(one(value));
    }
    let Action::Evaluate(id) = action else {
        return Ok((&[], Disposition::Transfer));
    };
    let value = body
        .values
        .get(id.0 as usize)
        .ok_or("invalid owned evaluation")?;
    match &value.kind {
        Rvalue::Aggregate { fields, .. }
        | Rvalue::Intrinsic {
            arguments: fields, ..
        }
        | Rvalue::Invoke {
            arguments: fields, ..
        }
        | Rvalue::Call {
            arguments: fields, ..
        } => {
            let values = body
                .arguments
                .get(fields.start as usize..fields.end as usize)
                .ok_or("invalid owned operand list")?;
            Ok((values, Disposition::Transfer))
        }
        Rvalue::Constrain(id)
        | Rvalue::Unsize { value: id, .. }
        | Rvalue::Union { value: id, .. } => Ok(one(id)),
        Rvalue::Repeat {
            value: operand,
            mode,
        } => {
            let disposition = match (types.kind(value.ty), *mode) {
                (
                    Some(Kind::Array { .. } | Kind::ArrayTerm { .. }),
                    crate::body::ir::RepeatMode::Constant,
                ) => Disposition::Instantiate,
                (Some(Kind::Array { length: 0, .. }), _) => Disposition::Discard,
                (Some(Kind::Array { .. }), _) => Disposition::Transfer,
                (Some(Kind::ArrayTerm { .. }), _) => {
                    if body
                        .owned
                        .as_ref()
                        .is_some_and(|o| o.creation(*operand).is_some())
                    {
                        return Err("unknown-length ordinary repeat owns a non-Copy operand");
                    }
                    return Ok((&[], Disposition::Transfer));
                }
                _ => return Err("owned repeat has no array type"),
            };
            Ok((std::slice::from_ref(operand), disposition))
        }
        _ => Ok((&[], Disposition::Transfer)),
    }
}
