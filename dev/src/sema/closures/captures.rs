//! Canonical capture prefixes are selected in two passes over a flat place forest.
use crate::resolve::model::DefId;
use std::collections::HashMap;

/// Required capture strength, independent of the closure's eventual call trait.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Mode {
    Shared,
    UniqueImmutable,
    Mutable,
    Value,
}
/// Projection identities come from typed places after precision/truncation rules.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Step {
    Field(u32),
    Dereference,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PathId(u32);
#[derive(Clone, Copy)]
struct Path {
    root: DefId,
    parent: Option<(PathId, Step)>,
    used: bool,
    mode: Option<Mode>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Capture {
    pub path: PathId,
    pub mode: Mode,
}
/// One closure's places. IDs belong to this forest; errors invalidate its result.
pub struct Graph {
    paths: Vec<Path>,
    roots: HashMap<DefId, PathId>,
    edges: HashMap<(PathId, Step), PathId>,
    work: usize,
    limit: usize,
    failed: bool,
}
impl Default for Graph {
    fn default() -> Self {
        Self {
            paths: Vec::new(),
            roots: HashMap::new(),
            edges: HashMap::new(),
            work: 0,
            limit: 1_000_000,
            failed: false,
        }
    }
}
/// Selected prefixes share their paths; no capture owns a copied ancestor chain.
pub struct Plan {
    paths: Vec<Path>,
    pub captures: Vec<Capture>,
    /// Actual place operations and finalization node visits, including cache hits.
    pub work: usize,
}
impl Plan {
    #[must_use]
    pub fn root(&self, path: PathId) -> Option<DefId> {
        self.paths.get(path.0 as usize).map(|p| p.root)
    }
    #[must_use]
    pub fn parent(&self, path: PathId) -> Option<(PathId, Step)> {
        self.paths.get(path.0 as usize)?.parent
    }
    #[must_use]
    pub fn path_count(&self) -> usize {
        self.paths.len()
    }
}
impl Graph {
    fn fail(&mut self, message: &'static str) -> &'static str {
        self.failed = true;
        message
    }
    fn charge(&mut self) -> Result<(), &'static str> {
        if self.failed {
            return Err("capture graph is invalid after an earlier failure");
        }
        if self.work >= self.limit {
            return Err(self.fail("closure capture work budget exceeded"));
        }
        self.work += 1;
        Ok(())
    }
    fn push(
        &mut self,
        root: DefId,
        parent: Option<(PathId, Step)>,
    ) -> Result<PathId, &'static str> {
        let id = u32::try_from(self.paths.len())
            .map_err(|_| self.fail("closure capture path identity overflow"))?;
        self.paths.push(Path {
            root,
            parent,
            used: false,
            mode: None,
        });
        Ok(PathId(id))
    }
    /// Intern a resolved outer binding without capturing it merely by visiting it.
    /// # Errors
    /// Reports exhausted work or identity storage; no result may then be published.
    pub fn root(&mut self, binding: DefId) -> Result<PathId, &'static str> {
        self.charge()?;
        if let Some(&id) = self.roots.get(&binding) {
            return Ok(id);
        }
        let id = self.push(binding, None)?;
        self.roots.insert(binding, id);
        Ok(id)
    }
    /// Intern one typed projection, sharing its complete parent path.
    /// # Errors
    /// Rejects missing parents, exhausted work and identity overflow.
    pub fn project(&mut self, parent: PathId, step: Step) -> Result<PathId, &'static str> {
        self.charge()?;
        let Some(path) = self.paths.get(parent.0 as usize) else {
            return Err(self.fail("invalid closure capture parent"));
        };
        if let Some(&id) = self.edges.get(&(parent, step)) {
            return Ok(id);
        }
        let id = self.push(path.root, Some((parent, step)))?;
        self.edges.insert((parent, step), id);
        Ok(id)
    }
    /// Record an actual use after choosing the required precision and capture mode.
    /// # Errors
    /// Rejects missing paths and exhausted work, including repeated cached uses.
    pub fn require(&mut self, path: PathId, mode: Mode) -> Result<(), &'static str> {
        self.charge()?;
        let Some(path) = self.paths.get_mut(path.0 as usize) else {
            return Err(self.fail("invalid required closure capture path"));
        };
        path.used = true;
        path.mode = Some(path.mode.map_or(mode, |prior| prior.max(mode)));
        Ok(())
    }
    /// Merge required prefixes without rescanning a subtree for each new use.
    /// Results retain stable path-discovery order; environment layout is separate.
    /// # Errors
    /// Rejects an earlier failed query or exhausted finalization work.
    pub fn finish(mut self) -> Result<Plan, &'static str> {
        if self.failed {
            return Err("capture graph is invalid after an earlier failure");
        }
        for index in (0..self.paths.len()).rev() {
            self.charge()?;
            let path = self.paths[index];
            if let Some((parent, _)) = path.parent {
                let parent = &mut self.paths[parent.0 as usize];
                parent.mode = parent.mode.max(path.mode);
            }
        }
        let mut covered = vec![false; self.paths.len()];
        let mut captures = Vec::new();
        for index in 0..self.paths.len() {
            self.charge()?;
            let path = self.paths[index];
            let inherited = path.parent.is_some_and(|(p, _)| covered[p.0 as usize]);
            covered[index] = inherited || path.used;
            if !inherited && path.used {
                let mode = path.mode.ok_or("required capture lacks a mode")?;
                let id = u32::try_from(index).map_err(|_| "closure capture identity overflow")?;
                captures.push(Capture {
                    path: PathId(id),
                    mode,
                });
            }
        }
        Ok(Plan {
            paths: self.paths,
            captures,
            work: self.work,
        })
    }
}

#[cfg(test)]
mod prefix_checks;
