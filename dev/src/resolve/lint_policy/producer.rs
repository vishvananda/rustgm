//! One source owner produces lint changes; sparse traversal tails retain real ancestry.
#[cfg(test)]
#[path = "../../../tests/invariants/capped_storage.rs"]
mod cap_tests;
mod snapshots;
#[cfg(test)]
#[path = "../../../tests/invariants/lint_storage.rs"]
mod storage_tests;
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
pub(crate) struct Changes<'a> {
    pub entries: &'a [Change],
    pub tail: Option<Tail>,
    pub deprecated: Option<Tail>,
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
    pub effect: u32,
    pub capture: u32,
    pub attribute: Attribute,
    pub tree_count: u32,
}
#[derive(Default)]
pub(super) struct Parsed {
    pub expectations: Vec<super::expectations::Expectation>,
    pub deprecated: Option<Box<super::expectations::Deprecation>>,
    pub changes: Vec<Change>,
    pub witnesses: Box<[Witness]>,
    pub trees: Box<[(u32, crate::syntax::token_tree::Tree)]>,
    pub attributes: usize,
    pub entries: usize,
}
pub(super) struct Owner {
    pub(super) key: NodeKey,
    effects: std::ops::Range<usize>,
    image: Arc<OriginalImage>,
    parent: Option<Tail>,
    header: (
        crate::syntax::item::AttributeRange,
        crate::syntax::ast::NodeId,
        crate::syntax::token_tree::Span,
    ),
    pub(super) parsed: Parsed,
}
#[derive(Clone, Copy, Default)]
pub(crate) struct Work {
    pub attributes: usize,
    pub expectations: usize,
    pub entries: usize,
    pub owners: usize,
    pub frames: usize,
    pub calls: usize,
    pub compositions: usize,
    pub steps: usize,
    pub bytes: usize,
}
impl Work {
    fn exhaustion(&self) -> Option<&'static str> {
        match (self.steps > 10_000_000, self.bytes > 64 * 1024 * 1024) {
            (false, false) => None,
            (true, false) => Some("source lint policy work budget exceeded"),
            (false, true) => Some("source lint policy storage budget exceeded"),
            (true, true) => Some("source lint policy work and storage budgets exceeded"),
        }
    }
    pub(in crate::resolve) fn charge(
        &mut self,
        key: NodeKey,
        steps: usize,
        bytes: usize,
    ) -> Result<(), Violation> {
        self.steps = self.steps.saturating_add(steps);
        self.bytes = self.bytes.saturating_add(bytes);
        if let Some(message) = self.exhaustion() {
            return Err(Violation {
                key,
                offset: 0,
                message,
            });
        }
        Ok(())
    }
}
pub(crate) struct Producer {
    pub(super) owners: Vec<Owner>,
    pub(super) index: HashMap<NodeKey, Tail>,
    frames: Vec<Frame>,
    frame_extent: u32,
    calls: Vec<Call>,
    empty_call_prefix: u32,
    call_extent: u32,
    composed: HashMap<Tail, Result<Policy, Violation>>,
    pub(super) sealed: bool,
    work: Work,
    pub(super) reads: Cell<Work>,
    pub(super) domain: Option<Arc<super::expectations::Domain>>,
    pub(super) expectation_owners: Vec<Tail>,
    pub(super) early: Vec<super::expectations::Early>,
    #[expect(
        clippy::box_collection,
        reason = "a cold optional pointer keeps ordinary producer records compact"
    )]
    pub(in crate::resolve) hidden_globs: Option<Box<Vec<crate::resolve::hidden_globs::Cause>>>,
    pub(super) ineffective_headers: Option<Box<super::ineffective_headers::Annotations>>,
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
            domain: None,
            expectation_owners: Vec::new(),
            early: Vec::new(),
            hidden_globs: None,
            ineffective_headers: None,
        }
    }
    pub(crate) fn work(&self) -> Work {
        let mut work = self.work;
        let reads = self.reads.get();
        work.steps = work.steps.saturating_add(reads.steps);
        work.bytes = work.bytes.saturating_add(reads.bytes);
        work
    }
    pub(crate) fn capped(
        frames: u32,
        calls: u32,
        cap: crate::support::diagnostics::LintCap,
        key: NodeKey,
    ) -> Result<Self, Violation> {
        let mut producer = Self::new(frames, calls);
        if cap.suppresses_lints() {
            producer.ensure_cap_domain(key, cap)?;
        }
        Ok(producer)
    }
    pub(super) fn require_cap(&self, package: &Package, key: NodeKey) -> Result<(), Violation> {
        self.attribute_read(key, 1)?;
        if self.cap() != package.lint_cap() {
            return Err(invalid(
                key,
                "lint producer changed its actual compilation cap",
            ));
        }
        Ok(())
    }
    pub(crate) fn validate_cap(&self, package: &Package, key: NodeKey) -> Result<(), Violation> {
        self.require_cap(package, key)
    }
    pub(crate) fn reserve_diagnostic(
        &mut self,
        key: NodeKey,
        bytes: usize,
    ) -> Result<(), Violation> {
        self.work.charge(key, 1, bytes)
    }
    /// Typed attribute classification consumes the existing general read budget;
    /// this cost-only operation publishes no owner, lint change or permission.
    pub(crate) fn attribute_read(&self, key: NodeKey, steps: usize) -> Result<(), Violation> {
        self.diagnostic_payload(key, steps, 0)
    }
    pub(crate) fn diagnostic_read(&self, key: NodeKey, bytes: usize) -> Result<(), Violation> {
        self.diagnostic_payload(key, 1, bytes)
    }
    pub(crate) fn diagnostic_payload(
        &self,
        key: NodeKey,
        steps: usize,
        bytes: usize,
    ) -> Result<(), Violation> {
        let mut reads = self.reads.get();
        let result = reads.charge(key, steps, bytes);
        self.reads.set(reads);
        result?;
        self.work().charge(key, 0, 0)
    }
    pub(crate) fn diagnostic_budget_exhausted(&self) -> bool {
        self.work().exhaustion().is_some()
    }
    pub(crate) fn validate_diagnostic_path(
        &self,
        package: &Package,
        mut tail: Option<Tail>,
        key: NodeKey,
        validated: &mut crate::support::hash::HashSet<Tail>,
    ) -> Result<(), Violation> {
        while let Some(current) = tail {
            self.diagnostic_read(key, 4 * size_of::<Tail>())?;
            if !validated.insert(current) {
                return Ok(());
            }
            let owner = self.record(current, key)?;
            owner.validate(package, &self.reads)?;
            if owner
                .parent
                .is_some_and(|parent| parent.0.get() >= current.0.get())
            {
                return Err(invalid(key, "source diagnostic lint ancestry changed"));
            }
            tail = owner.parent;
        }
        Ok(())
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
        self.require_cap(package, key)?;
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
            return Ok(if owner.parsed.has_policy() {
                Some(tail)
            } else {
                parent
            });
        }
        if self.sealed {
            return Err(invalid(
                key,
                "sealed lint producer cannot admit another owner",
            ));
        }
        let (effects, cap) = (configured.attributes.clone(), self.cap());
        self.work.charge(
            key,
            1,
            4 * size_of::<Owner>() + crate::support::retained::new_map::<NodeKey, Tail>(1),
        )?;

        let value = self
            .owners
            .len()
            .checked_add(1)
            .and_then(|value| u32::try_from(value).ok())
            .and_then(NonZeroU32::new)
            .ok_or_else(|| invalid(key, "source lint owner identities exceed 32 bits"))?;
        let tail = Tail(value);
        let (parsed, deferred) = attribute_changes::owner(
            package.unit(key.file),
            key,
            tail,
            effects.clone(),
            &mut self.work,
            cap,
        )?;
        self.publish_deferred(key, deferred)?;
        let next = if parsed.has_policy() {
            Some(tail)
        } else {
            parent
        };
        let annotation = super::ineffective_headers::capture(
            package.unit(key.file),
            key,
            tail,
            &parsed,
            &mut self.work,
        )?;
        self.publish_impl_annotation(key, annotation)?;
        let node = package.unit(key.file).configured.ast.node(key.node);
        let crate::syntax::ast::Kind::Attributed { attributes, value } = node.kind else {
            return Err(invalid(
                key,
                "lint source header lost its attributed syntax",
            ));
        };
        self.publish_expectation_owner(key, tail, &parsed)?;
        self.owners.push(Owner {
            key,
            header: (attributes, value, node.span),
            effects,
            image: Arc::clone(package.unit(key.file).source.original_image()),
            parent,
            parsed,
        });
        self.index.insert(key, tail);
        if self
            .owners
            .last()
            .is_some_and(|owner| owner.parsed.deprecated.is_some())
        {
            let unit = package.unit(key.file);
            let mut declaration = key;
            while let crate::syntax::ast::Kind::Attributed { value, .. } =
                unit.configured.ast.node(declaration.node).kind
            {
                self.work.charge(key, 1, 0)?;
                declaration.node = value;
            }
            self.work.charge(
                key,
                1,
                crate::support::retained::new_map::<NodeKey, Tail>(1),
            )?;
            self.index.insert(declaration, tail);
        }

        self.work.owners += 1;
        Ok(next)
    }
    fn publish_expectation_owner(
        &mut self,
        key: NodeKey,
        tail: Tail,
        parsed: &Parsed,
    ) -> Result<(), Violation> {
        if parsed.expectations.is_empty() {
            return Ok(());
        }
        let growth = crate::support::retained::Growth::vector::<Tail>(
            self.expectation_owners.len(),
            self.expectation_owners.capacity(),
            1,
        );
        self.work.charge(key, 1 + growth.work, growth.bytes)?;
        self.ensure_domain(key)?;
        self.expectation_owners.push(tail);
        Ok(())
    }
    fn require_tail(&self, tail: Option<Tail>, key: NodeKey) -> Result<(), Violation> {
        if let Some(tail) = tail {
            self.record(tail, key)?;
        }
        Ok(())
    }
    pub(super) fn record(&self, tail: Tail, key: NodeKey) -> Result<&Owner, Violation> {
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
        let extent = self.frame_extent.max(
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
                    let growth = crate::support::retained::Growth::vector::<Frame>(
                        self.frames.len(),
                        self.frames.capacity(),
                        1,
                    );
                    self.work
                        .charge(key, 1usize.saturating_add(growth.work), growth.bytes)?;
                    self.frames.push(Frame { index: frame, tail });
                    self.work.frames += 1;
                }
            }
        }
        self.frame_extent = extent;
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
        self.require_cap(package, key)?;
        let Some(root) = tail else {
            self.work.charge(key, 1, 0)?;
            return Ok(Policy::default().capped(self.cap()));
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
            return result.map(|policy| policy.capped(self.cap()));
        }
        self.work.compositions += 1;
        let result = self.compose(&path, key);
        self.work.charge(
            key,
            1,
            crate::support::retained::new_map::<Tail, Result<Policy, Violation>>(1),
        )?;
        self.composed.insert(root, result);
        result.map(|policy| policy.capped(self.cap()))
    }
    fn compose(&mut self, path: &[Tail], key: NodeKey) -> Result<Policy, Violation> {
        let mut policy = Policy::default();
        for &tail in path.iter().rev() {
            let count = self.record(tail, key)?.parsed.changes.len();
            self.work.charge(key, count, 0)?;
            let owner = self.record(tail, key)?;
            for change in &owner.parsed.changes {
                policy.apply(owner.key, change, self.cap())?;
            }
            if owner.parsed.deprecated.is_some() {
                policy.deprecated_origin = Some(tail);
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
        self.require_cap(package, key)?;
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
    ) -> Result<Changes<'_>, Violation> {
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
            return Ok(Changes {
                entries: &[],
                tail: parent,
                deprecated: None,
            });
        };
        let owner = unit
            .configured
            .owners
            .get(index.get() as usize - 1)
            .ok_or_else(|| invalid(key, "lint owner index is malformed"))?;
        if owner.attributes.is_empty() {
            return Ok(Changes {
                entries: &[],
                tail: parent,
                deprecated: None,
            });
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
        Ok(Changes {
            entries: &record.parsed.changes,
            tail: if record.parsed.has_policy() {
                Some(tail)
            } else {
                parent
            },
            deprecated: record.parsed.deprecated.as_ref().map(|_| tail),
        })
    }
    pub(crate) fn validate(&self, package: &Package) -> Result<(usize, usize, bool), Violation> {
        self.require_cap(
            package,
            NodeKey {
                file: crate::resolve::unit::FileId(0),
                node: package.modules[0].root,
            },
        )?;
        let work = self.work();
        if let Some(message) = work.exhaustion() {
            return Err(invalid(
                NodeKey {
                    file: crate::resolve::unit::FileId(0),
                    node: package.modules[0].root,
                },
                message,
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
pub(super) fn invalid(key: NodeKey, message: &'static str) -> Violation {
    Violation {
        key,
        offset: 0,
        message,
    }
}

impl Parsed {
    pub(super) fn has_policy(&self) -> bool {
        !self.changes.is_empty() || self.deprecated.is_some()
    }
}
