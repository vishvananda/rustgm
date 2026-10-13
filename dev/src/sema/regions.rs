//! Canonical signature binders and elision, separate from later outlives checking.
use crate::{resolve::model::DefId, types::RegionId};
use std::collections::{HashMap, HashSet};
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct BinderId(usize);
#[derive(Clone, Copy)]
pub(super) enum Position {
    Input,
    Impl,
    Output,
    Static,
    Local(crate::syntax::ast::NodeId),
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
    captured: Option<BinderId>,
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
            captured: None,
            declared,
            named: HashMap::new(),
            count: 0,
            output: None,
            candidates: HashSet::new(),
            candidate_count: 0,
        });
        id
    }
    /// A method body uses one flat environment, including anonymous impl slots.
    pub fn capture(&mut self, source: BinderId, declared: Vec<DefId>) -> BinderId {
        let id = self.binder(None, declared);
        let (existing, added) = self.binders.split_at_mut(id.0);
        let captured = &existing[source.0];
        let method = &mut added[0];
        method.named.clone_from(&captured.named);
        method.count = captured.count;
        let mut names = captured.declared.clone();
        names.append(&mut method.declared);
        method.declared = names;
        method.captured = Some(source);
        id
    }
    pub fn captured_region(
        &self,
        mut context: Option<BinderId>,
        source: BinderId,
        slot: u16,
    ) -> Result<RegionId, &'static str> {
        let mut depth = 0_u16;
        while let Some(id) = context {
            if id == source || self.binders[id.0].captured == Some(source) {
                return RegionId::bound(depth, slot);
            }
            context = self.binders[id.0].parent;
            depth = depth
                .checked_add(1)
                .ok_or("lifetime binder depth overflow")?;
        }
        Err("Self is outside its implementation lifetime environment")
    }
    pub fn count(&self, id: BinderId) -> u32 {
        self.binders[id.0].count
    }
    pub fn bindings(&self, id: BinderId) -> Result<Vec<(DefId, RegionId)>, &'static str> {
        let binder = &self.binders[id.0];
        binder
            .declared
            .iter()
            .filter_map(|definition| {
                binder
                    .named
                    .get(definition)
                    .map(|index| RegionId::bound(0, *index).map(|region| (*definition, region)))
            })
            .collect()
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
            Position::Local(_) => Err("local region requires body inference identity"),
            Position::Static => Ok(RegionId::STATIC),
            Position::Input | Position::Impl => RegionId::bound(0, self.fresh(id)?),
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
