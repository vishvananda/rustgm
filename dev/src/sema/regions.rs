//! Canonical signature binders and elision, separate from later outlives checking.
use crate::support::hash::{HashMap, HashSet};
use crate::{resolve::model::DefId, types::RegionId};
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct BinderId(usize);
#[derive(Clone, Copy)]
pub(super) enum Position {
    Input,
    Impl,
    Output,
    Static,
    Local(crate::syntax::ast::NodeId),
    Constant(crate::syntax::ast::NodeId),
    Default(DefId),
    Alias(DefId),
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
    owner: Option<DefId>,
    captured: Option<BinderId>,
    declared: Vec<DefId>,
    /// Membership is indexed before slot assignment; slots retain first-use order.
    named: HashMap<DefId, Option<u16>>,
    slot_names: Option<Box<HashMap<u16, DefId>>>,
    count: u32,
    output: Option<RegionId>,
    receiver: Option<RegionId>,
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
            owner: None,
            captured: None,
            named: declared.iter().copied().map(|id| (id, None)).collect(),
            slot_names: None,
            declared,
            count: 0,
            output: None,
            receiver: None,
            candidates: HashSet::default(),
            candidate_count: 0,
        });
        id
    }
    pub fn associate(&mut self, binder: BinderId, owner: DefId) {
        self.binders[binder.0].owner = Some(owner);
    }
    pub fn owner(&self, binder: BinderId) -> Option<DefId> {
        self.binders[binder.0].owner
    }
    pub fn parent(&self, binder: BinderId) -> Option<BinderId> {
        self.binders[binder.0].parent
    }
    /// A method body uses one flat environment, including anonymous impl slots.
    pub fn capture(&mut self, source: BinderId, declared: Vec<DefId>) -> BinderId {
        let id = self.binder(None, declared);
        let (existing, added) = self.binders.split_at_mut(id.0);
        let captured = &existing[source.0];
        let method = &mut added[0];
        method
            .named
            .extend(captured.named.iter().map(|(&id, &slot)| (id, slot)));
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
    /// Opaque results in edition 2024 capture even unused named input lifetimes.
    pub fn capture_all(&mut self, id: BinderId) -> Result<Vec<RegionId>, &'static str> {
        let declared = self.binders[id.0].declared.clone();
        for definition in declared {
            self.named(Some(id), definition)?;
        }
        (0..self.count(id))
            .map(|index| {
                RegionId::bound(
                    0,
                    u16::try_from(index).map_err(|_| "too many captured lifetimes")?,
                )
            })
            .collect()
    }
    pub fn count(&self, id: BinderId) -> u32 {
        self.binders[id.0].count
    }
    pub fn definition_at(&mut self, id: BinderId, slot: u16) -> Option<DefId> {
        let binder = &mut self.binders[id.0];
        if binder.slot_names.is_none() {
            binder.slot_names = Some(Box::new(
                binder
                    .named
                    .iter()
                    .filter_map(|(&name, &slot)| slot.map(|slot| (slot, name)))
                    .collect(),
            ));
        }
        binder.slot_names.as_ref()?.get(&slot).copied()
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
                    .copied()
                    .flatten()
                    .map(|index| RegionId::bound(0, index).map(|region| (*definition, region)))
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
            Position::Local(_) | Position::Constant(_) => {
                Err("local region requires body inference identity")
            }
            Position::Static => Ok(RegionId::STATIC),
            Position::Input | Position::Impl => RegionId::bound(0, self.fresh(id)?),
            Position::Output | Position::Default(_) | Position::Alias(_) => self.binders[id.0]
                .receiver
                .or(self.binders[id.0].output)
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
            if let Some(slot) = self.binders[id.0].named.get(&definition).copied() {
                let index = if let Some(index) = slot {
                    index
                } else {
                    let index = self.fresh(id)?;
                    self.binders[id.0].named.insert(definition, Some(index));
                    if let Some(names) = &mut self.binders[id.0].slot_names {
                        names.insert(index, definition);
                    }
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
    pub fn prefer_receiver(&mut self, id: BinderId, region: RegionId) {
        self.binders[id.0].receiver = Some(region);
    }
    pub fn begin_parameter(&mut self, id: BinderId) {
        self.binders[id.0].candidates.clear();
    }
    /// Several call bounds can share one quantifier, but each elides its own output.
    /// Keep lifetime identities while resetting only callable-local elision state.
    pub fn begin_callable(&mut self, id: BinderId) {
        let binder = &mut self.binders[id.0];
        binder.output = None;
        binder.receiver = None;
        binder.candidates.clear();
        binder.candidate_count = 0;
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
