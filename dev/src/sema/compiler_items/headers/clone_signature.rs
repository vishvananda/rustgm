//! Method signature shape is separate from original-body or caller proof.
use super::Trait;
use crate::{
    resolve::model::DefId,
    types::{Kind, Store, TypeId, callable::Convention},
};

pub(in crate::sema::compiler_items) struct Method {
    pub definition: DefId,
    pub parent: Option<DefId>,
    pub signature: TypeId,
    pub parameters: usize,
}

pub(in crate::sema::compiler_items) fn clone_method(
    store: &Store,
    owner: &Trait<'_>,
    method: &Method,
) -> Result<(), &'static str> {
    if method.parent != Some(owner.definition) || !owner.members.contains(&method.definition) {
        return Err("source clone_fn is not a member of its actual Clone owner");
    }
    let Some(Kind::Function {
        bound_regions: 1,
        parameters,
        result,
        unsafe_: false,
        convention: Convention::RUST,
    }) = store.kind(method.signature)
    else {
        return Err("source clone_fn requires a safe Rust signature with one borrow binder");
    };
    let [receiver] = store.list(parameters).ok_or("missing clone_fn inputs")? else {
        return Err("source clone_fn requires one shared receiver");
    };
    if result != owner.self_type
        || !matches!(store.kind(*receiver), Some(Kind::Reference {
            mutable: false, referent, region,
        }) if referent == owner.self_type && region.bound_parts() == Some((0, 0)))
        || method.parameters != 0
    {
        return Err("source clone_fn does not retain the exact Self input/result contract");
    }
    Ok(())
}
