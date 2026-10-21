//! Child environment formation imports typed source paths into the parent graph.
use super::{Captures, Checker, Collector, Error, HashMap, Mode, NodeId, PathId};
use crate::sema::bodies::closures::{Entry, places::Place};
use crate::types::closure::CallKind;

pub(in crate::sema::bodies::closures) fn children<'a>(
    descendants: &'a [Entry],
    offset: usize,
    indices: &[usize],
) -> Result<HashMap<NodeId, Option<&'a Captures>>, &'static str> {
    let mut children = HashMap::new();
    for &index in indices {
        let child = index
            .checked_sub(offset)
            .and_then(|index| descendants.get(index))
            .ok_or("nested capture child is outside the active descendant range")?;
        let Ok(metadata) = child.payload.closure() else {
            continue;
        };
        let captures = metadata.captures.as_deref();
        if captures.is_none() && !metadata.bindings.is_empty() {
            return Err("nested capture child has not retained its formation plan");
        }
        if children.insert(child.source, captures).is_some() {
            return Err("nested capture child source is duplicated");
        }
    }
    Ok(children)
}

impl Collector<'_> {
    pub(super) fn nested(&mut self, checker: &mut Checker<'_>, node: NodeId) -> Result<(), Error> {
        let captures = self
            .children
            .get(&node)
            .copied()
            .ok_or_else(|| checker.error(node, "missing immediate nested capture child"))?;
        let Some(captures) = captures else {
            return Ok(());
        };
        let mut imported = HashMap::new();
        for capture in captures.plan.captures() {
            self.work += 1;
            let Some(place) =
                self.import_child(checker, node, captures, capture.path, &mut imported)?
            else {
                continue;
            };
            self.nested_accesses
                .insert((checker.key(node), capture.path), place.path);
            let term = self.places.types[place.path.index()];
            let mut mode = match capture.mode {
                Mode::Value if checker.capture_copy(node, term)? => Mode::Shared,
                Mode::Value => {
                    self.kind = CallKind::Once;
                    Mode::Value
                }
                Mode::Mutable | Mode::UniqueImmutable => {
                    self.kind = self.kind.max(CallKind::Mutable);
                    capture.mode
                }
                Mode::Shared => Mode::Shared,
            };
            let path = place.capture_path(self.moving);
            if self.moving {
                mode = Mode::Value;
            }
            self.places
                .graph
                .require(path, mode)
                .map_err(|e| checker.error(node, e))?;
        }
        for &path in &captures.mentions {
            self.work += 1;
            let Some(place) = self.import_child(checker, node, captures, path, &mut imported)?
            else {
                continue;
            };
            self.nested_accesses
                .insert((checker.key(node), path), place.path);
            if self.mentioned.insert(place.path) {
                self.mentions.push(place.path);
            }
        }
        if self.work > 1_000_000 {
            return Err(checker.error(node, "nested capture formation work limit exceeded"));
        }
        Ok(())
    }
    fn import_child(
        &mut self,
        checker: &mut Checker<'_>,
        node: NodeId,
        captures: &Captures,
        path: PathId,
        imported: &mut HashMap<PathId, Option<Place>>,
    ) -> Result<Option<Place>, Error> {
        let mut pending = Vec::new();
        let mut cursor = path;
        while !imported.contains_key(&cursor) {
            self.work += 1;
            if pending.len() >= captures.plan.path_count() || self.work > 1_000_000 {
                return Err(checker.error(node, "nested capture prefix work limit exceeded"));
            }
            pending.push(cursor);
            let Some((parent, _)) = captures.plan.parent(cursor) else {
                break;
            };
            if parent.index() >= cursor.index() {
                return Err(checker.error(node, "nested capture ancestry is not ordered"));
            }
            cursor = parent;
        }
        while let Some(path) = pending.pop() {
            let place = if let Some((parent, step)) = captures.plan.parent(path) {
                let base = imported
                    .get(&parent)
                    .copied()
                    .ok_or_else(|| checker.error(node, "nested capture parent was not imported"))?;
                base.map(|base| self.places.import_step(checker, node, base, step))
                    .transpose()?
            } else {
                let root = captures
                    .plan
                    .root(path)
                    .ok_or_else(|| checker.error(node, "nested capture root is missing"))?;
                self.places.import_root(checker, node, root)?
            };
            imported.insert(path, place);
        }
        imported
            .get(&path)
            .copied()
            .ok_or_else(|| checker.error(node, "nested capture import is missing"))
    }
}
