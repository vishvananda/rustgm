//! Demand promotion from typed CFG values, retaining original runtime checks/effects.
mod operands;
mod places;
mod queries;
mod session;
pub(crate) use session::Scratch;
mod values;
use super::ir::{Body, Program, ProjectionId, Rvalue, ValueId};
use crate::{
    resolve::{expansion::Report, package::Error as Diagnostic},
    sema::{
        model::Declarations,
        values::{Constant, Evaluator, Failure},
    },
    types::Kind as Type,
};
use std::collections::HashMap;

/// Completed promotion facts, distinct from storage-liveness and loan proofs.
#[derive(Clone, Default)]
pub struct Facts {
    entries: Vec<(ValueId, Evaluated)>,
    index: HashMap<ValueId, usize>,
    pub steps: usize,
}
impl Facts {
    pub(crate) fn allocation_bytes(&self) -> usize {
        use crate::support::retained::{map, sum, vector};
        sum([vector(&self.entries), map(&self.index)])
    }
    #[must_use]
    pub fn get(&self, root: ValueId) -> Option<Constant> {
        self.index
            .get(&root)
            .and_then(|&index| self.entries[index].1.constant())
    }
    /// Eligibility proves permanent storage even when the value awaits instantiation.
    #[must_use]
    pub fn contains(&self, root: ValueId) -> bool {
        self.index.contains_key(&root)
    }
    pub fn entries(&self) -> impl Iterator<Item = (ValueId, Constant)> + '_ {
        self.entries
            .iter()
            .filter_map(|&(id, value)| value.constant().map(|value| (id, value)))
    }
    pub fn deferred(&self) -> impl Iterator<Item = ValueId> + '_ {
        self.entries
            .iter()
            .filter_map(|&(id, value)| matches!(value, Evaluated::Deferred).then_some(id))
    }
}
/// Evaluate only demanded temporary DAGs with the shared constant value engine.
/// Original statements remain: promotion cannot remove effects or overflow checks.
/// # Errors
/// Preserves required-constant failures and diagnoses malformed or exhausted queries.
pub fn analyze(
    report: &Report,
    facts: &mut Declarations,
    program: &mut Program,
) -> Result<(), Diagnostic> {
    if !program.bodies.iter().any(needs_analysis) {
        return Ok(());
    }
    let mut evaluator = Evaluator::new(report, facts);
    for (index, body) in program.bodies.iter_mut().enumerate() {
        if body.promotions.is_some() {
            continue;
        }
        let drop_values = program
            .drops
            .as_ref()
            .and_then(|p| p.bodies.get(index))
            .map(|b| b.values.as_slice());
        analyze_body(report, &mut evaluator, body, drop_values)?;
    }
    Ok(())
}
/// Evaluate demanded promotions using each template's scoped destruction facts.
/// This supplies provenance, not loan checking or an executable specialization.
/// # Errors
/// Reports invalid constants and query limits. Deferred values authorize no execution.
pub fn analyze_templates(
    report: &Report,
    facts: &mut Declarations,
    templates: &mut super::templates::Templates,
) -> Result<(), Diagnostic> {
    analyze_templates_reusing(report, facts, templates, &mut Scratch::default())
}
pub(crate) fn analyze_templates_reusing(
    report: &Report,
    facts: &mut Declarations,
    templates: &mut super::templates::Templates,
    scratch: &mut Scratch,
) -> Result<(), Diagnostic> {
    if !templates.bodies.iter().any(|t| needs_analysis(&t.body)) {
        return Ok(());
    }
    let mut evaluator = scratch.evaluator(report, facts);
    for template in &mut templates.bodies {
        analyze_body(
            report,
            &mut evaluator,
            &mut template.body,
            Some(&template.destruction.values),
        )?;
    }
    scratch.retain(evaluator);
    Ok(())
}
fn needs_analysis(body: &Body) -> bool {
    body.promotions.is_none()
        && body
            .values
            .iter()
            .any(|v| matches!(v.kind, Rvalue::Borrow { .. }))
}
/// Complete one new instance without rescanning previously prepared bodies.
/// # Errors
/// Reports the same constant, eligibility and work-limit failures as `analyze`.
pub fn instance(
    report: &Report,
    facts: &mut Declarations,
    body: &mut Body,
    destruction: &super::drops::Requirements,
) -> Result<(), Diagnostic> {
    if !needs_analysis(body) {
        return Ok(());
    }
    analyze_body(
        report,
        &mut Evaluator::new(report, facts),
        body,
        Some(&destruction.values),
    )
}
pub(crate) fn instance_reusing(
    report: &Report,
    facts: &mut Declarations,
    body: &mut Body,
    destruction: &super::drops::Requirements,
    scratch: &mut Scratch,
) -> Result<(), Diagnostic> {
    if !needs_analysis(body) {
        return Ok(());
    }
    let mut evaluator = scratch.evaluator(report, facts);
    analyze_body(report, &mut evaluator, body, Some(&destruction.values))?;
    scratch.retain(evaluator);
    Ok(())
}
fn analyze_body(
    report: &Report,
    evaluator: &mut Evaluator<'_>,
    body: &mut Body,
    drop_values: Option<&[bool]>,
) -> Result<(), Diagnostic> {
    if body.promotions.is_some() {
        return Ok(());
    }
    let mut engine = Engine::new(report, body, evaluator, drop_values);
    let output = engine.collect().map_err(|error| match error {
        Error::Constant(error) => error,
        Error::Evaluation(error) => {
            let span = report
                .package
                .unit(body.source.file)
                .configured
                .ast
                .node(body.source.node)
                .span;
            report.diagnostic(Diagnostic::at(
                body.source.file,
                span.start,
                error.message(),
            ))
        }
    })?;
    body.promotions = Some(output);
    Ok(())
}
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Key {
    Value(ValueId),
    Projection(ProjectionId),
}
#[derive(Clone, Copy)]
enum Evaluated {
    Constant(Constant),
    Deferred,
}
impl Evaluated {
    const fn constant(self) -> Option<Constant> {
        match self {
            Self::Constant(value) => Some(value),
            Self::Deferred => None,
        }
    }
}
#[derive(Clone, Copy)]
struct Ready {
    value: Evaluated,
    immediate: bool,
    constant_assignment: bool,
}
#[derive(Clone, Copy)]
enum State {
    Active,
    Runtime,
    Ready(Ready),
}
enum Error {
    Evaluation(Failure),
    Constant(Diagnostic),
}
impl From<Failure> for Error {
    fn from(value: Failure) -> Self {
        Self::Evaluation(value)
    }
}
impl From<&'static str> for Error {
    fn from(value: &'static str) -> Self {
        Self::Evaluation(Failure::Invalid(value))
    }
}
struct Engine<'a, 'b> {
    drop_values: Option<&'a [bool]>,
    report: &'a Report,
    sources: Vec<Option<crate::syntax::ast::NodeId>>,
    body: &'a Body,
    evaluator: &'a mut Evaluator<'b>,
    states: HashMap<Key, State>,
    work: Vec<(Key, bool)>,
    roots: Vec<Option<ValueId>>,
    steps: usize,
    limit: usize,
}
impl<'a, 'b> Engine<'a, 'b> {
    fn new(
        report: &'a Report,
        body: &'a Body,
        evaluator: &'a mut Evaluator<'b>,
        drop_values: Option<&'a [bool]>,
    ) -> Self {
        Self {
            drop_values,
            report,
            sources: Vec::new(),
            body,
            evaluator,
            states: HashMap::new(),
            work: Vec::new(),
            roots: Vec::new(),
            steps: 0,
            limit: 1_000_000,
        }
    }
    fn collect(&mut self) -> Result<Facts, Error> {
        let mut facts = Facts::default();
        if !self
            .body
            .values
            .iter()
            .any(|v| matches!(v.kind, Rvalue::Borrow { .. }))
        {
            return Ok(facts);
        }
        self.index_roots()?;
        for value in &self.body.values {
            self.charge(1)?;
            let Rvalue::Borrow { place, .. } = value.kind else {
                continue;
            };
            if !self.borrow_allowed(value.ty)? {
                continue;
            }
            let Some(root) = self.root(place)? else {
                continue;
            };
            if self
                .drop_values
                .and_then(|values| values.get(root.0 as usize))
                .copied()
                == Some(true)
            {
                continue;
            }
            if facts.index.contains_key(&root) {
                continue;
            }
            if let Some(value) = self.demand(Key::Value(root))? {
                if !self.storage_allowed(root, value.value)? {
                    continue;
                }
                facts.index.insert(root, facts.entries.len());
                facts.entries.push((root, value.value));
            }
        }
        facts.steps = self.steps;
        Ok(facts)
    }
    fn storage_allowed(&mut self, root: ValueId, value: Evaluated) -> Result<bool, Error> {
        use crate::types::interior_mutability::{Answer, Query};
        let declarations = self.evaluator.facts();
        if !declarations.aggregates.has_shared_mutation() {
            return Ok(true);
        }
        let ty = self.value(root)?.ty;
        let (answer, work) = match value {
            Evaluated::Constant(value) => {
                crate::sema::values::Reader::new(declarations).shared_mutation_work(ty, value)?
            }
            Evaluated::Deferred => {
                let mut query = Query::new(&declarations.types, &declarations.aggregates);
                (query.contains(ty)?, query.work().steps)
            }
        };
        self.charge(work)?;
        Ok(answer == Answer::Ordinary)
    }
    fn charge(&mut self, count: usize) -> Result<(), Error> {
        self.steps = self
            .steps
            .checked_add(count)
            .filter(|n| *n <= self.limit)
            .ok_or("promotion query work budget exceeded")?;
        Ok(())
    }
    fn borrow_allowed(&self, ty: crate::types::TypeId) -> Result<bool, Error> {
        match self.evaluator.facts().types.kind(ty) {
            Some(Type::Reference { mutable: false, .. }) => Ok(true),
            Some(Type::Reference {
                mutable: true,
                referent,
                ..
            }) => Ok(matches!(
                self.evaluator.facts().types.kind(referent),
                Some(Type::Array { length: 0, .. })
            )),
            _ => Err("borrow value lacks a reference type".into()),
        }
    }
}
