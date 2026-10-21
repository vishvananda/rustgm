//! Callable expectations retain their quantified contract and unresolved holes.
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
        let term = if let Some(signature) = self
            .state
            .opaque_callables
            .as_deref()
            .and_then(|signatures| signatures.get(&term))
            .copied()
        {
            self.known(node, signature)?
        } else {
            term
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

    /// Index a defining result's callable interface once. This contextual hint
    /// supplies no permission: the original bound still requires selection.
    pub(in crate::sema::bodies) fn declared_closure_signature(
        &mut self,
        node: NodeId,
        id: crate::types::opaque::OpaqueId,
        term: TermId,
        clause: crate::types::parameters::clauses::Clause,
    ) -> Result<(), Error> {
        use crate::types::{
            arguments::Argument,
            parameters::{clauses::Clause, predicates::Bound},
        };
        let (predicate, regions) = match clause {
            Clause::Predicate(predicate) => (predicate, 0),
            Clause::ForAll(id) => {
                let quantified = self
                    .facts
                    .types
                    .quantified(id)
                    .ok_or_else(|| self.error(node, "missing contextual callable bound"))?;
                (quantified.predicate, quantified.bound_regions)
            }
        };
        let Bound::ProjectionEqual(result) = predicate.bound else {
            return Ok(());
        };
        let Some(Kind::Projection(application)) = self.facts.types.kind(predicate.ty) else {
            return Ok(());
        };
        if self.facts.associated.callable_output()
            != Some(crate::resolve::model::DefId(application.item.0))
            || !matches!(self.facts.types.kind(application.subject), Some(Kind::Opaque(actual, _)) if actual == id)
        {
            return Ok(());
        }
        let application = self
            .facts
            .types
            .trait_application(application.trait_application)
            .ok_or_else(|| self.error(node, "missing contextual callable application"))?;
        let mut arguments = self
            .facts
            .types
            .arguments(application.arguments)
            .ok_or_else(|| self.error(node, "missing contextual callable inputs"))?;
        let Some(Argument::Type(tuple)) = arguments.next() else {
            return Err(self.error(node, "contextual callable requires one argument tuple"));
        };
        if arguments.next().is_some() {
            return Err(self.error(node, "contextual callable has extra arguments"));
        }
        let Some(Kind::Tuple(parameters)) = self.facts.types.kind(tuple) else {
            return Ok(());
        };
        let signature = self
            .facts
            .types
            .intern(Kind::Function {
                parameters,
                result,
                bound_regions: regions,
                unsafe_: false,
                convention: crate::types::callable::Convention::RUST,
            })
            .map_err(|e| self.error(node, e))?;
        self.state
            .opaque_callables
            .get_or_insert_with(Default::default)
            .entry(term)
            .or_insert(signature);
        Ok(())
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
    // Open this signature's binder once over all roots, without interning a
    // transport-only type. Shared partial graphs and holes retain identity.
    context
        .map_region_roots(store, &children, &mut interface.opening)
        .map_err(|_| "invalid contextual lifetime mapping")
}
