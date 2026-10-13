//! Canonical signature binders and elision, separate from later outlives checking.
use crate::{resolve::model::DefId, types::RegionId};
use std::collections::{HashMap, HashSet};
#[derive(Clone, Copy)]
pub(super) struct BinderId(usize);
#[derive(Clone, Copy)]
pub(super) enum Position {
    Input,
    Output,
    Static,
}
#[derive(Clone, Copy)]
pub(super) struct Context {
    pub binder: Option<BinderId>,
    pub position: Position,
}
impl Context {
    pub const STATIC: Self = Self {
        binder: None,
        position: Position::Static,
    };
    pub const ALIAS: Self = Self {
        binder: None,
        position: Position::Output,
    };
}
struct Binder {
    parent: Option<BinderId>,
    declared: Vec<DefId>,
    named: HashMap<DefId, u16>,
    count: u32,
    output: Option<RegionId>,
    candidates: HashSet<RegionId>,
    candidate_count: usize,
}
#[derive(Default)]
pub(super) struct Regions {
    binders: Vec<Binder>,
}
impl Regions {
    pub fn binder(&mut self, parent: Option<BinderId>, declared: Vec<DefId>) -> BinderId {
        let id = BinderId(self.binders.len());
        self.binders.push(Binder {
            parent,
            declared,
            named: HashMap::new(),
            count: 0,
            output: None,
            candidates: HashSet::new(),
            candidate_count: 0,
        });
        id
    }
    pub fn count(&self, id: BinderId) -> u32 {
        self.binders[id.0].count
    }
    fn fresh(&mut self, id: BinderId) -> Result<u16, &'static str> {
        let binder = &mut self.binders[id.0];
        let index = u16::try_from(binder.count)
            .map_err(|_| "signature has too many lifetime parameters")?;
        binder.count += 1;
        Ok(index)
    }
    pub fn elided(&mut self, context: Context) -> Result<RegionId, &'static str> {
        if matches!(context.position, Position::Static) {
            return Ok(RegionId::STATIC);
        }
        let id = context.binder.ok_or("missing lifetime specifier")?;
        match context.position {
            Position::Static => Ok(RegionId::STATIC),
            Position::Input => RegionId::bound(0, self.fresh(id)?),
            Position::Output => self.binders[id.0]
                .output
                .ok_or("cannot infer an output lifetime from these parameters"),
        }
    }
    pub fn named(
        &mut self,
        mut binder: Option<BinderId>,
        definition: DefId,
    ) -> Result<RegionId, &'static str> {
        let mut depth = 0;
        while let Some(id) = binder {
            if self.binders[id.0].declared.contains(&definition) {
                let index = if let Some(index) = self.binders[id.0].named.get(&definition) {
                    *index
                } else {
                    let index = self.fresh(id)?;
                    self.binders[id.0].named.insert(definition, index);
                    index
                };
                return RegionId::bound(depth, index);
            }
            binder = self.binders[id.0].parent;
            depth = depth
                .checked_add(1)
                .ok_or("lifetime binder depth overflow")?;
        }
        Err("lifetime is outside the signature's binders")
    }
    pub fn begin_parameter(&mut self, id: BinderId) {
        self.binders[id.0].candidates.clear();
    }
    pub fn record(&mut self, context: Context, region: RegionId) {
        if matches!(context.position, Position::Input)
            && let Some(id) = context.binder
        {
            self.binders[id.0].candidates.insert(region);
        }
    }
    pub fn end_parameter(&mut self, id: BinderId) {
        let binder = &mut self.binders[id.0];
        if binder.candidate_count == 0 && binder.candidates.len() == 1 {
            binder.output = binder.candidates.iter().next().copied();
        }
        binder.candidate_count += binder.candidates.len();
        if binder.candidate_count > 1 {
            binder.output = None;
        }
    }
}
