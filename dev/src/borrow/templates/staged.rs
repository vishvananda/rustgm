//! Owner indexes and region proofs survive one active initialized plan at a time.
use super::{Checked, Contract, Error, State, Templates, loans, regions};
use crate::types::{ItemId, TypeId};

pub(crate) struct Batch<'a> {
    templates: &'a Templates,
    contexts: Option<&'a Templates>,
    state: State,
    proof_contracts: Option<Vec<Contract>>,
    cursor: usize,
    verifier: loans::templates::Verifier,
}
impl<'a> Batch<'a> {
    pub(super) fn new(
        templates: &'a Templates,
        contexts: Option<&'a Templates>,
        mut state: State,
        proof_contracts: Option<Vec<Contract>>,
        custom: &[ItemId],
    ) -> Result<Self, Error> {
        let count = state.regions.bodies.len();
        state
            .loans
            .try_reserve_exact(count)
            .map_err(|_| error("loan proof allocation failed"))?;
        Ok(Self {
            templates,
            contexts,
            state,
            proof_contracts,
            cursor: 0,
            verifier: loans::templates::Verifier::new(custom).map_err(error)?,
        })
    }
    fn template(&self, index: usize) -> Option<&'a crate::body::templates::Template> {
        if index < self.templates.len() {
            self.templates.get(index)
        } else {
            self.contexts
                .and_then(|c| c.bodies.get(index - self.templates.len()))
        }
    }
    pub(crate) fn next<S: std::hash::BuildHasher>(
        &mut self,
        environment: &mut crate::borrow::Environment<'_, S>,
        position: &impl Fn(
            TypeId,
            crate::body::drops::MemberQuery,
        ) -> Result<crate::body::drops::MemberPosition, &'static str>,
    ) -> Result<Option<Row<'a, '_>>, Error> {
        let index = self.cursor;
        if index == self.state.regions.bodies.len() {
            return Ok(None);
        }
        let template = self
            .template(index)
            .ok_or_else(|| error("staged cleanup lost its actual template"))?;
        loans::templates::validate(template).map_err(error)?;
        let region = self
            .state
            .regions
            .bodies
            .get_mut(index)
            .ok_or_else(|| error("staged cleanup lost its actual region result"))?;
        let original = self
            .proof_contracts
            .as_mut()
            .unwrap_or(&mut self.state.contracts)
            .get_mut(index)
            .ok_or_else(|| error("staged cleanup lost its actual input"))?;
        let plan = self.verifier.prepare(
            environment,
            &template.body,
            &template.destruction,
            &template.environment,
            region,
            position,
        )?;
        Ok(Some(Row {
            plan,
            original,
            updated: None,
            region,
            verifier: &mut self.verifier,
            loans: &mut self.state.loans,
            cursor: &mut self.cursor,
        }))
    }
    pub(super) fn complete_ordinary<S: std::hash::BuildHasher>(
        &mut self,
        environment: &mut crate::borrow::Environment<'_, S>,
        position: impl Fn(
            TypeId,
            crate::body::drops::MemberQuery,
        ) -> Result<crate::body::drops::MemberPosition, &'static str>,
    ) -> Result<(), Error> {
        while let Some(template) = self.template(self.cursor) {
            let region = self
                .state
                .regions
                .bodies
                .get(self.cursor)
                .ok_or_else(|| error("staged ordinary proof lost its region result"))?;
            loans::templates::validate(template).map_err(error)?;
            self.state.loans.push(self.verifier.body(
                environment,
                &template.body,
                &template.destruction,
                &template.environment,
                region,
                &position,
            )?);
            self.cursor += 1;
        }
        Ok(())
    }
    pub(crate) fn finish(
        mut self,
        types: &crate::types::Store,
    ) -> Result<Checked<'a>, Error> {
        if self.cursor != self.state.regions.bodies.len() || self.state.loans.len() != self.cursor {
            return Err(error("staged cleanup proofs have incomplete body coverage"));
        }
        self.state.future_arena = super::future_arena(types, self.templates, &self.state.anonymous)?;
        self.state.regions.bodies.truncate(self.templates.len());
        self.state.loans.truncate(self.templates.len());
        if let Some(proof) = self.proof_contracts {
            for (index, input) in proof.into_iter().take(self.templates.len()).enumerate() {
                if input.source.source().is_some() {
                    self.state.contracts[index] = input;
                }
            }
        }
        self.state.contracts.truncate(self.templates.len());
        Ok(Checked {
            templates: self.templates,
            state: self.state,
        })
    }
}
pub(crate) struct Row<'a, 'p> {
    plan: loans::templates::Prepared<'a>,
    original: &'p mut Contract,
    updated: Option<Contract>,
    region: &'p mut regions::Body,
    verifier: &'p mut loans::templates::Verifier,
    loans: &'p mut Vec<loans::Analysis>,
    cursor: &'p mut usize,
}
impl Row<'_, '_> {
    pub(crate) fn input(&mut self) -> (&loans::templates::Prepared<'_>, &mut Contract) {
        (
            &self.plan,
            self.updated.get_or_insert_with(|| self.original.clone()),
        )
    }
    pub(crate) fn active(&self) -> Result<bool, &'static str> {
        self.plan.active_const()
    }
    pub(crate) fn refresh(
        &mut self,
        types: &crate::types::Store,
        variances: &crate::types::variance::Variances,
        queries: &mut crate::types::outlives::Queries<'_>,
    ) -> Result<(), Error> {
        let Some(input) = &self.updated else {
            return Ok(());
        };
        // Source inputs remain owned here. Semantic preparation only appends its
        // selected obligations and replaces the authenticated cold defining state.
        if input.key != self.original.key
            || input.source != self.original.source
            || input.signature != self.original.signature
            || input.universals != self.original.universals
            || input.assumptions != self.original.assumptions
            || input.parameters != self.original.parameters
            || input.type_assumptions != self.original.type_assumptions
        {
            return Err(error(
                "const cleanup selection changed its actual input contract",
            ));
        }
        *self.region = regions::check_one(types, variances, queries, input, self.plan.body())
            .map_err(|e| Error {
                node: e.source,
                message: e.message,
            })?;
        Ok(())
    }
    pub(crate) fn finish<S: std::hash::BuildHasher>(
        self,
        environment: &mut crate::borrow::Environment<'_, S>,
    ) -> Result<(), Error> {
        let analysis = if self.updated.is_some() {
            self.verifier.finish(environment, self.plan, self.region)?
        } else {
            self.verifier
                .finish_initial(environment, self.plan, self.region)?
        };
        if let Some(updated) = self.updated {
            *self.original = updated;
        }
        self.loans.push(analysis);
        *self.cursor += 1;
        Ok(())
    }
}
fn error(message: &str) -> Error {
    Error {
        node: None,
        message: message.into(),
    }
}
