//! Required features consume the completed source request inventory.
pub(super) mod annotated;
#[cfg(test)]
#[path = "../../../tests/invariants/required_predicates.rs"]
mod source_tests;
use super::{Checker, Error};
use crate::{
    resolve::{
        expansion::Report,
        model::{DefId, DefKind},
        unit::NodeKey,
    },
    support::{hash::HashSet, retained::Growth},
    types::parameters::environment::Environment,
};

/// Transient coverage for the existing resolved-definition publication pass.
#[derive(Default)]
pub(in crate::sema) struct RequiredFeatures {
    pending: HashSet<NodeKey>,
}
impl RequiredFeatures {
    pub(in crate::sema) fn has_pending(&self) -> bool {
        !self.pending.is_empty()
    }
    pub(in crate::sema) fn new(report: &Report) -> Result<Self, Error> {
        let Some(producer) = report.lint_producer() else {
            if report.attribute_effects.is_some() || report.package.library_policy().is_some() {
                return Err(Error::unlocated(
                    "required feature publication lost its attribute producer",
                ));
            }
            return Ok(Self::default());
        };
        let mut result = Self::default();
        let keys = producer.feature_bound_keys();
        let module = &report.package.modules[0];
        let root = NodeKey {
            file: module.file,
            node: module.root,
        };
        producer.attribute_read(
            root,
            keys.size_hint().1.ok_or_else(|| {
                Error::unlocated("feature annotation inventory lacks its traversal bound")
            })?,
        )?;
        for key in keys {
            let growth =
                Growth::map::<NodeKey, ()>(result.pending.len(), result.pending.capacity(), 1);
            producer.diagnostic_payload(key, 1 + growth.work, growth.bytes)?;
            result.pending.insert(key);
        }
        Ok(result)
    }
    pub(in crate::sema) fn declaration(
        &mut self,
        report: &Report,
        facts: &mut crate::sema::model::Declarations,
        owner: DefId,
        key: NodeKey,
    ) -> Result<(), Error> {
        if self.pending.is_empty() {
            return Ok(());
        }
        let definition = &report.resolution.definitions[owner.0 as usize];
        if !matches!(
            definition.kind,
            DefKind::Function | DefKind::Trait | DefKind::SelfType
        ) {
            return Ok(());
        }
        if definition.kind == DefKind::SelfType {
            let ast = &report.package.unit(key.file).configured.ast;
            if !matches!(ast.node(key.node).kind, crate::syntax::ast::Kind::Item(index)
                if matches!(ast.items[index as usize].kind, crate::syntax::item::ItemKind::Impl { .. }))
            {
                return Ok(());
            }
        }
        let producer = report
            .lint_producer()
            .ok_or_else(|| Error::unlocated("feature publication lost its source ledger"))?;
        producer.attribute_read(key, 1)?;
        if !self.pending.contains(&key) {
            return Ok(());
        }
        let clauses = annotated::declaration_clauses(report, facts, owner, key)?
            .ok_or_else(|| Error::unlocated("pending feature owner lost its exact bound"))?;
        if !clauses.is_empty() {
            if facts.generics.predicates.contains_key(&owner) {
                return Err(Error::unlocated(
                    "source feature predicates were already published",
                ));
            }
            let rows = &facts.generics.predicates;
            let growth = Growth::map::<DefId, Vec<crate::sema::generics::Predicate>>(
                rows.len(),
                rows.capacity(),
                1,
            );
            producer.diagnostic_payload(key, 1 + growth.work, growth.bytes)?;
            facts.generics.predicates.insert(owner, clauses);
        }
        self.pending.remove(&key);
        Ok(())
    }
    pub(in crate::sema) fn finish(self) -> Result<(), Error> {
        if let Some(key) = self
            .pending
            .into_iter()
            .min_by_key(|key| (key.file.0, key.node.get()))
        {
            return Err(Error::at(
                key.file,
                0,
                "required feature attribute lacks its resolved declaration",
            ));
        }
        Ok(())
    }
}

