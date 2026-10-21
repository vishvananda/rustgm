//! A known function-pointer expectation supplies the closure's quantified contract.
use super::{Checker, Constructor, Error, NodeId, TermId, TypeId, universes::Interface};
use crate::types::{Kind, Store, infer};

#[derive(Clone, Copy)]
pub(super) struct Expected {
    signature: TypeId,
    pub regions: u32,
}
impl Checker<'_> {
    pub(super) fn closure_expectation(
        &mut self,
        node: NodeId,
        expected: Option<TermId>,
        count: u32,
    ) -> Result<Option<Expected>, Error> {
        let Some(term) = expected else {
            return Ok(None);
        };
        let Some(Constructor::Function {
            bound_regions,
            parameters,
            ..
        }) = self.contextual_head(node, term)?
        else {
            return Ok(None);
        };
        if parameters != count {
            return Err(self.error(
                node,
                "closure input count differs from its expected signature",
            ));
        }
        let signature = match self.state.infer.resolved(&mut self.facts.types, term) {
            Ok(signature) => signature,
            Err(infer::Error::Unresolved) => return Ok(None),
            Err(error) => return Err(self.error(node, error)),
        };
        let signature = self.alias_type(node, signature)?;
        Ok(Some(Expected {
            signature,
            regions: bound_regions,
        }))
    }
}

pub(super) fn constrain(
    store: &mut Store,
    context: &mut infer::Context,
    signature: TermId,
    interface: &mut Interface,
    expected: Expected,
) -> Result<(), &'static str> {
    let regions = interface.count()?;
    let expected = interface
        .opening
        .signature(store, expected.signature, regions)?;
    let Some(Kind::Function {
        parameters, result, ..
    }) = store.kind(expected)
    else {
        return Err("missing contextual closure signature");
    };
    let mut children = store
        .list(parameters)
        .ok_or("missing contextual inputs")?
        .to_vec();
    children.push(result);
    for (index, ty) in children.into_iter().enumerate() {
        let expected = context
            .known(store, ty)
            .map_err(|_| "invalid contextual closure type")?;
        let actual = context
            .child(store, signature, index)
            .map_err(|_| "invalid closure input")?;
        context
            .equate(store, actual, expected)
            .map_err(|_| "closure differs from its expected signature")?;
    }
    Ok(())
}
