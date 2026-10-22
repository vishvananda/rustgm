//! Canonical projection recipes for loan overlap, distinct from owned move paths.
mod view;
use crate::body::ir::{
    Action, Body, LocalId, LocalKind, Place, ProjectionKind, Selection, ValueId,
};
use crate::support::hash::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct Id(pub usize);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Step {
    Dereference { mutable: bool },
    Field(Selection),
    UnionField(Selection),
    Element(u64),
    Range(u64, u64),
    FromEnd { offset: u64, minimum: u64 },
    Subslice { prefix: u64, suffix: u64 },
    Dynamic(ValueId),
}
#[derive(Clone, Copy)]
struct Path {
    root: Place,
    parent: Option<Id>,
    step: Option<Step>,
    shared: bool,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Depth {
    Deep,
    Shallow,
    Discriminant,
    Storage,
}
#[derive(Clone, Copy)]
enum Choices {
    Single(Id),
    Range(usize, usize),
}
pub(super) struct Index {
    paths: Vec<Path>,
    roots: HashMap<Place, Id>,
    children: HashMap<(Id, Step), Id>,
    projections: Vec<Id>,
    aliases: HashMap<LocalId, Vec<Id>>,
    expanded: HashMap<Id, Choices>,
    alternatives: Vec<Id>,
    left: Vec<Step>,
    right: Vec<Step>,
    pub steps: usize,
}
impl Index {
    pub fn new(body: &Body) -> Result<Self, &'static str> {
        let mut this = Self {
            paths: Vec::new(),
            roots: HashMap::default(),
            children: HashMap::default(),
            projections: Vec::new(),
            aliases: HashMap::default(),
            expanded: HashMap::default(),
            alternatives: Vec::new(),
            left: Vec::new(),
            right: Vec::new(),
            steps: 0,
        };
        this.charge(body.locals.len().saturating_add(body.statements.len()))?;
        for (i, local) in body.locals.iter().enumerate() {
            if matches!(local.kind, LocalKind::Guard(_)) {
                if this.aliases.len() >= 500_000 {
                    return Err("loan guard index budget exceeded");
                }
                this.aliases.insert(
                    LocalId(u32::try_from(i).map_err(|_| "too many loan locals")?),
                    Vec::new(),
                );
            }
        }
        for (ordinal, projection) in body.projections.iter().enumerate() {
            if matches!(projection.kind, ProjectionKind::RawDereference { .. }) {
                let id = crate::body::ir::ProjectionId(
                    u32::try_from(ordinal).map_err(|_| "too many raw loan projections")?,
                );
                this.capacity()?;
                let path = Id(this.paths.len());
                this.paths.push(Path {
                    root: Place::Projection(id),
                    parent: None,
                    step: None,
                    shared: false,
                });
                this.projections.push(path);
                continue;
            }
            let base = this.place(projection.base)?;
            if matches!(projection.kind, ProjectionKind::CaptureBoundary { .. }) {
                this.projections.push(base);
                continue;
            }
            let step = match projection.kind {
                ProjectionKind::CaptureBoundary { .. } => {
                    return Err("capture boundary escaped transparent loan dispatch");
                }
                ProjectionKind::Dereference { mutable, .. } => Step::Dereference { mutable },
                ProjectionKind::RawDereference { .. } => {
                    return Err("raw loan root escaped dispatch");
                }
                ProjectionKind::Field(field) => Step::Field(field),
                ProjectionKind::UnionField(field) => Step::UnionField(field),
                ProjectionKind::ConstantIndex(i) => Step::Element(i),
                ProjectionKind::Subarray { start, end } => Step::Range(start, end),
                ProjectionKind::FromEnd { offset, minimum } => Step::FromEnd { offset, minimum },
                ProjectionKind::Subslice { prefix, suffix } => Step::Subslice { prefix, suffix },
                ProjectionKind::Index { index, .. }
                | ProjectionKind::IndexParameter { index, .. }
                | ProjectionKind::SliceIndex { index } => Step::Dynamic(index),
            };
            let path = this.child(base, step)?;
            this.projections.push(path);
        }
        let mut alias_count = 0;
        for statement in &body.statements {
            if let Action::BindView { view, place } = statement.action {
                alias_count += 1;
                if alias_count > 500_000 {
                    return Err("loan guard source budget exceeded");
                }
                let id = this.place(place)?;
                this.aliases
                    .get_mut(&view)
                    .ok_or("invalid loan guard alias")?
                    .push(id);
            }
        }
        Ok(this)
    }
    pub fn charge(&mut self, count: usize) -> Result<(), &'static str> {
        self.steps = self
            .steps
            .checked_add(count)
            .filter(|n| *n <= 32_000_000)
            .ok_or("loan place work budget exceeded")?;
        Ok(())
    }
    pub fn place(&mut self, place: Place) -> Result<Id, &'static str> {
        self.charge(1)?;
        if let Place::Projection(id) = place {
            return self
                .projections
                .get(id.0 as usize)
                .copied()
                .ok_or("invalid loan projection base");
        }
        if let Some(&id) = self.roots.get(&place) {
            return Ok(id);
        }
        self.capacity()?;
        let id = Id(self.paths.len());
        self.paths.push(Path {
            root: place,
            parent: None,
            step: None,
            shared: false,
        });
        self.roots.insert(place, id);
        Ok(id)
    }
    fn capacity(&self) -> Result<(), &'static str> {
        if self.paths.len().saturating_add(self.alternatives.len()) >= 500_000 {
            return Err("loan place storage budget exceeded");
        }
        Ok(())
    }
    fn child(&mut self, mut base: Id, mut step: Step) -> Result<Id, &'static str> {
        self.charge(1)?;
        if let Some(Step::Subslice { prefix, suffix }) = self.paths[base.0].step {
            let sum = |a: u64, b: u64| a.checked_add(b).ok_or("loan slice offset overflow");
            let normalized = match step {
                Step::Element(i) => Some(Step::Element(sum(prefix, i)?)),
                Step::FromEnd { offset, minimum } => Some(Step::FromEnd {
                    offset: sum(suffix, offset)?,
                    minimum: sum(sum(prefix, suffix)?, minimum)?,
                }),
                Step::Subslice {
                    prefix: a,
                    suffix: b,
                } => Some(Step::Subslice {
                    prefix: sum(prefix, a)?,
                    suffix: sum(suffix, b)?,
                }),
                _ => None,
            };
            if let Some(normalized) = normalized {
                base = self.paths[base.0].parent.ok_or("loan slice has no base")?;
                step = normalized;
            }
        }
        if let Some(Step::Range(start, end)) = self.paths[base.0].step {
            match step {
                Step::Element(i) if i < end - start => {
                    base = self.paths[base.0].parent.ok_or("loan range has no base")?;
                    step = Step::Element(start + i);
                }
                Step::Range(a, b) if a <= b && b <= end - start => {
                    base = self.paths[base.0].parent.ok_or("loan range has no base")?;
                    step = Step::Range(start + a, start + b);
                }
                _ => {}
            }
        }
        if let Some(&id) = self.children.get(&(base, step)) {
            return Ok(id);
        }
        self.capacity()?;
        let path = self.paths[base.0];
        let id = Id(self.paths.len());
        self.paths.push(Path {
            root: path.root,
            parent: Some(base),
            step: Some(step),
            shared: path.shared || step == Step::Dereference { mutable: false },
        });
        self.children.insert((base, step), id);
        Ok(id)
    }
    fn recipe(&mut self, mut id: Id, left: bool) -> Result<Place, &'static str> {
        let steps = if left {
            &mut self.left
        } else {
            &mut self.right
        };
        steps.clear();
        while let Some(parent) = self.paths[id.0].parent {
            steps.push(
                self.paths[id.0]
                    .step
                    .ok_or("missing loan projection step")?,
            );
            id = parent;
        }
        let root = self.paths[id.0].root;
        let count = steps.len();
        self.charge(count)?;
        Ok(root)
    }
    fn choices(&mut self, id: Id) -> Result<Choices, &'static str> {
        if let Some(&choices) = self.expanded.get(&id) {
            return Ok(choices);
        }
        let root = self.recipe(id, true)?;
        let Place::Local(local) = root else {
            return Ok(Choices::Single(id));
        };
        if !self.aliases.contains_key(&local) {
            return Ok(Choices::Single(id));
        }
        let start = self.alternatives.len();
        let mut work = vec![(id, 0)];
        while let Some((id, depth)) = work.pop() {
            self.charge(1)?;
            if depth > self.aliases.len() {
                return Err("cyclic loan guard alias");
            }
            let root = self.recipe(id, true)?;
            let aliases = match root {
                Place::Local(local) => self.aliases.get(&local).cloned(),
                _ => None,
            };
            if let Some(aliases) = aliases {
                if aliases.is_empty() {
                    return Err("loan guard view has no source");
                }
                self.charge(aliases.len())?;
                if work.len().saturating_add(aliases.len()) > 500_000 {
                    return Err("loan alias worklist budget exceeded");
                }
                for mut alias in aliases {
                    for i in (0..self.left.len()).rev() {
                        alias = self.child(alias, self.left[i])?;
                    }
                    work.push((alias, depth + 1));
                }
            } else {
                self.capacity()?;
                self.alternatives.push(id);
            }
        }
        let result = Choices::Range(start, self.alternatives.len());
        self.expanded.insert(id, result);
        Ok(result)
    }
    pub fn expanded(
        &mut self,
        place: Place,
    ) -> Result<impl Iterator<Item = (Id, bool)> + '_, &'static str> {
        let id = self.place(place)?;
        let (single, start, end) = match self.choices(id)? {
            Choices::Single(id) => (Some(id), 0, 0),
            Choices::Range(a, b) => (None, a, b),
        };
        self.charge(end - start + usize::from(single.is_some()))?;
        Ok(single
            .into_iter()
            .chain(self.alternatives[start..end].iter().copied())
            .map(|id| (id, !self.paths[id.0].shared)))
    }
    pub fn root(&self, id: Id) -> Place {
        self.paths[id.0].root
    }
    pub fn conflicts(
        &mut self,
        loan: Id,
        access: Place,
        depth: Depth,
    ) -> Result<bool, &'static str> {
        self.accessed(loan, access, depth, false)
    }
    pub fn protected_conflicts(
        &mut self,
        protected: Place,
        access: Place,
        depth: Depth,
        deep: bool,
    ) -> Result<bool, &'static str> {
        let id = self.place(protected)?;
        match self.choices(id)? {
            Choices::Single(id) => self.accessed(id, access, depth, !deep),
            Choices::Range(a, b) => {
                for i in a..b {
                    if self.accessed(self.alternatives[i], access, depth, !deep)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
        }
    }
    fn accessed(
        &mut self,
        loan: Id,
        access: Place,
        depth: Depth,
        shallow: bool,
    ) -> Result<bool, &'static str> {
        let access = self.place(access)?;
        // Ending a view's hidden pointer storage does not end the selected
        // referent. Loans through a view retain the selected source identity.
        if depth == Depth::Storage {
            return self.overlap(loan, access, depth, shallow);
        }
        match self.choices(access)? {
            Choices::Single(access) => self.overlap(loan, access, depth, shallow),
            Choices::Range(a, b) => {
                for i in a..b {
                    if self.overlap(loan, self.alternatives[i], depth, shallow)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
        }
    }
    fn overlap(
        &mut self,
        loan: Id,
        access: Id,
        depth: Depth,
        shallow: bool,
    ) -> Result<bool, &'static str> {
        if self.paths[loan.0].root != self.paths[access.0].root {
            return Ok(false);
        }
        self.recipe(loan, true)?;
        self.recipe(access, false)?;
        if shallow && self.left.len() < self.right.len() {
            return Ok(false);
        }
        for (&a, &b) in self.left.iter().rev().zip(self.right.iter().rev()) {
            if matches!((a, b), (Step::UnionField(a), Step::UnionField(b)) if a != b) {
                return Ok(true);
            }
            if disjoint(a, b) {
                return Ok(false);
            }
        }
        if self.left.len() > self.right.len() {
            let remaining = &self.left[..self.left.len() - self.right.len()];
            if depth == Depth::Discriminant {
                return Ok(false);
            }
            if depth != Depth::Deep
                && remaining
                    .iter()
                    .any(|s| matches!(s, Step::Dereference { .. }))
            {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
fn disjoint(a: Step, b: Step) -> bool {
    match (a, b) {
        (Step::Field(a), Step::Field(b)) => a != b,
        (Step::Element(i), Step::Range(a, b)) | (Step::Range(a, b), Step::Element(i)) => {
            i < a || i >= b
        }
        (Step::Range(a, b), Step::Range(c, d)) => b <= c || d <= a,
        (Step::Element(a), Step::Element(b))
        | (Step::FromEnd { offset: a, .. }, Step::FromEnd { offset: b, .. }) => a != b,
        (Step::Element(i), Step::FromEnd { offset, minimum })
        | (Step::FromEnd { offset, minimum }, Step::Element(i)) => {
            i.checked_add(offset).is_some_and(|sum| sum < minimum)
        }
        (Step::Element(i), Step::Subslice { prefix, .. })
        | (Step::Subslice { prefix, .. }, Step::Element(i)) => i < prefix,
        (Step::FromEnd { offset, .. }, Step::Subslice { suffix, .. })
        | (Step::Subslice { suffix, .. }, Step::FromEnd { offset, .. }) => offset <= suffix,
        _ => false,
    }
}
