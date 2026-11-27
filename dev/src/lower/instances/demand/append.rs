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
        let mut closures = Vec::new();
        collect_type_closures(&facts.types, [instance.ty()], &mut closures);
        match facts.types.kind(instance.ty()) {
            Some(Kind::FunctionItem { item, .. }) => {
                if !self.with_checked(report, |checked| Ok(checked.get(item).is_some()))? {
                    let definition = crate::resolve::model::DefId(item.0);
                    if let Some(variant) = facts.members.constructor(definition) {
                        self.demand_constructor(
                            report, facts, program, definition, variant, service,
                        )?;
                    } else if !facts
                        .foreign
                        .as_deref()
                        .and_then(|rows| rows.function(definition))
                        .is_some_and(|function| {
                            function.linkage == crate::support::linkage::Linkage::Import
                        })
                        && imported_source_function(report, facts, definition, service)?
                    {
                        let original = crate::body::transported_originals::demand_selected(
                            report, facts, definition, service,
                        )?;
                        collect_body_closures(
                            &facts.types,
                            &original.template().body,
                            &mut closures,
                        );
                        self.append_demanded(report, facts, program, original, service)?;
                    }
                }
            }
            Some(Kind::Closure { identity, .. }) => {
                closures.push(identity);
            }
            _ => {}
        }
        self.demand_closures(report, facts, program, closures, service)
    }
    fn demand_closures(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        program: &mut proof::Builder,
        mut pending: Vec<crate::types::closure::ClosureId>,
        service: &mut dyn Preparation,
    ) -> Result<(), Error> {
        let mut seen = crate::support::hash::HashSet::default();
        while let Some(identity) = pending.pop() {
            if !seen.insert(identity)
                || self.with_checked(report, |checked| Ok(checked.closure(identity).is_some()))?
            {
                continue;
            }
            let mut steps = 0;
            let source =
                crate::sema::crates::is_source_closure_original(facts, identity, &mut steps);
            facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(steps);
            if !source.map_err(|e| diagnostic(report, None, e.into()))? {
                continue;
            }
            let original = crate::body::transported_originals::demand_selected_closure(
                report, facts, identity, service,
            )?;
            collect_body_closures(&facts.types, &original.template().body, &mut pending);
            self.append_demanded(report, facts, program, original, service)?;
        }
        Ok(())
    }
    fn append_demanded(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        program: &mut proof::Builder,
        original: crate::body::transported_originals::Original,
        service: &mut dyn Preparation,
    ) -> Result<(), Error> {
        let closure_owner = match original.template().body.owner {
            crate::body::ir::Owner::Closure {
                identity,
                execution: Some(_),
                ..
            } => Some(identity),
            _ => None,
        };
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
        if let Some(identity) = closure_owner {
            self.publish_closure_conditions(report, facts, identity)?;
        }
        Ok(())
    }
    fn demand_constructor(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        program: &mut proof::Builder,
        definition: crate::resolve::model::DefId,
        variant: crate::sema::members::VariantId,
        service: &mut dyn Preparation,
    ) -> Result<(), Error> {
        let Some((template, raw)) =
            crate::body::constructor_template(report, facts, definition, variant)?
        else {
            return Ok(());
        };
        let mut prepared = raw.clone();
        if facts.uses_source_copy() {
            crate::body::copy_requirements::prepare(facts, &template.body, &mut prepared)
                .map_err(|e| diagnostic(report, Some(template.body.source.into()), e))?;
        }
        prepared
            .prepare_outlives(&mut facts.types, &mut facts.lifetimes.nominal_requirements)
            .map_err(|e| diagnostic(report, Some(template.body.source.into()), e.into()))?;
        let original = crate::body::transported_originals::Original::from_parts(template, raw);
        let Self::Retained(owner) = self else {
            return Err(diagnostic(
                report,
                None,
                "selected constructor append requires its actual retained proof owner".into(),
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
        **owner = Some(
            program.append_synthesized_template(report, facts, retained, original, &prepared)?,
        );
        Ok(())
    }
    fn publish_closure_conditions(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        identity: crate::types::closure::ClosureId,
    ) -> Result<(), Error> {
        let published = self.with_checked(report, |checked| {
            let Some((template, contract, regions)) = checked.closure_proof(identity) else {
                return Ok(None);
            };
            let (steps, conditions) = crate::borrow::templates::collect_closure_conditions(
                &facts.types,
                &facts.lifetimes.nominal_requirements,
                &facts.lifetimes.callables,
                identity,
                contract,
                regions.output.as_ref(),
            )
            .map_err(|e| diagnostic(report, Some(template.body.source.into()), e.into()))?;
            Ok(Some((template.body.source, steps, conditions)))
        })?;
        if let Some((source, steps, conditions)) = published {
            facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(steps);
            facts
                .closures
                .retain_conditions(identity, source, conditions.as_ref())
                .map_err(|e| diagnostic(report, Some(source.into()), e.into()))?;
        }
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
        return Ok(false);
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

fn collect_body_closures(
    types: &crate::types::Store,
    body: &crate::body::ir::Body,
    out: &mut Vec<crate::types::closure::ClosureId>,
) {
    let roots = body
        .locals
        .iter()
        .map(|local| local.ty)
        .chain(body.values.iter().map(|value| value.ty));
    collect_type_closures(types, roots, out);
}

fn collect_type_closures(
    types: &crate::types::Store,
    roots: impl IntoIterator<Item = crate::types::TypeId>,
    out: &mut Vec<crate::types::closure::ClosureId>,
) {
    let mut visited = crate::support::hash::HashSet::default();
    let mut stack: Vec<crate::types::TypeId> = roots.into_iter().collect();
    while let Some(ty) = stack.pop() {
        if !visited.insert(ty) {
            continue;
        }
        let Some(kind) = types.kind(ty) else {
            continue;
        };
        match kind {
            Kind::Closure {
                identity,
                arguments,
                signature,
                environment,
            } => {
                out.push(identity);
                stack.push(signature);
                stack.push(environment);
                push_arg_types(types, arguments, &mut stack);
            }
            Kind::Adt(_, args)
            | Kind::Opaque(_, args)
            | Kind::Future {
                arguments: args, ..
            }
            | Kind::Dynamic {
                arguments: args, ..
            } => push_arg_types(types, args, &mut stack),
            Kind::FunctionItem {
                arguments,
                signature,
                ..
            } => {
                stack.push(signature);
                push_arg_types(types, arguments, &mut stack);
            }
            Kind::Projection(proj) => {
                stack.push(proj.subject);
                push_arg_types(types, proj.arguments, &mut stack);
                if let Some(app) = types.trait_application(proj.trait_application) {
                    push_arg_types(types, app.arguments, &mut stack);
                }
            }
            Kind::Tuple(list) => {
                if let Some(items) = types.list(list) {
                    stack.extend_from_slice(items);
                }
            }
            Kind::Function {
                parameters, result, ..
            } => {
                stack.push(result);
                if let Some(items) = types.list(parameters) {
                    stack.extend_from_slice(items);
                }
            }
            Kind::Array { element, .. }
            | Kind::ArrayTerm { element, .. }
            | Kind::Slice(element)
            | Kind::Reference {
                referent: element, ..
            }
            | Kind::Pointer {
                pointee: element, ..
            }
            | Kind::Pattern { base: element, .. } => stack.push(element),
            Kind::Never | Kind::Primitive(_) | Kind::Parameter(_) | Kind::Foreign(_) => {}
        }
    }
}

fn push_arg_types(
    types: &crate::types::Store,
    args: crate::types::GenericArgs,
    stack: &mut Vec<crate::types::TypeId>,
) {
    if let Some(items) = types.arguments(args) {
        for arg in items {
            if let crate::types::arguments::Argument::Type(ty) = arg {
                stack.push(ty);
            }
        }
    }
}
