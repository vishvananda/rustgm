//! Demand-only derived siblings reuse the original-node journal and typed lists.
use super::{Kind, NodeId, Unit, error};
use crate::{
    lex::token::LexError,
    syntax::{
        ast::{List, Node},
        item::ItemKind,
    },
};

struct Rewrite {
    parent: NodeId,
    children: Vec<NodeId>,
}
struct Plan {
    owners: Vec<NodeId>,
    groups: Vec<Vec<NodeId>>,
    rewrites: Vec<Rewrite>,
    extra_edges: usize,
}
impl Unit {
    pub(crate) fn attach_derived(
        &mut self,
        outputs: &mut [(NodeId, NodeId)],
        charge: &crate::expand::builtin::choice::Charge<'_>,
    ) -> Result<(), LexError> {
        if outputs.is_empty() {
            return Ok(());
        }
        let Plan {
            owners,
            groups,
            rewrites,
            extra_edges,
        } = self.derive_splice_plan(outputs, charge)?;
        charge(
            rewrites.len(),
            rewrites
                .len()
                .saturating_mul(super::super::original::Original::retirement_row_bytes()),
        )?;
        self.reserve_derive_splices(groups.len(), rewrites.len(), extra_edges)?;
        for (owner, group) in owners.into_iter().zip(groups) {
            let span = self.configured.ast.node(owner).span;
            let children = append_list(&mut self.configured.ast.edges, group);
            self.configured.ast.nodes.push(Node {
                span,
                kind: Kind::Expansion {
                    children,
                    tail: None,
                },
            });
            self.configured.active.push(true);
            self.owner_indices.push(None);
        }
        for rewrite in rewrites {
            let previous = *self.configured.ast.node(rewrite.parent);
            let owner = self.owner_indices[rewrite.parent.get() as usize];
            self.original.save(
                rewrite.parent,
                previous,
                true,
                owner,
                owner.is_some_and(|owner| self.configured.owners[owner.get() as usize - 1].enabled),
            );
            let children = append_list(&mut self.configured.ast.edges, rewrite.children);
            let kind = match previous.kind {
                Kind::Crate(_) => Kind::Crate(children),
                Kind::Expansion { tail, .. } => Kind::Expansion { children, tail },
                Kind::Item(index) => {
                    let mut item = self.configured.ast.items[index as usize];
                    item.kind = ItemKind::Module {
                        items: Some(children),
                    };
                    let index = u32::try_from(self.configured.ast.items.len())
                        .map_err(|_| error(0, "derived module overflow"))?;
                    self.configured.ast.items.push(item);
                    Kind::Item(index)
                }
                _ => {
                    return Err(error(
                        previous.span.start,
                        "derived parent changed during installation",
                    ));
                }
            };
            self.configured.ast.nodes[rewrite.parent.get() as usize].kind = kind;
        }
        Ok(())
    }
    fn reserve_derive_splices(
        &mut self,
        groups: usize,
        rewrites: usize,
        extra_edges: usize,
    ) -> Result<(), LexError> {
        self.original
            .reserve_retirements(rewrites)
            .map_err(|message| error(0, message))?;
        self.configured
            .ast
            .nodes
            .try_reserve(groups)
            .map_err(|_| error(0, "derive wrapper allocation failed"))?;
        self.configured
            .ast
            .edges
            .try_reserve(extra_edges)
            .map_err(|_| error(0, "derive edge allocation failed"))?;
        self.configured
            .ast
            .items
            .try_reserve(rewrites)
            .map_err(|_| error(0, "derive module allocation failed"))?;
        self.configured
            .active
            .try_reserve(groups)
            .map_err(|_| error(0, "derive active allocation failed"))?;
        self.owner_indices
            .try_reserve(groups)
            .map_err(|_| error(0, "derive owner-index allocation failed"))?;
        Ok(())
    }
    fn derive_splice_plan(
        &self,
        outputs: &mut [(NodeId, NodeId)],
        charge: &crate::expand::builtin::choice::Charge<'_>,
    ) -> Result<Plan, LexError> {
        let ast = &self.configured.ast;
        charge(
            ast.nodes
                .len()
                .saturating_add(ast.edges.len())
                .saturating_add(outputs.len()),
            ast.edges
                .len()
                .saturating_mul(8 * size_of::<NodeId>())
                .saturating_add(ast.nodes.len().saturating_mul(4 * size_of::<Rewrite>()))
                .saturating_add(outputs.len().saturating_mul(16 * size_of::<Node>())),
        )?;
        charge(
            outputs
                .len()
                .saturating_mul(outputs.len().checked_ilog2().unwrap_or(0) as usize + 1),
            0,
        )?;
        outputs.sort_unstable_by_key(|(owner, generated)| (owner.get(), generated.get()));
        let base =
            u32::try_from(ast.nodes.len()).map_err(|_| error(0, "derive splice node overflow"))?;
        let (owners, groups) = group_outputs(outputs)?;
        let mut found = vec![0_u8; owners.len()];
        let mut rewrites = Vec::new();
        let logarithm = owners.len().checked_ilog2().unwrap_or(0) as usize + 1;
        for (index, node) in ast.nodes.iter().enumerate() {
            if self.configured.active.get(index) != Some(&true) {
                continue;
            }
            let list = match node.kind {
                Kind::Crate(list) | Kind::Expansion { children: list, .. } => Some(list),
                Kind::Item(item) => match ast.items[item as usize].kind {
                    ItemKind::Module { items } => items,
                    _ => None,
                },
                _ => None,
            };
            let Some(list) = list else {
                continue;
            };
            let children = ast
                .edges
                .get(list.start as usize..list.end as usize)
                .ok_or_else(|| error(node.span.start, "derive parent lost its typed item list"))?;
            charge(children.len().saturating_mul(logarithm), 0)?;
            if !children.iter().any(|child| {
                owners
                    .binary_search_by_key(&child.get(), |owner| owner.get())
                    .is_ok()
            }) {
                continue;
            }
            let mut rewritten = Vec::with_capacity(children.len());
            for &child in children {
                let replacement = if let Ok(owner) =
                    owners.binary_search_by_key(&child.get(), |owner| owner.get())
                {
                    found[owner] = found[owner].saturating_add(1);
                    NodeId::checked(
                        base.checked_add(
                            u32::try_from(owner)
                                .map_err(|_| error(0, "derive wrapper overflow"))?,
                        )
                        .ok_or_else(|| error(0, "derive wrapper overflow"))?,
                    )
                    .ok_or_else(|| error(0, "derive wrapper overflow"))?
                } else {
                    child
                };
                rewritten.push(replacement);
            }
            rewrites.push(Rewrite {
                parent: NodeId::new(
                    u32::try_from(index).map_err(|_| error(0, "derive parent overflow"))?,
                ),
                children: rewritten,
            });
        }
        if found.iter().any(|count| *count != 1) {
            return Err(error(
                0,
                "derived owner must occur once in an actual module item list",
            ));
        }
        Plan::checked(owners, groups, rewrites, ast)
    }
}
impl Plan {
    fn checked(
        owners: Vec<NodeId>,
        groups: Vec<Vec<NodeId>>,
        rewrites: Vec<Rewrite>,
        ast: &crate::syntax::ast::Syntax<crate::literal::storage::StoredValue>,
    ) -> Result<Self, LexError> {
        let extra_edges = groups
            .iter()
            .map(Vec::len)
            .chain(rewrites.iter().map(|rewrite| rewrite.children.len()))
            .try_fold(0_usize, |total, next| total.checked_add(next))
            .ok_or_else(|| error(0, "derive splice edge overflow"))?;
        if ast
            .edges
            .len()
            .checked_add(extra_edges)
            .is_none_or(|count| u32::try_from(count).is_err())
            || ast
                .nodes
                .len()
                .checked_add(groups.len())
                .is_none_or(|count| u32::try_from(count).is_err())
            || ast
                .items
                .len()
                .checked_add(rewrites.len())
                .is_none_or(|count| u32::try_from(count).is_err())
        {
            return Err(error(0, "derive splice exceeds compact arena bounds"));
        }
        Ok(Plan {
            owners,
            groups,
            rewrites,
            extra_edges,
        })
    }
}
fn group_outputs(
    outputs: &[(NodeId, NodeId)],
) -> Result<(Vec<NodeId>, Vec<Vec<NodeId>>), LexError> {
    let mut owners = Vec::new();
    let mut groups = Vec::new();
    for &(owner, generated) in outputs.iter() {
        if owners.last() != Some(&owner) {
            owners.push(owner);
            groups.push(vec![owner]);
        }
        let group = groups
            .last_mut()
            .ok_or_else(|| error(0, "derive splice group disappeared"))?;
        group.push(generated);
    }
    Ok((owners, groups))
}
fn append_list(edges: &mut Vec<NodeId>, values: Vec<NodeId>) -> List {
    let start = u32::try_from(edges.len()).unwrap_or(u32::MAX);
    edges.extend(values);
    List {
        start,
        end: u32::try_from(edges.len()).unwrap_or(u32::MAX),
    }
}
