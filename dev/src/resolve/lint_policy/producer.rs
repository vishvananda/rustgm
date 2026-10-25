//! One source owner produces lint changes; sparse traversal tails retain real ancestry.
mod snapshots;
#[cfg(test)]
#[path = "../../../tests/invariants/lint_producer.rs"]
mod tests;
use super::{Change, Policy, Violation, attribute_changes};
use crate::{
    expand::builtin::BuiltinKind,
    resolve::{
        model::ScopeId,
        package::{ModuleId, Package},
        unit::NodeKey,
    },
    support::{hash::HashMap, source::OriginalImage},
    syntax::attribute::Attribute,
};
use std::{cell::Cell, num::NonZeroU32, sync::Arc};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Tail(NonZeroU32);
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct Site {
    pub call: u32,
    pub owner: NodeKey,
    pub capture: u32,
    pub kind: BuiltinKind,
    pub module: ModuleId,
    pub scope: ScopeId,
    pub scope_parent: Option<ScopeId>,
}
struct Call {
    site: Site,
    tail: Option<Tail>,
}
struct Frame {
    index: u32,
    tail: Tail,
}
pub(super) struct Witness {
    pub effect: usize,
    pub capture: u32,
    pub attribute: Attribute,
    pub trees: Vec<(u32, crate::syntax::token_tree::Tree)>,
}
#[derive(Default)]
pub(super) struct Parsed {
    pub changes: Vec<Change>,
    pub witnesses: Vec<Witness>,
    pub attributes: usize,
    pub entries: usize,
}
struct Owner {
    key: NodeKey,
    effects: std::ops::Range<usize>,
    image: Arc<OriginalImage>,
    parent: Option<Tail>,
    parsed: Parsed,
}
#[derive(Clone, Copy, Default)]
pub(crate) struct Work {
    pub attributes: usize,
    pub entries: usize,
    pub owners: usize,
    pub frames: usize,
    pub calls: usize,
    pub compositions: usize,
    pub steps: usize,
    pub bytes: usize,
}
impl Work {
    pub(super) fn charge(
        &mut self,
        key: NodeKey,
        steps: usize,
        bytes: usize,
    ) -> Result<(), Violation> {
        self.steps = self.steps.saturating_add(steps);
        self.bytes = self.bytes.saturating_add(bytes);
        if self.steps > 10_000_000 || self.bytes > 64 * 1024 * 1024 {
            return Err(Violation {
                key,
                offset: 0,
                message: "source lint policy work or storage budget exceeded",
            });
        }
        Ok(())
    }
}
pub(crate) struct Producer {
    owners: Vec<Owner>,
    index: HashMap<NodeKey, Tail>,
    frames: Vec<Frame>,
    frame_extent: u32,
    calls: Vec<Call>,
    empty_call_prefix: u32,
    call_extent: u32,
    composed: HashMap<Tail, Result<Policy, Violation>>,
    sealed: bool,
    work: Work,
    reads: Cell<Work>,
}
impl Producer {
    pub(crate) fn new(frames: u32, calls: u32) -> Self {
        Self {
            owners: Vec::new(),
            index: HashMap::default(),
            frames: Vec::new(),
            frame_extent: frames,
            calls: Vec::new(),
            empty_call_prefix: calls,
            call_extent: calls,
            composed: HashMap::default(),
            sealed: false,
            work: Work::default(),
            reads: Cell::default(),
        }
    }
    pub(crate) fn work(&self) -> Work {
        let mut work = self.work;
        let reads = self.reads.get();
        work.steps = work.steps.saturating_add(reads.steps);
        work.bytes = work.bytes.saturating_add(reads.bytes);
        work
    }
    pub(crate) fn reserve_diagnostic(
        &mut self,
        key: NodeKey,
        bytes: usize,
    ) -> Result<(), Violation> {
        self.work.charge(key, 1, bytes)
    }
    fn charge_read(&self, key: NodeKey, steps: usize) -> Result<(), Violation> {
        let mut reads = self.reads.get();
        let result = reads.charge(key, steps, 0);
        self.reads.set(reads);
        result
    }
    pub(crate) fn owner(
        &mut self,
        package: &Package,
        key: NodeKey,
        parent: Option<Tail>,
    ) -> Result<Option<Tail>, Violation> {
        self.work.charge(key, 1, 0)?;
        let configured = snapshots::configured(package, key)?;
        if configured.attributes.is_empty() {
            return Ok(parent);
        }
        self.require_tail(parent, key)?;
        if let Some(&tail) = self.index.get(&key) {
            let owner = self.record(tail, key)?;
            owner.validate(package, &self.reads)?;
            if owner.parent != parent {
                return Err(invalid(key, "source lint owner parent tail changed"));
            }
            return Ok(if owner.parsed.changes.is_empty() {
                parent
            } else {
                Some(tail)
            });
        }
        if self.sealed {
            return Err(invalid(
                key,
                "sealed lint producer cannot admit another owner",
            ));
        }
        let effects = configured.attributes.clone();
        self.work.charge(
            key,
            1,
            4 * size_of::<Owner>() + crate::support::retained::new_map::<NodeKey, Tail>(1),
        )?;
        let parsed =
            attribute_changes::owner(package.unit(key.file), key, effects.clone(), &mut self.work)?;
        let value = self
            .owners
            .len()
            .checked_add(1)
            .and_then(|value| u32::try_from(value).ok())
            .and_then(NonZeroU32::new)
            .ok_or_else(|| invalid(key, "source lint owner identities exceed 32 bits"))?;
        let tail = Tail(value);
        let next = if parsed.changes.is_empty() {
            parent
        } else {
            Some(tail)
        };
        self.owners.push(Owner {
            key,
            effects,
            image: Arc::clone(package.unit(key.file).source.original_image()),
            parent,
            parsed,
        });
        self.index.insert(key, tail);
        self.work.owners += 1;
        Ok(next)
    }
    fn require_tail(&self, tail: Option<Tail>, key: NodeKey) -> Result<(), Violation> {
        if let Some(tail) = tail {
            self.record(tail, key)?;
        }
        Ok(())
    }
    fn record(&self, tail: Tail, key: NodeKey) -> Result<&Owner, Violation> {
        self.charge_read(key, 1)?;
        self.owners
            .get(tail.0.get() as usize - 1)
            .ok_or_else(|| invalid(key, "unknown source lint ancestry tail"))
    }
    pub(crate) fn frame(&mut self, frame: u32, key: NodeKey) -> Result<Option<Tail>, Violation> {
        self.work.charge(
            key,
            1 + self.frames.len().checked_ilog2().unwrap_or(0) as usize,
            0,
        )?;
        if frame >= self.frame_extent || self.sealed {
            return Err(invalid(key, "source lint traversal frame is not live"));
        }
        Ok(self
            .frames
            .binary_search_by_key(&frame, |slot| slot.index)
            .ok()
            .map(|index| self.frames[index].tail))
    }
    pub(crate) fn set_frame(
        &mut self,
        frame: u32,
        tail: Option<Tail>,
        key: NodeKey,
    ) -> Result<(), Violation> {
        self.work.charge(key, 1, 0)?;
        self.require_tail(tail, key)?;
        if frame > self.frame_extent || self.sealed {
            return Err(invalid(
                key,
                "source lint frame publication skipped its actual index",
            ));
        }
        self.frame_extent = self.frame_extent.max(
            frame
                .checked_add(1)
                .ok_or_else(|| invalid(key, "source lint frame extent overflow"))?,
        );
        match self.frames.binary_search_by_key(&frame, |slot| slot.index) {
            Ok(index) => {
                if let Some(tail) = tail {
                    self.frames[index].tail = tail;
                } else {
                    self.frames.remove(index);
                }
            }
            Err(index) => {
                if let Some(tail) = tail {
                    if index != self.frames.len() {
                        return Err(invalid(
                            key,
                            "source lint frame was not appended in live index order",
                        ));
                    }
                    self.work.charge(key, 1, 4 * size_of::<Frame>())?;
                    self.frames.push(Frame { index: frame, tail });
                    self.work.frames += 1;
                }
            }
        }
        Ok(())
    }
    pub(crate) fn truncate(&mut self, mark: u32) {
        while self.frames.last().is_some_and(|frame| frame.index >= mark) {
            self.frames.pop();
        }
        self.frame_extent = mark;
    }
    pub(crate) fn call(&mut self, site: Site, tail: Option<Tail>) -> Result<(), Violation> {
        self.work.charge(site.owner, 1, 4 * size_of::<Call>())?;
        self.require_tail(tail, site.owner)?;
        if site.call != self.call_extent || self.sealed {
            return Err(invalid(
                site.owner,
                "source lint call publication has another actual index",
            ));
        }
        self.call_extent = self
            .call_extent
            .checked_add(1)
            .ok_or_else(|| invalid(site.owner, "source lint call extent overflow"))?;
        self.calls.push(Call { site, tail });
        self.work.calls += 1;
        Ok(())
    }
    pub(crate) fn call_tail(&mut self, site: Site) -> Result<Option<Tail>, Violation> {
        self.work.charge(site.owner, 1, 0)?;
        if self.sealed || site.call >= self.call_extent {
            return Err(invalid(
                site.owner,
                "source lint call snapshot is not active",
            ));
        }
        if site.call < self.empty_call_prefix {
            return Ok(None);
        }
        let index = (site.call - self.empty_call_prefix) as usize;
        let call = self
            .calls
            .get(index)
            .ok_or_else(|| invalid(site.owner, "source lint call snapshot is missing"))?;
        if call.site != site {
            return Err(invalid(
                site.owner,
                "source lint actual call owner, capture or scope changed",
            ));
        }
        Ok(call.tail)
    }
    pub(crate) fn policy(
        &mut self,
        package: &Package,
        tail: Option<Tail>,
        key: NodeKey,
    ) -> Result<Policy, Violation> {
        let Some(root) = tail else {
            self.work.charge(key, 1, 0)?;
            return Ok(Policy::default());
        };
        let mut path = Vec::new();
        let mut current = Some(root);
        while let Some(tail) = current {
            self.work.charge(key, 1, 4 * size_of::<Tail>())?;
            let owner = self.record(tail, key)?;
            owner.validate(package, &self.reads)?;
            if owner
                .parent
                .is_some_and(|parent| parent.0.get() >= tail.0.get())
            {
                return Err(invalid(key, "source lint ancestry is cyclic or forward"));
            }
            current = owner.parent;
            path.push(tail);
        }
        if let Some(&result) = self.composed.get(&root) {
            return result;
        }
        self.work.compositions += 1;
        let result = self.compose(&path, key);
        self.work.charge(
            key,
            1,
            crate::support::retained::new_map::<Tail, Result<Policy, Violation>>(1),
        )?;
        self.composed.insert(root, result);
        result
    }
    fn compose(&mut self, path: &[Tail], key: NodeKey) -> Result<Policy, Violation> {
        let mut policy = Policy::default();
        for &tail in path.iter().rev() {
            let count = self.record(tail, key)?.parsed.changes.len();
            self.work.charge(key, count, 0)?;
            let owner = self.record(tail, key)?;
            for change in &owner.parsed.changes {
                policy.apply(owner.key, change)?;
            }
        }
        Ok(policy)
    }
    pub(crate) fn finish(&mut self) {
        self.frames = Vec::new();
        self.calls = Vec::new();
        self.composed = HashMap::default();
        self.sealed = true;
    }
    /// Authenticate an already produced owner without composing ancestor policy.
    pub(crate) fn validate_owner(&self, package: &Package, key: NodeKey) -> Result<(), Violation> {
        self.charge_read(key, 1)?;
        let tail = self
            .index
            .get(&key)
            .copied()
            .ok_or_else(|| invalid(key, "source header owner was not positively produced"))?;
        self.record(tail, key)?.validate(package, &self.reads)
    }
    pub(crate) fn changes(
        &self,
        package: &Package,
        key: NodeKey,
        parent: Option<Tail>,
    ) -> Result<(&[Change], Option<Tail>), Violation> {
        self.charge_read(key, 1)?;
        let Some(unit) = package.units.get(key.file.0 as usize) else {
            return Err(invalid(key, "foreign lint policy source file"));
        };
        let Some(index) = unit
            .owner_indices
            .get(key.node.get() as usize)
            .copied()
            .flatten()
        else {
            return Ok((&[], parent));
        };
        let owner = unit
            .configured
            .owners
            .get(index.get() as usize - 1)
            .ok_or_else(|| invalid(key, "lint owner index is malformed"))?;
        if owner.attributes.is_empty() {
            return Ok((&[], parent));
        }
        let tail = self
            .index
            .get(&key)
            .copied()
            .ok_or_else(|| invalid(key, "active source lint owner was never produced"))?;
        let record = self.record(tail, key)?;
        record.validate(package, &self.reads)?;
        if record.parent != parent {
            return Err(invalid(
                key,
                "semantic lint owner has another actual ancestor tail",
            ));
        }
        Ok((
            &record.parsed.changes,
            if record.parsed.changes.is_empty() {
                parent
            } else {
                Some(tail)
            },
        ))
    }
    pub(crate) fn validate(&self, package: &Package) -> Result<(usize, usize, bool), Violation> {
        let work = self.work();
        if work.steps > 10_000_000 || work.bytes > 64 * 1024 * 1024 {
            return Err(invalid(
                NodeKey {
                    file: crate::resolve::unit::FileId(0),
                    node: package.modules[0].root,
                },
                "combined lint source work or storage budget exceeded",
            ));
        }
        if !self.sealed {
            return Err(invalid(
                NodeKey {
                    file: crate::resolve::unit::FileId(0),
                    node: package.modules[0].root,
                },
                "source lint producer has not finished expansion",
            ));
        }
        snapshots::validate(self, package)
    }
}
fn invalid(key: NodeKey, message: &'static str) -> Violation {
    Violation {
        key,
        offset: 0,
        message,
    }
}
