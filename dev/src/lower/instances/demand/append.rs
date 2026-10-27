//! The same demand loop suspends its checked borrow before a selected proof append.
use super::{Checked, Error, Instance, Kind, Report, diagnostic, proof};
use crate::{
    borrow::templates::Retained,
    sema::{dispatch::Preparation, model::Declarations},
};

pub(super) enum Proofs<'a, 'templates> {
    Borrowed(&'a Checked<'templates>),
    Retained(&'a mut Option<Retained>),
}
impl Proofs<'_, '_> {
    pub(super) fn with_checked<R>(
        &mut self,
        report: &Report,
        run: impl for<'a> FnOnce(&Checked<'a>) -> Result<R, Error>,
    ) -> Result<R, Error> {
        match self {
            Self::Borrowed(checked) => run(checked),
            Self::Retained(owner) => owner
                .as_mut()
                .ok_or_else(|| {
                    diagnostic(
                        report,
                        None,
                        "selected original proof owner is consumed".into(),
                    )
                })?
                .with_checked(run)
                .map_err(|e| diagnostic(report, e.node, e.message))?,
        }
    }
    pub(super) fn demand(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        program: &mut proof::Builder,
        instance: Instance,
        service: &mut dyn Preparation,
    ) -> Result<(), Error> {
        let Some(Kind::FunctionItem { item, .. }) = facts.types.kind(instance.ty()) else {
            return Ok(());
        };
        if self.with_checked(report, |checked| Ok(checked.get(item).is_some()))? {
            return Ok(());
        }
        if facts
            .foreign
            .as_deref()
            .and_then(|rows| rows.function(crate::resolve::model::DefId(item.0)))
            .is_some_and(|function| function.linkage == crate::support::linkage::Linkage::Import)
        {
            return Ok(());
        }
        if !imported_source_function(report, facts, crate::resolve::model::DefId(item.0), service)?
        {
            return Ok(());
        }
        let original = crate::body::transported_originals::demand_selected(
            report,
            facts,
            crate::resolve::model::DefId(item.0),
            service,
        )?;
        let prepared = crate::pipeline::OriginalInput::prepare_selected(facts, &original, service)?;
        let Self::Retained(owner) = self else {
            return Err(diagnostic(
                report,
                None,
                "selected original append requires its actual retained proof owner".into(),
            ));
        };
        let retained = owner.take().ok_or_else(|| {
            diagnostic(
                report,
                None,
                "selected original proof owner is consumed".into(),
            )
        })?;
        let count = facts.destructors.implementations().len();
        let bound = retained
            .append_bound(original.contract(), count)
            .map_err(|e| diagnostic(report, e.node, e.message))?;
        service.prepay(facts, bound.work, bound.bytes)?;
        **owner =
            Some(program.append_selected_template(report, facts, retained, original, &prepared)?);
        Ok(())
    }
}

/// Classify actual ownership only; imported body/header readiness is still owed.
fn imported_source_function(
    report: &Report,
    facts: &mut Declarations,
    definition: crate::resolve::model::DefId,
    service: &mut dyn Preparation,
) -> Result<bool, Error> {
    service.prepay(facts, 2, 0)?;
    let actual = report
        .resolution
        .definitions
        .get(definition.0 as usize)
        .filter(|definition| definition.kind == crate::resolve::model::DefKind::Function)
        .ok_or_else(|| {
            diagnostic(
                report,
                None,
                "selected body demand requires its actual callable declaration".into(),
            )
        })?;
    let catalog = facts
        .crates
        .as_deref()
        .and_then(crate::sema::crates::Linkage::origin_catalog)
        .ok_or_else(|| {
            diagnostic(
                report,
                None,
                "selected body demand lost its actual imported catalog".into(),
            )
        })?;
    let Some(imported) = catalog
        .declaration_if_imported(definition)
        .map_err(|e| diagnostic(report, None, e.into()))?
    else {
        if actual.node.is_none() {
            return Err(diagnostic(
                report,
                None,
                "source-free executable declaration lacks its actual imported catalog identity"
                    .into(),
            ));
        }
        return Ok(());
    };
    if actual.node.is_some() {
        return Err(diagnostic(
            report,
            None,
            "imported executable declaration changed its actual source-free origin".into(),
        ));
    }
    catalog
        .validate_declaration(
            imported,
            definition,
            crate::resolve::model::DefKind::Function,
        )
        .map_err(|e| diagnostic(report, None, e.into()))?;
    let mut steps = 0;
    let source = crate::sema::crates::is_source_original(facts, definition, &mut steps);
    facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(steps);
    source.map_err(|e| diagnostic(report, None, e.into()))
}

pub(in crate::lower::instances) fn materialize(
    report: &Report,
    facts: &mut Declarations,
    roots: crate::lower::roots::Selected<'_>,
    templates: Retained,
    program: proof::Builder,
    ordinary: &super::Ordinary,
    service: &mut dyn Preparation,
) -> Result<proof::Builder, Error> {
    let mut templates = Some(templates);
    super::materialize(
        report,
        facts,
        roots,
        Proofs::Retained(&mut templates),
        program,
        ordinary,
        Some(service),
    )
}
