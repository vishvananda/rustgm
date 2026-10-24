//! Retained configured attribute trees also authenticate each parsed identifier.
use super::{Attribute, Builtin, Forest, Input, Owner, State, Unit, lists};
use crate::{resolve::bindings::claim_tokens::Spelling, syntax::token_tree::Tree};

pub(super) struct Witness {
    pub(super) index: usize,
    pub(super) capture: u32,
    pub(super) attribute: Attribute,
    pub(super) nodes: Vec<Node>,
    pub(super) allowance: bool,
}
pub(super) type Node = (u32, Tree, Option<Spelling>);
#[derive(Clone, Copy)]
pub(super) struct Capture<'owner, 'unit> {
    pub owner: &'owner Owner<'unit>,
    pub index: usize,
    pub attribute: Attribute,
    pub forest: &'owner Forest,
    pub builtin: Option<Builtin>,
    pub allowance: bool,
}
impl Witness {
    pub(super) fn capture(
        capture: Capture<'_, '_>,
        names: &mut lists::Builder,
        work: &mut State,
    ) -> Result<Self, &'static str> {
        let Capture {
            owner,
            index,
            attribute,
            forest,
            builtin,
            allowance,
        } = capture;
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
            let spelling = if let Some(name) = forest.name(owner.unit.source.text(), node) {
                if allowance && !(attribute.path.start..attribute.path.end).contains(&node) {
                    let span = forest
                        .tree(node)
                        .ok_or("missing feature witness tree")?
                        .span;
                    let source = owner
                        .unit
                        .source
                        .text()
                        .get(span.start as usize..span.end as usize)
                        .ok_or("missing feature witness lexeme")?;
                    Some(Spelling::Feature(names.lexeme(source, work)?))
                } else {
                    Some(Spelling::read(name, builtin)?)
                }
            } else {
                None
            };
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
            allowance,
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
                    self.nodes.iter().all(|(index, node, spelling)| {
                        capture.forest.tree(*index) == Some(*node)
                            && spelling.as_ref().is_none_or(|spelling| {
                                spelling.matches(unit.source.text(), &capture.forest, *index)
                            })
                    })
                })
    }
}