fn payment(
    report: &Report,
    key: NodeKey,
) -> Option<impl Fn(usize, usize) -> Result<(), &'static str> + '_> {
    report.lint_producer().map(|producer| {
        move |steps, bytes| {
            producer
                .diagnostic_payload(key, steps, bytes)
                .map_err(|error| error.message)
        }
    })
}

impl crate::sema::model::Declarations {
    pub(crate) fn source_complete_environment(
        &mut self,
        report: &Report,
        key: NodeKey,
        owner: DefId,
    ) -> Result<(), &'static str> {
        match payment(report, key) {
            Some(pay) => self
                .generics
                .complete_environment_charged(&mut self.types, owner, &pay),
            None => self.generics.complete_environment(&mut self.types, owner),
        }
    }
    pub(crate) fn source_parameter_environment(
        &self,
        report: &Report,
        key: NodeKey,
        owner: DefId,
    ) -> Result<Environment, &'static str> {
        match payment(report, key) {
            Some(pay) => self
                .generics
                .parameter_environment_charged(&self.types, owner, &pay),
            None => self.generics.parameter_environment(&self.types, owner),
        }
    }
}

impl Checker<'_> {
    pub(super) fn replace_source_bounds(
        &mut self,
        owner: DefId,
        completed: Vec<crate::sema::generics::Predicate>,
    ) -> Result<(), Error> {
        let key = self.report.resolution.definitions[owner.0 as usize].node;
        let producer = self.report.lint_producer();
        let pay = |steps, bytes| {
            let key = key.ok_or("feature bound reconstruction lost its declaration source")?;
            producer
                .ok_or("feature bound reconstruction requires its attribute producer")?
                .diagnostic_payload(key, steps, bytes)
                .map_err(|error| error.message)
        };
        self.facts
            .generics
            .replace_declaration_bounds_charged(&self.facts.types, owner, completed, &pay)
            .map_err(|error| {
                key.map_or_else(|| Error::unlocated(error), |key| self.error(key, error))
            })
    }
    pub(super) fn complete_source_environment(
        &mut self,
        key: NodeKey,
        owner: DefId,
    ) -> Result<(), Error> {
        self.facts
            .source_complete_environment(self.report, key, owner)
            .map_err(|e| self.error(key, e))
    }
    pub(super) fn elaborated_source_environment(
        &mut self,
        key: NodeKey,
        owner: DefId,
    ) -> Result<Environment, Error> {
        let result = match payment(self.report, key) {
            Some(pay) => self.facts.generics.elaborated_environment_charged(
                &mut self.facts.types,
                owner,
                &pay,
            ),
            None => self
                .facts
                .generics
                .elaborated_environment(&mut self.facts.types, owner),
        };
        result.map_err(|e| self.error(key, e))
    }
    pub(super) fn prepare_feature_policy(&mut self) -> Result<(), Error> {
        let Some(library) = self.report.package.library_policy() else {
            return Ok(());
        };
        let requests = library.predicate_requests().map_err(Error::unlocated)?;
        let root = &self.report.package.modules[0];
        let key = crate::resolve::unit::NodeKey {
            file: root.file,
            node: root.root,
        };
        let producer = self
            .report
            .lint_producer()
            .ok_or_else(|| self.error(key, "source feature policy lost its attribute producer"))?;
        let pay = |steps, bytes| {
            producer
                .diagnostic_payload(key, steps, bytes)
                .map_err(|error| error.message)
        };
        pay(
            requests.len(),
            requests
                .len()
                .saturating_mul(size_of::<crate::types::parameters::features::FeatureId>()),
        )
        .map_err(|e| self.error(key, e))?;
        let mut enabled = Vec::with_capacity(requests.len());
        for request in requests {
            enabled.push(
                self.facts
                    .types
                    .intern_feature_charged(request, &pay)
                    .map_err(|e| self.error(key, e))?,
            );
        }
        self.facts
            .publish_feature_policy(&enabled, &pay)
            .map_err(|e| self.error(key, e))
    }
}
