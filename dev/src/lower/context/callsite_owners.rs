//! A reached operation borrows an actual indexed body, never a copied proof token.
use super::{Context, Owner, TypeId, ir};
use crate::{
    resolve::model::DefId,
    types::closure::{Adapter, ClosureId},
};

#[derive(PartialEq, Eq)]
enum Callable {
    Adapter(Adapter, TypeId),
    Closure(ClosureId, TypeId, Option<TypeId>),
    Function(DefId, TypeId, Option<TypeId>),
}
impl Callable {
    fn owner(owner: Owner) -> Result<Self, &'static str> {
        match owner {
            Owner::ClosureAdapter { adapter, signature } => Ok(Self::Adapter(adapter, signature)),
            Owner::Closure {
                identity,
                signature,
                execution,
            } => Ok(Self::Closure(identity, signature, execution)),
            Owner::Function {
                definition,
                signature,
                execution,
            } => Ok(Self::Function(definition, signature, execution)),
            _ => Err("reached source operation requires an indexed callable body"),
        }
    }
}
impl Context<'_> {
    pub(in crate::lower) fn require_reached_body(
        &self,
        index: usize,
        body: &ir::Body,
    ) -> Result<(), &'static str> {
        let retained = self
            .bodies
            .as_slice()
            .get(index)
            .ok_or("reached source operation has no proved body")?;
        matched_header(retained, body)?;
        let found = match body.owner {
            Owner::ClosureAdapter { adapter, .. } => {
                if body.instance != Some(adapter.instance()) {
                    return Err("reached adapter has another full instance");
                }
                self.adapter_indices.get(&adapter).copied()
            }
            Owner::Closure { .. } | Owner::Function { .. } if body.instance.is_some() => body
                .instance
                .and_then(|instance| self.instance_indices.get(&instance).copied()),
            Owner::Closure { identity, .. } => self.closure_indices.get(&identity).copied(),
            Owner::Function { definition, .. } => self
                .body_indices
                .get(definition.0 as usize)
                .copied()
                .flatten(),
            _ => return Err("reached source operation has a noncallable owner"),
        };
        if found != Some(index) {
            return Err("reached source operation lost its exact owner index");
        }
        Ok(())
    }
}

fn matched_header(retained: &ir::Body, body: &ir::Body) -> Result<(), &'static str> {
    if retained.source != body.source
        || retained.key() != body.key()
        || retained.instance != body.instance
        || retained.result != body.result
        || retained.owner.signatures() != body.owner.signatures()
        || Callable::owner(retained.owner)? != Callable::owner(body.owner)?
    {
        return Err("reached source operation differs from its retained proved header");
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../tests/invariants/callee_lineage.rs"]
mod tests;
