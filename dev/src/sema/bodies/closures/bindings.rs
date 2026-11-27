//! Lexical imports share inference holes without publishing another body's bindings.
use super::{Checker, Error, NodeId, TermId, TypeId, uses::Captures};
use crate::{
    resolve::model::{DefId, Definition},
    sema::{bodies::State, model::Declarations},
    types::{
        infer::{self, Solved},
        substitute::Bound,
    },
};

pub(super) struct Constructed {
    pub identity: crate::types::closure::ClosureId,
    pub self_ty: TermId,
    pub bindings: Vec<(DefId, TermId)>,
}
impl Checker<'_> {
    pub(super) fn closure_identity(
        &mut self,
        node: NodeId,
        signature: TermId,
    ) -> Result<Constructed, Error> {
        let identity = self
            .facts
            .types
            .reserve_closure()
            .map_err(|e| self.error(node, e))?;
        let bindings = self.closure_bindings(node)?;
        let environment = if bindings.is_empty() {
            self.known(node, crate::types::TypeId::UNIT)?
        } else {
            self.variable(node, super::Domain::Any)?
        };
        let arguments = self.closure_arguments(node)?;
        let self_ty = self
            .state
            .infer
            .closure(identity, signature, environment, &arguments)
            .map_err(|e| self.error(node, e))?;
        Ok(Constructed {
            identity,
            self_ty,
            bindings,
        })
    }
    pub(super) fn closure_bindings(&mut self, node: NodeId) -> Result<Vec<(DefId, TermId)>, Error> {
        let scope = self
            .report
            .resolution
            .node_scopes
            .get(&(self.state.module, node))
            .copied()
            .ok_or_else(|| self.error(node, "missing closure lexical scope"))?;
        let source = self.key(node);
        let imports = self
            .facts
            .closures
            .lexical_bindings(&self.report.resolution, scope)
            .map_err(|message| {
                crate::sema::model::Declarations::error(self.report, source, message)
            })?
            .to_vec();
        let mut regions = crate::types::infer::RegionMapping::default();
        imports
            .into_iter()
            .map(|definition| {
                let term = self.state.locals[definition.0 as usize].ok_or_else(|| {
                    self.error(node, "closure import is outside its defining body")
                })?;
                // Enclosing bound slots belong to the parent's body, whereas
                // the closure signature introduces its own public binder.
                // Refresh all lexical roots through the same cached mapping.
                let term = if self.state.body_binders == 0 {
                    term
                } else {
                    self.alias_inferred_mapped(node, term, &mut regions)?
                };
                Ok((definition, term))
            })
            .collect()
    }
}

/// Match capture paths to parent storage while preserving the child interface.
pub(super) fn parent_paths(
    facts: &mut Declarations,
    definitions: &[Definition],
    parent: &mut State,
    captures: &Captures,
    solved: &Solved,
    canonical: &mut Bound,
    aliases: &mut Bound,
) -> Result<Vec<TypeId>, String> {
    if captures.types.len() != captures.plan.path_count() {
        return Err("capture formation inventory differs from its typed paths".into());
    }
    let mut types = Vec::with_capacity(captures.types.len());
    let mut alias_paths = Vec::with_capacity(captures.types.len());
    let parent_has_aliases = !parent.call_types.universal_aliases().is_empty();
    for ((root, prefix), term) in captures.plan.path_origins().zip(&captures.types) {
        let ty = solved.types[term.0 as usize];
        let bound = canonical.apply(&mut facts.types, ty)?;
        let use_aliases = match prefix {
            Some(prefix) => *alias_paths
                .get(prefix.index())
                .ok_or("capture formation parent must precede its projection")?,
            None if parent_has_aliases => {
                binding_uses_aliases(facts, definitions, parent, root, ty, bound, aliases)?
            }
            None => false,
        };
        alias_paths.push(use_aliases);
        types.push(if use_aliases {
            aliases.apply(&mut facts.types, ty)?
        } else {
            bound
        });
    }
    Ok(types)
}

fn binding_uses_aliases(
    facts: &mut Declarations,
    definitions: &[Definition],
    parent: &mut State,
    root: DefId,
    ty: TypeId,
    canonical: TypeId,
    aliases: &mut Bound,
) -> Result<bool, String> {
    let Some(binding) = definitions
        .get(root.0 as usize)
        .and_then(|definition| definition.node)
        .filter(|&node| parent.terms.get(node).is_some())
        .and_then(|_| parent.locals[root.0 as usize])
    else {
        // An inherited capture uses the enclosing closure's physical path.
        return Ok(false);
    };
    let free = aliases.apply(&mut facts.types, ty)?;
    if free == canonical {
        return Ok(false);
    }
    // The same body slot can appear as a bound lifetime or a free alias in
    // an inferred local. Both views come from the child interface. Select the
    // alias view only when it exactly matches the parent binding's type.
    match parent.infer.resolved(&mut facts.types, binding) {
        Ok(actual) => Ok(actual == free),
        Err(infer::Error::Unresolved | infer::Error::Normalization) => Ok(false),
        Err(error) => Err(error.to_string()),
    }
}
