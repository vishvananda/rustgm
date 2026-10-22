//! Incremental equality components restore selected witnesses without rescanning old proofs.
use super::{
    Action, Bound, RegionId,
    region_mapping::{Mapping, Mode},
};
use std::collections::HashMap;

struct Vertex {
    parent: usize,
    size: usize,
    anchor: RegionId,
}
#[derive(Default)]
pub(crate) struct RegionEquations {
    indices: HashMap<RegionId, usize>,
    vertices: Vec<Vertex>,
}
impl RegionEquations {
    fn index(&mut self, region: RegionId) -> usize {
        *self.indices.entry(region).or_insert_with(|| {
            let index = self.vertices.len();
            self.vertices.push(Vertex {
                parent: index,
                size: 1,
                anchor: region,
            });
            index
        })
    }

    fn root(&mut self, mut index: usize, steps: &mut usize) -> usize {
        while self.vertices[index].parent != index {
            *steps += 1;
            let parent = self.vertices[index].parent;
            self.vertices[index].parent = self.vertices[parent].parent;
            index = self.vertices[index].parent;
        }
        index
    }

    fn equate(&mut self, left: RegionId, right: RegionId, steps: &mut usize) {
        *steps += 1;
        let (left, right) = (self.index(left), self.index(right));
        let (mut left, mut right) = (self.root(left, steps), self.root(right, steps));
        if left == right {
            return;
        }
        if self.vertices[left].size < self.vertices[right].size {
            std::mem::swap(&mut left, &mut right);
        }
        self.vertices[right].parent = left;
        self.vertices[left].size += self.vertices[right].size;
        // Surrounding bound identities are caller anchors despite their high
        // numeric encoding. Within free identities, older callers/aliases win
        // over the implementation witnesses allocated after them.
        self.vertices[left].anchor = std::cmp::min_by_key(
            self.vertices[left].anchor,
            self.vertices[right].anchor,
            |&region| {
                let rank = if region == RegionId::STATIC {
                    0
                } else if region.bound_parts().is_some() {
                    1
                } else {
                    2
                };
                (rank, region.0)
            },
        );
    }

    pub(crate) fn extend(
        &mut self,
        equations: impl IntoIterator<Item = (RegionId, RegionId)>,
    ) -> Result<usize, &'static str> {
        let mut steps = 0;
        for (left, right) in equations {
            self.equate(left, right, &mut steps);
            if steps > super::WORK_LIMIT || self.vertices.len() > super::WORK_LIMIT {
                return Err("projection closure work budget exceeded");
            }
        }
        Ok(steps)
    }
}

pub(super) struct Closing {
    caller_end: u32,
    captures: u16,
    equations: RegionEquations,
}
impl Closing {
    pub(super) fn allocation(&self) -> crate::support::retained::Allocation {
        let mut total = crate::support::retained::Allocation::default();
        total.map(&self.equations.indices);
        total.vector(&self.equations.vertices);
        total
    }

    pub(super) fn region(
        &mut self,
        region: RegionId,
        anchors: &HashMap<RegionId, RegionId>,
        steps: &mut usize,
    ) -> Result<RegionId, &'static str> {
        *steps += 1;
        let anchor = if let Some(&index) = self.equations.indices.get(&region) {
            let root = self.equations.root(index, steps);
            self.equations.vertices[root].anchor
        } else {
            region
        };
        if let Some(&closed) = anchors.get(&anchor) {
            return Ok(closed);
        }
        if let Some((depth, slot)) = anchor.bound_parts() {
            return RegionId::bound(
                depth
                    .checked_add(self.captures)
                    .ok_or("projection capture depth exhausted")?,
                slot,
            );
        }
        if anchor.0 < self.caller_end {
            return Ok(anchor);
        }
        Err("selected projection lifetime has no visible caller anchor")
    }
}

