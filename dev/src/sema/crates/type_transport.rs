//! Typed signature graphs use one indexed remap per import, never source reconstruction.
use super::{Declarations, Error, error, scalar};
use crate::{
    support::crates::{Lifetime, Signature, TypeGraph, TypeNode, TypeRef},
    types::{Kind, RegionId, TypeId, callable::Convention},
};
use std::collections::HashMap;

pub(super) fn import(facts: &mut Declarations, signature: &Signature) -> Result<TypeId, Error> {
    let mut types = Vec::with_capacity(signature.types().nodes().len());
    for node in signature.types().nodes() {
        facts.crates.get_or_insert_default().import_work.type_nodes += 1;
        let ty = match *node {
            TypeNode::Scalar(value) => scalar(value),
            TypeNode::Reference {
                region,
                mutable,
                referent,
            } => {
                let referent = *types
                    .get(usize::from(referent.0))
                    .ok_or_else(|| error("missing imported referent"))?;
                let region = match region {
                    Lifetime::Static => RegionId::STATIC,
                    Lifetime::Bound(slot) => RegionId::bound(0, slot).map_err(error)?,
                };
                facts
                    .types
                    .intern(Kind::Reference {
                        region,
                        mutable,
                        referent,
                    })
                    .map_err(error)?
            }
        };
        types.push(ty);
    }
    let get = |id: TypeRef| {
        types
            .get(usize::from(id.0))
            .copied()
            .ok_or_else(|| error("unknown imported type root"))
    };
    let parameters = signature
        .parameters()
        .iter()
        .copied()
        .map(get)
        .collect::<Result<Vec<_>, _>>()?;
    let result = get(signature.result())?;
    let work = &mut facts.crates.get_or_insert_default().import_work;
    work.signatures += 1;
    work.roots += signature.parameters().len() + 1;
    let parameters = facts.types.intern_list(&parameters).map_err(error)?;
    facts
        .types
        .intern(Kind::Function {
            bound_regions: u32::from(signature.bound_regions()),
            parameters,
            result,
            unsafe_: signature.unsafe_(),
            convention: Convention::RUST,
        })
        .map_err(error)
}

#[derive(Default)]
struct Export {
    nodes: Vec<TypeNode>,
    remap: HashMap<TypeId, TypeRef>,
}
impl Export {
    fn root(&mut self, facts: &Declarations, root: TypeId) -> Result<TypeRef, Error> {
        let mut work = vec![(root, false)];
        while let Some((ty, finish)) = work.pop() {
            if self.remap.contains_key(&ty) {
                continue;
            }
            if self.nodes.len() >= TypeGraph::MAX_NODES || work.len() > TypeGraph::MAX_DEPTH {
                return Err(error("exported type graph exceeds profile budget"));
            }
            let node = if let Some(Kind::Reference {
                region,
                mutable,
                referent,
            }) = facts.types.kind(ty)
            {
                if !finish {
                    work.push((ty, true));
                    work.push((referent, false));
                    continue;
                }
                let region = if region == RegionId::STATIC {
                    Lifetime::Static
                } else {
                    match region.bound_parts() {
                        Some((0, slot)) => Lifetime::Bound(slot),
                        _ => return Err(error("exported reference retains a nonlocal lifetime")),
                    }
                };
                TypeNode::Reference {
                    region,
                    mutable,
                    referent: *self
                        .remap
                        .get(&referent)
                        .ok_or_else(|| error("missing exported referent"))?,
                }
            } else {
                TypeNode::Scalar(super::export::scalar(facts, ty)?)
            };
            let id = TypeRef(
                u16::try_from(self.nodes.len()).map_err(|_| error("too many exported types"))?,
            );
            self.nodes.push(node);
            self.remap.insert(ty, id);
        }
        self.remap
            .get(&root)
            .copied()
            .ok_or_else(|| error("missing exported type root"))
    }
}
pub(super) fn export(facts: &Declarations, signature: TypeId) -> Result<Signature, Error> {
    let Some(Kind::Function {
        bound_regions,
        parameters,
        result,
        unsafe_,
        convention: Convention::RUST,
    }) = facts.types.kind(signature)
    else {
        return Err(error(
            "export calling convention is outside the metadata profile",
        ));
    };
    let mut graph = Export::default();
    let parameters = facts
        .types
        .list(parameters)
        .ok_or_else(|| error("missing export parameters"))?
        .iter()
        .map(|&ty| graph.root(facts, ty))
        .collect::<Result<Vec<_>, _>>()?;
    let result = graph.root(facts, result)?;
    Signature::with_types(
        graph.nodes,
        u16::try_from(bound_regions).map_err(|_| error("too many exported lifetime slots"))?,
        parameters,
        result,
        unsafe_,
    )
    .map_err(error)
}
