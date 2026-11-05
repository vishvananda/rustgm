//! Mentioned paths bind to structural coordinates once; unmentioned fields inherit.
use super::{
    parts::{Form, Range},
    tags::{Anchor, Coordinate, Located, Step, Tags},
};
use crate::{
    body::{
        drops::{MemberPosition, MemberQuery},
        ir::{Place, Selection},
    },
    borrow::ownership::{Component, PathId, Paths, paths::Access},
    support::retained::Growth,
    types::TypeId,
};
impl Tags {
    pub(super) fn locate(
        &mut self,
        paths: &Paths,
        position: &impl Fn(TypeId, MemberQuery) -> Result<MemberPosition, &'static str>,
    ) -> Result<(), &'static str> {
        for (index, path) in paths.entries.iter().enumerate() {
            self.parts.charge(1)?;
            let (Some(parent), Some(component)) = (path.parent, path.component) else {
                continue;
            };
            if let Some(origin) = self.origins.get(&parent).copied() {
                self.parts.pay(Growth::map::<PathId, u32>(
                    self.origins.len(),
                    self.origins.capacity(),
                    1,
                ))?;
                self.origins
                    .try_reserve(1)
                    .map_err(|_| "cleanup qualifier path origin allocation failed")?;
                self.origins
                    .insert(PathId(super::parts::compact(index)?), origin);
            }
            let Some(located) = self.locations.get(&parent).copied() else {
                continue;
            };
            let form =
                self.parts.entries[self.nodes[located.coordinate.0 as usize].part.0 as usize].form;
            let (step, merged) = match component {
                Component::Field(Selection::Tuple(field)) => (Step::Tuple(field), located.merged),
                Component::Field(field @ Selection::Nominal(_)) => {
                    let owner = paths
                        .entries
                        .get(parent.0 as usize)
                        .ok_or("cleanup tag projected owner is missing")?
                        .ty;
                    let MemberPosition::Field { variant, field } =
                        position(owner, MemberQuery::Field(field))?
                    else {
                        return Err("cleanup tag projection lost actual declaration coordinates");
                    };
                    (Step::Nominal { variant, field }, located.merged)
                }
                Component::Index(_) | Component::Subarray { .. } if matches!(form, Form::Array) => {
                    (Step::Element, true)
                }
                _ => return Err("cleanup tag projection changed its structural form"),
            };
            let Some(coordinate) = self.child(located.coordinate, step)? else {
                continue;
            };
            self.parts.pay(Growth::map::<PathId, Located>(
                self.locations.len(),
                self.locations.capacity(),
                1,
            ))?;
            self.locations
                .try_reserve(1)
                .map_err(|_| "cleanup tag path index allocation failed")?;
            let path_id = PathId(super::parts::compact(index)?);
            self.locations
                .insert(path_id, Located { coordinate, merged });
            if !merged {
                self.nodes[coordinate.0 as usize].anchor = Anchor::Path(path_id);
            }
        }
        self.resolve_anchors(paths)
    }
    pub(super) fn child(
        &self,
        parent: Coordinate,
        step: Step,
    ) -> Result<Option<Coordinate>, &'static str> {
        let node = self
            .nodes
            .get(parent.0 as usize)
            .ok_or("cleanup tag parent is missing")?;
        let part = self
            .parts
            .entries
            .get(node.part.0 as usize)
            .ok_or("cleanup tag part is missing")?;
        let ordinal = match (part.form, step) {
            (Form::Tuple, Step::Tuple(field)) => field,
            (Form::Array, Step::Element) => 0,
            (Form::Nominal { .. }, Step::Nominal { variant, field }) => {
                let variant = part
                    .variants
                    .start
                    .checked_add(variant)
                    .filter(|&index| index < part.variants.end)
                    .ok_or("cleanup tag variant exceeds actual declaration")?;
                let fields = self
                    .parts
                    .variants
                    .get(variant as usize)
                    .ok_or("cleanup tag variant is missing")?
                    .fields;
                if field >= fields.end - fields.start {
                    return Err("cleanup tag field exceeds actual declaration");
                }
                let first = self
                    .parts
                    .variants
                    .get(part.variants.start as usize)
                    .ok_or("cleanup tag first variant is missing")?
                    .fields
                    .start;
                fields
                    .start
                    .checked_sub(first)
                    .and_then(|n| n.checked_add(field))
                    .ok_or("cleanup tag child ordinal overflow")?
            }
            (Form::Leaf, _) => return Ok(None),
            _ => return Err("cleanup tag child changed its actual structural owner"),
        };
        self.child_ordinal(node.children, ordinal)
    }
    fn child_ordinal(
        &self,
        children: Range,
        ordinal: u32,
    ) -> Result<Option<Coordinate>, &'static str> {
        let entries = self
            .children
            .get(children.start as usize..children.end as usize)
            .ok_or("cleanup qualifier children are missing")?;
        self.parts
            .charge(entries.len().max(1).ilog2() as usize + 1)?;
        Ok(entries
            .binary_search_by_key(&ordinal, |child| child.ordinal)
            .ok()
            .map(|index| entries[index].coordinate))
    }
    fn resolve_anchors(&mut self, paths: &Paths) -> Result<(), &'static str> {
        // Parents are inserted before children, independently of tag-bit order.
        for index in 0..self.nodes.len() {
            self.parts.charge(1)?;
            if !matches!(self.nodes[index].anchor, Anchor::Inherited) {
                continue;
            }
            let parent = self.nodes[index]
                .parent
                .ok_or("cleanup tag root has no real anchor")?;
            let bit = match self
                .nodes
                .get(parent.0 as usize)
                .ok_or("cleanup tag parent is missing")?
                .anchor
            {
                Anchor::Path(path) => {
                    paths
                        .entries
                        .get(path.0 as usize)
                        .ok_or("cleanup tag anchor path is missing")?
                        .start
                }
                Anchor::Shallow(bit) => bit,
                Anchor::Inherited => return Err("cleanup tag anchor resolution is out of order"),
            };
            self.nodes[index].anchor = Anchor::Shallow(bit);
        }
        Ok(())
    }
    pub(super) fn place(
        &self,
        paths: &Paths,
        place: Place,
    ) -> Result<Option<Located>, &'static str> {
        let info = paths.place(place)?;
        if info.access != Access::Owned || info.union || info.empty {
            return Ok(None);
        }
        let Some(path) = info.path else {
            return Ok(None);
        };
        let Some(mut result) = self.locations.get(&path).copied() else {
            return Ok(None);
        };
        if info.indexed {
            let Some(coordinate) = self.child(result.coordinate, Step::Element)? else {
                return Ok(None);
            };
            result = Located {
                coordinate,
                merged: true,
            };
        }
        Ok(Some(result))
    }
}