impl Bound {
    /// Conservative owner charge before building a closed source equality fold.
    pub(crate) fn projection_closure_preflight(equations: usize, slots: usize) -> usize {
        let vertices = equations.saturating_mul(2).saturating_add(slots);
        crate::support::retained::sum([
            size_of::<Self>(),
            size_of::<Mapping>(),
            size_of::<Closing>(),
            crate::support::retained::new_map::<RegionId, usize>(vertices),
            vertices
                .saturating_mul(4)
                .saturating_mul(size_of::<Vertex>()),
            crate::support::retained::new_map::<RegionId, RegionId>(vertices),
        ])
    }
    /// Restore one immutable callable context using equations from its selected
    /// proof graph. Equations remain obligations; this fold does not prove them.
    /// # Errors
    /// Rejects invalid caller frontiers and non-free alias identities.
    pub(crate) fn close_projection_regions(
        caller_end: u32,
        captures: u16,
        anchors: HashMap<RegionId, RegionId>,
    ) -> Result<Self, &'static str> {
        if caller_end == 0
            || caller_end >= 0x8000_0000
            || anchors.keys().any(|id| id.bound_parts().is_some())
        {
            return Err("invalid projection closure context");
        }
        let mut fold = Self::new(Vec::new());
        fold.action = Action::Regions(Box::new(Mapping {
            mode: Mode::Close(Box::new(Closing {
                caller_end,
                captures,
                equations: RegionEquations::default(),
            })),
            replacements: anchors,
        }));
        Ok(fold)
    }

    /// Move a query-owned index into this scope's folder for one restoration.
    /// Moving it back keeps cached scope identities separate without copying the
    /// cumulative equation graph or requiring shared mutable storage.
    pub(crate) fn exchange_projection_equations(
        &mut self,
        equations: &mut RegionEquations,
    ) -> Result<(), &'static str> {
        let Action::Regions(mapping) = &mut self.action else {
            return Err("missing projection closure");
        };
        let Mode::Close(closing) = &mut mapping.mode else {
            return Err("invalid projection closure mode");
        };
        std::mem::swap(&mut closing.equations, equations);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Kind, Primitive, Store, TypeId};

    #[test]
    fn selected_equations_close_witnesses_and_preserve_nested_binders() -> Result<(), &'static str>
    {
        let local = RegionId::bound(0, 0)?;
        let mut fold = Bound::close_projection_regions(10, 1, [(RegionId(10), local)].into())?;
        let mut equations = RegionEquations::default();
        equations.extend([(RegionId(11), RegionId(12)), (RegionId(12), RegionId(10))])?;
        fold.exchange_projection_equations(&mut equations)?;
        assert_eq!(fold.region(RegionId(11), 0)?, local);
        assert_eq!(fold.region(RegionId(12), 2)?, RegionId::bound(2, 0)?);
        assert_eq!(fold.region(RegionId(3), 0)?, RegionId(3));
        assert_eq!(
            fold.region(RegionId::bound(0, 2)?, 0)?,
            RegionId::bound(1, 2)?
        );
        assert!(fold.region(RegionId(13), 0).is_err());
        let mut store = Store::default();
        let reference = store.intern(Kind::Reference {
            mutable: false,
            region: local,
            referent: TypeId::primitive(Primitive::U8),
        })?;
        let parameters = store.intern_list(&[reference])?;
        let function = store.intern(Kind::Function {
            bound_regions: 1,
            parameters,
            result: reference,
            unsafe_: false,
            convention: crate::types::callable::Convention::RUST,
        })?;
        assert_eq!(fold.apply(&mut store, function)?, function);
        Ok(())
    }

    #[test]
    fn incrementally_closing_wide_selected_prefixes_has_linear_work() -> Result<(), &'static str> {
        for width in [16, 64, 256, 1024] {
            let local = RegionId::bound(0, 0)?;
            let mut fold = Bound::close_projection_regions(10, 1, [(RegionId(10), local)].into())?;
            let mut equations = RegionEquations::default();
            let mut steps = 0;
            for offset in 1..=width {
                let next = RegionId(10 + offset);
                steps += equations.extend([(next, RegionId(next.0 - 1))])?;
                fold.exchange_projection_equations(&mut equations)?;
                assert_eq!(fold.region(next, 0)?, local);
                fold.exchange_projection_equations(&mut equations)?;
            }
            assert!(
                steps + fold.steps < width as usize * 12,
                "{}: {}",
                width,
                steps + fold.steps
            );
            assert_eq!(fold.region(RegionId(11), 0)?, local);
        }
        Ok(())
    }

    #[test]
    fn shared_equations_preserve_each_closures_visible_aliases() -> Result<(), &'static str> {
        let local = RegionId::bound(0, 0)?;
        let mut parent = Bound::close_projection_regions(10, 1, [(RegionId(10), local)].into())?;
        let mut child = Bound::close_projection_regions(
            10,
            1,
            [(RegionId(10), RegionId(10)), (RegionId(20), local)].into(),
        )?;
        let mut sibling = Bound::close_projection_regions(10, 1, [(RegionId(30), local)].into())?;
        let mut equations = RegionEquations::default();
        equations.extend([(RegionId(11), RegionId(10)), (RegionId(21), RegionId(20))])?;
        child.exchange_projection_equations(&mut equations)?;
        assert_eq!(child.region(RegionId(11), 0)?, RegionId(10));
        assert_eq!(child.region(RegionId(21), 0)?, local);
        child.exchange_projection_equations(&mut equations)?;
        parent.exchange_projection_equations(&mut equations)?;
        assert_eq!(parent.region(RegionId(11), 0)?, local);
        assert!(parent.region(RegionId(21), 0).is_err());
        parent.exchange_projection_equations(&mut equations)?;
        sibling.exchange_projection_equations(&mut equations)?;
        assert!(sibling.region(RegionId(11), 0).is_err());
        assert!(sibling.region(RegionId(21), 0).is_err());
        Ok(())
    }

    #[test]
    fn selected_witnesses_restore_surrounding_bound_captures() -> Result<(), &'static str> {
        let mut fold = Bound::close_projection_regions(10, 1, HashMap::new())?;
        let mut equations = RegionEquations::default();
        equations.extend([(RegionId(11), RegionId::bound(0, 2)?)])?;
        fold.exchange_projection_equations(&mut equations)?;
        assert_eq!(fold.region(RegionId(11), 0)?, RegionId::bound(1, 2)?);
        assert_eq!(fold.region(RegionId(11), 2)?, RegionId::bound(3, 2)?);
        Ok(())
    }
}
