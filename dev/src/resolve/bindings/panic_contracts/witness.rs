//! Retained configured attribute trees also authenticate each parsed identifier.
use super::{Attribute, Builtin, Forest, Input, Owner, State, Unit};
use crate::{resolve::bindings::claim_tokens::Spelling, syntax::token_tree::Tree};

pub(super) struct Witness {
    pub(super) index: usize,
    pub(super) capture: u32,
    pub(super) attribute: Attribute,
    pub(super) nodes: Vec<Node>,
}
pub(super) type Node = (u32, Tree, Option<Spelling>);
impl Witness {
    pub(super) fn capture(
        owner: &Owner<'_>,
        index: usize,
        attribute: Attribute,
        forest: &Forest,
        builtin: Option<Builtin>,
        work: &mut State,
    ) -> Result<Self, &'static str> {
        let mut count = attribute.path.end - attribute.path.start;
        let input = if let Input::Delimited(group) = attribute.input {
            let end = forest.after(group);
            count = count
                .checked_add(end - group)
                .ok_or("macro attribute witness size overflow")?;
            group..end
        } else {
            0..0
        };
        work.charge(
            // Each identifier is compared with at most four known identities.
            (count as usize).saturating_mul(4),
            (count as usize).saturating_mul(4 * size_of::<Node>()),
        )?;
        let mut nodes = Vec::new();
        nodes
            .try_reserve_exact(count as usize)
            .map_err(|_| "macro attribute witness allocation failed")?;
        for node in (attribute.path.start..attribute.path.end).chain(input) {
            let spelling = forest
                .name(owner.unit.source.text(), node)
                .map(|name| Spelling::read(name, builtin))
                .transpose()?;
            nodes.push((
                node,
                forest
                    .tree(node)
                    .ok_or("invalid macro attribute witness tree")?,
                spelling,
            ));
        }
        Ok(Self {
            index,
            capture: owner.unit.configured.effects[index].capture,
            attribute,
            nodes,
        })
    }
    pub(super) fn valid(&self, unit: &Unit) -> bool {
        let Some(effect) = unit.configured.effects.get(self.index) else {
            return false;
        };
        effect.attribute.effect == crate::expand::configured::Effect::Keep
            && effect.capture == self.capture
            && effect.attribute.attribute == self.attribute
            && unit
                .configured
                .ast
                .attributes
                .get(self.capture as usize)
                .is_some_and(|capture| {
                    self.nodes.iter().all(|&(index, node, spelling)| {
                        capture.forest.tree(index) == Some(node)
                            && spelling.is_none_or(|spelling| {
                                capture.forest.name(unit.source.text(), index)
                                    == Some(spelling.name())
                            })
                    })
                })
    }
}
