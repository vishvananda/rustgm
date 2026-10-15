//! Body proofs retain shared annotation terms and their well-formedness roots.
use super::{Checker, Context, Declarations, Error, NodeId, NodeKey, Report, TermId};
use crate::sema::annotations::{Fields, Host};

impl Checker<'_> {
    pub(super) fn annotation(&mut self, root: NodeId) -> Result<TermId, Error> {
        let mut scratch = std::mem::take(&mut self.annotations);
        let result = scratch.infer(self, self.key(root));
        self.annotations = scratch;
        result
    }
}
impl Host for Checker<'_> {
    fn report(&self) -> &Report {
        self.report
    }
    fn facts(&self) -> &Declarations {
        self.facts
    }
    fn inference(&mut self) -> (&mut Declarations, &mut Context) {
        (self.facts, &mut self.infer)
    }
    fn cached(&self, key: NodeKey) -> Option<TermId> {
        self.terms[key.file.0 as usize][key.node.0 as usize]
    }
    fn save(&mut self, key: NodeKey, term: TermId) {
        self.save(key.node, term);
    }
    fn nominal(&mut self, term: TermId, fields: Fields) {
        self.call_types.insert_nominal(term, fields);
    }
    fn requirement(&mut self, key: NodeKey, predicate: crate::types::infer::predicates::Predicate) {
        self.call_types.obligations.push((key.node, predicate));
    }
    fn callable(&mut self, key: NodeKey) {
        self.annotation_checks.push(key.node);
    }
    fn scope(
        &mut self,
        key: NodeKey,
        parent: crate::types::lifetime_proof::ScopeId,
        count: u32,
    ) -> Result<crate::types::lifetime_proof::ScopeId, Error> {
        self.projections
            .scope(parent, count)
            .map_err(|e| self.error(key.node, e))
    }
    fn deferred(
        &mut self,
        key: NodeKey,
        scope: crate::types::lifetime_proof::ScopeId,
        expansion: crate::sema::alias_requirements::Expansion,
        supplied: &[crate::sema::generics::AnnotationArgument],
        fields: &Fields,
    ) -> Result<(), Error> {
        self.defer_contract(key.node, scope, expansion, supplied, fields)
    }
    fn normalization(
        &mut self,
        key: NodeKey,
        scope: crate::types::lifetime_proof::ScopeId,
        term: TermId,
    ) -> Result<(), Error> {
        self.defer_normalization(key.node, scope, term)
    }
    fn child(&mut self, key: NodeKey) {
        self.annotation_children.insert(key.node);
    }
}
