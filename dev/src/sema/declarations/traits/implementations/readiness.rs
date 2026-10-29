//! Demand checked constant members independently of method signature completion.
use super::{Checker, Error, Work, constants::Members};
use crate::sema::traits::{
    conformance::Definitions,
    implementations::{Availability, ConstantRequest},
};
use crate::support::hash::{HashMap, HashSet};

#[derive(Default)]
pub(in crate::sema::declarations) struct Readiness {
    pub(super) members: Option<Box<Members>>,
    active: HashSet<ConstantRequest>,
    pub(super) contracts: HashMap<ConstantRequest, super::constants::Contract>,
}

impl Readiness {
    fn members(
        &mut self,
        report: &crate::resolve::expansion::Report,
        facts: &crate::sema::model::Declarations,
    ) -> &mut Members {
        self.members.get_or_insert_with(|| {
            Box::new(if report.package.source_core().is_some() {
                Members::default()
            } else {
                Members::new(report, facts)
            })
        })
    }

    pub(super) fn take_members(
        &mut self,
        report: &crate::resolve::expansion::Report,
        facts: &crate::sema::model::Declarations,
    ) -> Members {
        self.members
            .take()
            .map_or_else(|| Members::new(report, facts), |members| *members)
    }
}

impl Checker<'_> {
    pub(in crate::sema::declarations) fn constant_member(
        &mut self,
        request: ConstantRequest,
        finish: bool,
    ) -> Result<(), Error> {
        let key = self.report.resolution.definitions[request.declaration.0 as usize]
            .node
            .ok_or_else(|| {
                Error::at(
                    crate::resolve::unit::FileId(0),
                    0,
                    "missing constant member source",
                )
            })?;
        if self
            .facts
            .generics
            .implementations
            .as_ref()
            .is_some_and(|registry| {
                registry
                    .available_constant(
                        request.owner,
                        request.declaration,
                        Availability::Declarations,
                    )
                    .is_some()
            })
        {
            return Ok(());
        }
        let selected = self
            .constant_members
            .members(self.report, &self.facts)
            .selected(
                &mut self.facts,
                self.report,
                request.owner,
                request.declaration,
            )
            .map_err(|(member, error)| {
                self.error(
                    self.report.resolution.definitions[member.0 as usize]
                        .node
                        .unwrap_or(key),
                    error,
                )
            })?;
        if !finish {
            if !self.constant_members.active.insert(request) {
                return Err(self.error(key, "cycle in associated constant member contracts"));
            }
            self.work.push(Work::ConstantMember(request, true));
            if let Some(actual) = selected {
                self.work.push(Work::Definition(actual, false));
            }
            self.work.push(Work::Definition(request.declaration, false));
            return Ok(());
        }
        let predicate = self
            .facts
            .generics
            .implementations
            .as_ref()
            .and_then(|registry| registry.header(request.owner))
            .map(|header| header.predicate)
            .ok_or_else(|| self.error(key, "missing selected constant implementation header"))?;
        // This query has declaration authority. Normalize only the member's
        // type dependencies; unrelated associated definitions can depend on its
        // value. Whole-impl conformance still gates all completed facts.
        let definitions =
            Definitions::new(&self.facts, predicate, &[]).map_err(|e| self.error(key, e))?;
        if let Some(actual) = selected {
            let mut substitution =
                crate::sema::traits::results::substitution(&self.facts, predicate)
                    .map_err(|e| self.error(key, e))?;
            let contract = self.constant_contract(
                request.owner,
                key,
                (request.declaration, actual),
                &mut substitution,
                &definitions,
            )?;
            // Selection has declaration authority only. The final impl pass
            // consumes this exact normalized contract and proves every lifetime.
            self.constant_members.contracts.insert(request, contract);
        } else if !matches!(
            self.item(key)?.kind,
            crate::syntax::item::ItemKind::Const { value: Some(_), .. }
        ) {
            return Err(self.error(
                key,
                "implementation is missing a required associated constant",
            ));
        }
        self.facts
            .generics
            .implementations
            .as_mut()
            .ok_or_else(|| Error::at(key.file, 0, "missing constant implementation registry"))?
            .complete_constant(request, selected.unwrap_or(request.declaration))
            .map_err(|e| self.error(key, e))?;
        self.constant_members.active.remove(&request);
        Ok(())
    }
    pub(super) fn finish_constant_contracts(&self) -> Result<(), Error> {
        let Some(request) = self
            .constant_members
            .contracts
            .keys()
            .min_by_key(|request| (request.owner.0, request.declaration.0))
        else {
            return Ok(());
        };
        let definition = &self.report.resolution.definitions[request.declaration.0 as usize];
        let key = definition.node.ok_or_else(|| {
            Error::at(
                crate::resolve::unit::FileId(0),
                0,
                "unproved constant contract has no source",
            )
        })?;
        Err(self.error(
            key,
            "associated constant lifetime contract was not discharged",
        ))
    }
}
