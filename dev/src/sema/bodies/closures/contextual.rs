//! Function-pointer expectations retain their quantified contract and unresolved holes.
use super::{Checker, Constructor, Error, NodeId, TermId, TypeId, universes::Interface};
use crate::types::{Kind, Store, infer};

#[derive(Clone, Copy)]
pub(super) enum Signature {
    Known(TypeId),
    Inferred(TermId),
}
#[derive(Clone, Copy)]
pub(super) struct Expected {
    pub signature: Signature,
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
            convention: crate::types::callable::Convention::RUST,
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
            Ok(signature) => Signature::Known(self.alias_type(node, signature)?),
            Err(infer::Error::Unresolved) => {
                Signature::Inferred(self.alias_inferred_type(node, term)?)
            }
            Err(error) => return Err(self.error(node, error)),
        };
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
    let children = match expected.signature {
        Signature::Known(expected) => known_children(store, context, interface, expected)?,
        Signature::Inferred(expected) => inferred_children(store, context, interface, expected)?,
    };
    for (index, expected) in children.into_iter().enumerate() {
        let actual = context
            .child(store, signature, index)
            .map_err(|_| "invalid closure input")?;
        context
            .equate(store, actual, expected)
            .map_err(|_| "closure differs from its expected signature")?;
    }
    Ok(())
}

fn known_children(
    store: &mut Store,
    context: &mut infer::Context,
    interface: &mut Interface,
    expected: TypeId,
) -> Result<Vec<TermId>, &'static str> {
    let regions = interface.count()?;
    let expected = interface.opening.signature(store, expected, regions)?;
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
    children
        .into_iter()
        .map(|ty| {
            context
                .known(store, ty)
                .map_err(|_| "invalid contextual closure type")
        })
        .collect()
}

fn inferred_children(
    store: &mut Store,
    context: &mut infer::Context,
    interface: &mut Interface,
    expected: TermId,
) -> Result<Vec<TermId>, &'static str> {
    let Some(Constructor::Function { parameters, .. }) = context
        .head(store, expected)
        .map_err(|_| "invalid contextual closure signature")?
    else {
        return Err("missing contextual closure signature");
    };
    let mut children = Vec::with_capacity(parameters as usize + 1);
    for index in 0..=parameters as usize {
        children.push(
            context
                .child(store, expected, index)
                .map_err(|_| "invalid contextual closure input")?,
        );
    }
    // Open this signature's binder once over all roots. Shared partial graphs
    // and repeated inference holes retain their identity across its children.
    let tuple = context
        .construct(Constructor::Tuple, &children)
        .map_err(|_| "invalid contextual input bundle")?;
    let opened = context
        .map_regions(store, tuple, &mut interface.opening)
        .map_err(|_| "invalid contextual lifetime mapping")?;
    for (index, child) in children.iter_mut().enumerate() {
        *child = context
            .child(store, opened, index)
            .map_err(|_| "invalid opened contextual input")?;
    }
    Ok(children)
}
