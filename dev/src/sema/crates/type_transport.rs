//! Typed signature graphs use one indexed remap per import, never source reconstruction.
use super::{Declarations, Error, error, scalar};
use crate::support::hash::HashMap;
use crate::{
    support::crates::{Abi, Lifetime, RecordRef, Signature, TypeGraph, TypeNode, TypeRef},
    types::{GenericArgs, ItemId, Kind, RegionId, TypeId, callable::Convention},
};

pub(super) fn import(facts: &mut Declarations, signature: &Signature) -> Result<TypeId, Error> {
    let types = import_graph(facts, signature.types(), &[])?;
    import_roots(facts, signature, &types)
}
pub(super) fn import_graph(
    facts: &mut Declarations,
    graph: &TypeGraph,
    records: &[TypeId],
) -> Result<Vec<TypeId>, Error> {
    let mut types = Vec::with_capacity(graph.nodes().len());
    for node in graph.nodes() {
        facts.crates.get_or_insert_default().import_work.type_nodes += 1;
        let ty = match *node {
            TypeNode::Scalar(value) => scalar(value),
            TypeNode::Record(record) => *records
                .get(usize::from(record.0))
                .ok_or_else(|| error("missing imported nominal identity"))?,
            TypeNode::Array { element, length } => {
                let element = *types
                    .get(usize::from(element.0))
                    .ok_or_else(|| error("missing imported array element"))?;
                facts
                    .types
                    .intern(Kind::Array { element, length })
                    .map_err(error)?
            }
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
    Ok(types)
}
pub(super) fn import_roots(
    facts: &mut Declarations,
    signature: &Signature,
    types: &[TypeId],
) -> Result<TypeId, Error> {
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
            convention: Convention {
                abi: match signature.abi() {
                    Abi::Rust => crate::types::callable::Abi::Rust,
                    Abi::C => crate::types::callable::Abi::C,
                },
                variadic: signature.variadic(),
            },
        })
        .map_err(error)
}

#[derive(Default)]
pub(super) struct Export {
    pub(super) nodes: Vec<TypeNode>,
    remap: HashMap<TypeId, TypeRef>,
}
impl Export {
    pub(super) fn root(
        &mut self,
        facts: &Declarations,
        root: TypeId,
        records: &HashMap<ItemId, RecordRef>,
    ) -> Result<TypeRef, Error> {
        let mut work = vec![(root, false)];
        while let Some((ty, finish)) = work.pop() {
            if self.remap.contains_key(&ty) {
                continue;
            }
            if self.nodes.len() >= TypeGraph::MAX_NODES || work.len() > TypeGraph::MAX_DEPTH {
                return Err(error("exported type graph exceeds profile budget"));
            }
            let kind = facts.types.kind(ty);
            let child = match kind {
                Some(Kind::Reference { referent, .. }) => Some(referent),
                Some(Kind::Array { element, .. }) => Some(element),
                _ => None,
            };
            if !finish && let Some(child) = child {
                work.push((ty, true));
                work.push((child, false));
                continue;
            }
            let node = match kind {
                Some(Kind::Reference {
                    region,
                    mutable,
                    referent,
                }) => {
                    let region = if region == RegionId::STATIC {
                        Lifetime::Static
                    } else {
                        match region.bound_parts() {
                            Some((0, slot)) => Lifetime::Bound(slot),
                            _ => {
                                return Err(error(
                                    "exported reference retains a nonlocal lifetime",
                                ));
                            }
                        }
                    };
                    TypeNode::Reference {
                        region,
                        mutable,
                        referent: self.mapped(referent)?,
                    }
                }
                Some(Kind::Array { element, length }) => TypeNode::Array {
                    element: self.mapped(element)?,
                    length,
                },
                Some(Kind::Adt(item, GenericArgs::EMPTY)) => TypeNode::Record(
                    *records
                        .get(&item)
                        .ok_or_else(|| error("exposed nominal is outside the metadata profile"))?,
                ),
                _ => TypeNode::Scalar(super::export::scalar(facts, ty)?),
            };
            let id = TypeRef(
                u16::try_from(self.nodes.len()).map_err(|_| error("too many exported types"))?,
            );
            self.nodes.push(node);
            self.remap.insert(ty, id);
        }
        self.mapped(root)
    }
    fn mapped(&self, ty: TypeId) -> Result<TypeRef, Error> {
        self.remap
            .get(&ty)
            .copied()
            .ok_or_else(|| error("missing exported type root"))
    }
}
pub(super) struct SignatureRoots {
    slots: u16,
    parameters: Vec<TypeRef>,
    result: TypeRef,
    unsafe_: bool,
    abi: Abi,
    variadic: bool,
}
impl SignatureRoots {
    pub(super) fn finish(self, graph: TypeGraph) -> Result<Signature, Error> {
        Signature::with_shared_types(
            graph,
            self.slots,
            self.parameters,
            self.result,
            self.unsafe_,
        )
        .and_then(|signature| signature.with_convention(self.abi, self.variadic))
        .map_err(error)
    }
}
pub(super) fn export_roots(
    facts: &Declarations,
    signature: TypeId,
    graph: &mut Export,
    records: &HashMap<ItemId, RecordRef>,
) -> Result<SignatureRoots, Error> {
    let Some(Kind::Function {
        bound_regions,
        parameters,
        result,
        unsafe_,
        convention,
    }) = facts.types.kind(signature)
    else {
        return Err(error(
            "export calling convention is outside the metadata profile",
        ));
    };
    let parameters = facts
        .types
        .list(parameters)
        .ok_or_else(|| error("missing export parameters"))?
        .iter()
        .map(|&ty| graph.root(facts, ty, records))
        .collect::<Result<Vec<_>, _>>()?;
    let result = graph.root(facts, result, records)?;
    Ok(SignatureRoots {
        slots: u16::try_from(bound_regions)
            .map_err(|_| error("too many exported lifetime slots"))?,
        parameters,
        result,
        unsafe_,
        variadic: convention.variadic,
        abi: match convention.abi {
            crate::types::callable::Abi::Rust => Abi::Rust,
            crate::types::callable::Abi::C => Abi::C,
            crate::types::callable::Abi::RustCall => {
                return Err(error("rust-call export is outside the metadata profile"));
            }
        },
    })
}
pub(super) fn export(facts: &Declarations, signature: TypeId) -> Result<Signature, Error> {
    let mut graph = Export::default();
    let roots = export_roots(facts, signature, &mut graph, &HashMap::default())?;
    Signature::with_types(
        graph.nodes,
        roots.slots,
        roots.parameters,
        roots.result,
        roots.unsafe_,
    )
    .and_then(|signature| signature.with_convention(roots.abi, roots.variadic))
    .map_err(error)
}
