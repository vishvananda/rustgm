//! Imported formation facts retain their exact source root and both lifetime views.
use super::{Environment, captures::PathId};
use crate::types::TypeId;
use std::collections::HashSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FormationInput {
    pub root: PathId,
    pub child: TypeId,
    pub parent: TypeId,
}
impl Environment {
    /// Select actual capture/mention roots without reconstructing either type view.
    /// The returned work counts seed visits and distinct-root ancestry visits.
    /// # Errors
    /// Rejects incomplete type coverage or malformed capture ancestry.
    pub fn formation_inputs(&self) -> Result<(Vec<FormationInput>, usize), &'static str> {
        let count = self.plan.path_count();
        if self.path_types.len() != count || self.formation_types.len() != count {
            return Err("capture formation type coverage differs from path forest");
        }
        let mut roots = HashSet::new();
        let mut inputs = Vec::new();
        let mut work = 0;
        for path in self
            .plan
            .captures()
            .iter()
            .map(|capture| capture.path)
            .chain(self.mentions.iter().copied())
        {
            work += 1;
            let definition = self
                .plan
                .root(path)
                .ok_or("invalid capture formation path")?;
            if !roots.insert(definition) {
                continue;
            }
            let mut root = path;
            loop {
                work += 1;
                let Some((parent, _)) = self.plan.parent(root) else {
                    break;
                };
                if parent.index() >= root.index() || self.plan.root(parent) != Some(definition) {
                    return Err("capture formation ancestry is not an ordered source tree");
                }
                root = parent;
            }
            inputs.push(FormationInput {
                root,
                child: *self
                    .path_types
                    .get(root.index())
                    .ok_or("missing child formation type")?,
                parent: *self
                    .formation_types
                    .get(root.index())
                    .ok_or("missing parent formation type")?,
            });
        }
        Ok((inputs, work))
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/source_roots.rs"]
mod tests;
