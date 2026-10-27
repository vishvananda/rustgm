//! Iterative depth-first demand distinguishes recursion from unbounded instance growth.
use super::staged_originals::{Collect, Context as Originals};
use super::{dependencies::Dependencies, diagnostic, proof};
use crate::support::hash::{HashMap, HashSet};
use crate::{
    body::{
        self,
        ir::{Body, BodyKey, Owner},
        regions::Contract,
    },
    borrow::templates::Checked,
    resolve::{expansion::Report, model::DefId, package::Error},
    sema::model::Declarations,
    types::{ItemId, Kind, instance::Instance},
};
mod paid_owners;

enum Work {
    Adapter(body::invocation_entries::Entry, body::ir::ProofSite),
    Enter(Instance, body::ir::ProofSite),
    Leave(BodyKey),
}
#[derive(Default)]
struct Ordinary {
    functions: HashMap<ItemId, usize>,
    closures: HashMap<crate::types::closure::ClosureId, usize>,
}
/// A source family selects either its checked template or existing concrete body.
struct Selected<'a> {
    family: BodyKey,
    template: Option<(&'a body::templates::Template, &'a Contract)>,
    ordinary: Option<usize>,
}
impl Selected<'_> {
    fn concrete(&self, visited: &mut HashSet<usize>) -> Result<Option<usize>, &'static str> {
        let index = self
            .ordinary
            .ok_or("demanded callable has no concrete body or checked template")?;
        Ok(visited.insert(index).then_some(index))
    }
}
impl Ordinary {
    fn adapter_source(
        &self,
        types: &crate::types::Store,
        templates: &Checked<'_>,
        instances: &HashMap<Instance, usize>,
        entry: body::invocation_entries::Entry,
    ) -> Result<Option<usize>, &'static str> {
        let adapter = entry
            .adapter()
            .ok_or("adapter demand lacks its selected identity")?;
        if adapter.instance() != entry.instance() {
            return Err("adapter demand selects a different source instance");
        }
        let Some(Kind::Closure { identity, .. }) = types.kind(adapter.instance().ty()) else {
            return Err("adapter demand has no closure identity");
        };
        if templates.closure(identity).is_some() {
            return Ok(instances.get(&adapter.instance()).copied());
        }
        self.closures
            .get(&identity)
            .copied()
            .map(Some)
            .ok_or("adapter demand requires a proved ordinary source body")
    }
    fn select<'a>(
        &self,
        facts: &Declarations,
        templates: &'a Checked<'_>,
        instance: Instance,
    ) -> Result<Option<Selected<'a>>, &'static str> {
        Ok(Some(match facts.types.kind(instance.ty()) {
            Some(Kind::FunctionItem { item, .. }) => {
                // Actual helpers still follow ordinary original/materialized proofs.
                // The typed descriptor only gates their later cleanup adapter.
                crate::sema::compiler_items::source_drop_glue(facts, instance)?;
                if let Some(kind) = facts
                    .source_bindings
                    .as_deref()
                    .map(|c| c.intrinsic_kind(DefId(item.0)))
                    .transpose()?
                    .flatten()
                {
                    return Err(match kind {
                        crate::support::bindings::SourceIntrinsic::Layout(_) => {
                            "source layout intrinsic runtime lowering is not implemented"
                        }
                        crate::support::bindings::SourceIntrinsic::NeedsDrop => {
                            "source needs_drop intrinsic must be evaluated in a const block"
                        }
                        crate::support::bindings::SourceIntrinsic::ConstEvalSelect => {
                            "source callback selection requires its proved reached runtime operation"
                        }
                        crate::support::bindings::SourceIntrinsic::Transmute => {
                            "source transmute requires its proved direct runtime operation"
                        }
                        crate::support::bindings::SourceIntrinsic::Abort => {
                            "source abort requires its proved reached runtime operation"
                        }
                        crate::support::bindings::SourceIntrinsic::UbChecks => {
                            "source ub_checks requires its proved direct runtime operation"
                        }
                    });
                }
                if templates.get(item).is_none()
                    && facts
                        .foreign
                        .as_deref()
                        .and_then(|r| r.function(DefId(item.0)))
                        .is_some_and(|f| f.linkage == crate::support::linkage::Linkage::Import)
                {
                    return Ok(None);
                }
                let template = templates
                    .get(item)
                    .map(|template| {
                        templates
                            .input(item)
                            .map(|input| (template, input))
                            .ok_or("missing checked template input")
                    })
                    .transpose()?;
                Selected {
                    family: BodyKey::Declaration(DefId(item.0)),
                    template,
                    ordinary: self.functions.get(&item).copied(),
                }
            }
            Some(Kind::Closure { identity, .. }) => Selected {
                family: BodyKey::Closure(identity),
                template: templates.closure(identity),
                ordinary: self.closures.get(&identity).copied(),
            },
            _ => return Err("invalid demanded instance"),
        }))
    }
}
#[derive(Default)]
struct Budget {
    active: HashMap<BodyKey, usize>,
    bytes: usize,
    instances: usize,
    adapters: HashSet<crate::types::closure::Adapter>,
}
impl Budget {
    fn adapter(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        templates: &Checked<'_>,
        program: &mut proof::Builder,
        original: usize,
        request: (body::invocation_entries::Entry, body::ir::ProofSite),
    ) -> Result<Option<(usize, BodyKey)>, Error> {
        let (entry, source) = request;
        let error = |e: &str| diagnostic(report, Some(source), e.into());
        let adapter = entry
            .adapter()
            .ok_or_else(|| error("missing adapter demand identity"))?;
        if !self.adapters.insert(adapter) {
            return Ok(None);
        }
        let source = original;
        let original = program
            .body(source)
            .map_err(|e| diagnostic(report, None, e.into()))?;
        let prepared = body::adapters::build(facts, entry, original)
            .map_err(|e| diagnostic(report, Some(original.source.into()), e.into()))?;
        let family = prepared.body.key();
        self.enter(family, &prepared.body, &prepared.input)
            .map_err(|e| diagnostic(report, Some(prepared.body.source.into()), e.into()))?;
        let index = program
            .push_adapter(report, facts, templates, source, prepared)
            .map_err(|e| diagnostic(report, e.node, e.message))?;
        Ok(Some((index, family)))
    }
    fn leave(&mut self, report: &Report, item: BodyKey) -> Result<(), Error> {
        let error = |message: &str| diagnostic(report, None, message.into());
        let active = self
            .active
            .get_mut(&item)
            .ok_or_else(|| error("invalid active instance stack"))?;
        *active = active
            .checked_sub(1)
            .ok_or_else(|| error("unbalanced instance stack"))?;
        Ok(())
    }
    fn factory(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        templates: &Checked<'_>,
        program: &mut proof::Builder,
        selected: crate::sema::primitive_methods::Selected,
    ) -> Result<(usize, BodyKey), Error> {
        let (body, input) = body::primitive_methods::build(report, facts, selected)?;
        let family = body.key();
        self.enter(family, &body, &input)
            .map_err(|e| diagnostic(report, Some(body.source.into()), e.into()))?;
        let index = program
            .push_factory(facts, templates, selected, body, &input)
            .map_err(|e| diagnostic(report, e.node, e.message))?;
        Ok((index, family))
    }
    fn enter(&mut self, item: BodyKey, body: &Body, input: &Contract) -> Result<(), &'static str> {
        let active = self.active.entry(item).or_default();
        if *active >= 128 {
            return Err("recursive generic instantiation exceeds 128 active instances");
        }
        self.instances += 1;
        if self.instances > 100_000 {
            return Err("generic instantiation exceeds 100000 bodies");
        }
        let bytes = body
            .retained_bytes()
            .saturating_add(std::mem::size_of_val(input.obligations.as_slice()))
            .saturating_add(std::mem::size_of_val(input.assumptions.as_slice()))
            .saturating_add(std::mem::size_of_val(input.type_assumptions.as_slice()));
        self.bytes = self.bytes.saturating_add(bytes);
        if self.bytes > 64_000_000 {
            return Err("generic instance bodies exceed 64 MB budget");
        }
        *active += 1;
        Ok(())
    }
}
pub(super) fn prepare<C: Collect>(
    report: &Report,
    facts: &mut Declarations,
    roots: crate::lower::roots::Selected<'_>,
    mut originals: Originals<'_, '_, C>,
    inputs: Vec<Contract>,
    concrete: HashMap<BodyKey, Body>,
    constants: Option<crate::pipeline::execution::Context>,
) -> Result<proof::Builder, Error> {
    let (program, ordinary) = prepare_ordinary(
        report,
        facts,
        &mut originals,
        inputs,
        concrete,
        constants,
        roots.globals,
    )?;
    let templates = originals.checked;
    // Discovery includes implicit constructor factories. With no templates,
    // ordinary lowering already follows calls and selected cleanup dependencies.
    // All source bodies have still been checked before taking this path.
    if templates.is_empty()
        && !program.has_adapters()
        && !facts.types.has_dynamic_shapes()
        && facts
            .source_bindings
            .as_deref()
            .is_none_or(|catalog| !catalog.has_intrinsics())
        && facts
            .primitive_methods
            .as_deref()
            .is_none_or(crate::sema::primitive_methods::Registry::is_empty)
    {
        return Ok(program);
    }
    materialize(report, facts, roots, templates, program, &ordinary)
}

fn materialize(
    report: &Report,
    facts: &mut Declarations,
    roots: crate::lower::roots::Selected<'_>,
    templates: &Checked<'_>,
    mut program: proof::Builder,
    ordinary: &Ordinary,
) -> Result<proof::Builder, Error> {
    let (mut work, mut dependencies) = initial_work(report, facts, roots, &mut program, ordinary)?;
    let mut instances = HashMap::default();
    let mut visited = HashSet::default();
    let mut budget = Budget::default();
    let mut cleanup = body::instantiate::Cleanup::new(facts);
    while let Some(action) = work.pop() {
        let (instance, source) = match action {
            Work::Adapter(entry, source) => {
                let original = ordinary
                    .adapter_source(&facts.types, templates, &instances, entry)
                    .map_err(|e| diagnostic(report, Some(source), e.into()))?;
                let Some(original) = original else {
                    work.push(Work::Adapter(entry, source));
                    work.push(Work::Enter(entry.instance(), source));
                    continue;
                };
                if let Some((index, family)) = budget.adapter(
                    report,
                    facts,
                    templates,
                    &mut program,
                    original,
                    (entry, source),
                )? {
                    work.push(Work::Leave(family));
                    dependencies.schedule(
                        report,
                        facts,
                        &mut program,
                        DependencyRoot::Body(index),
                        &mut work,
                    )?;
                }
                continue;
            }
            Work::Enter(instance, source) => (instance, source),
            Work::Leave(item) => {
                budget.leave(report, item)?;
                continue;
            }
        };
        let error = |e: &str| diagnostic(report, Some(source), e.into());
        let index = if let Some(selected) = facts
            .primitive_methods
            .as_deref()
            .and_then(|registry| registry.get(instance))
        {
            if instances.contains_key(&instance) {
                continue;
            }
            let (index, family) =
                budget.factory(report, facts, templates, &mut program, selected)?;
            instances.insert(instance, index);
            work.push(Work::Leave(family));
            index
        } else {
            let Some(selected) = ordinary.select(facts, templates, instance).map_err(error)? else {
                continue;
            };
            if let Some((template, input)) = selected.template {
                if instances.contains_key(&instance) {
                    continue;
                }
                budget
                    .enter(selected.family, &template.body, input)
                    .map_err(error)?;
                let request = Specialization {
                    template,
                    input,
                    instance,
                    source,
                };
                let index = specialize(
                    report,
                    facts,
                    templates,
                    &mut program,
                    &mut cleanup,
                    request,
                )?;
                instances.insert(instance, index);
                work.push(Work::Leave(selected.family));
                index
            } else {
                let Some(index) = selected.concrete(&mut visited).map_err(error)? else {
                    continue;
                };
                index
            }
        };
        dependencies.schedule(
            report,
            facts,
            &mut program,
            DependencyRoot::Body(index),
            &mut work,
        )?;
    }
    Ok(program)
}

#[derive(Clone, Copy)]
struct Specialization<'a> {
    template: &'a body::templates::Template,
    input: &'a Contract,
    instance: Instance,
    source: body::ir::ProofSite,
}
// Entered dependencies come only from a body sealed by Builder::push.
fn specialize(
    report: &Report,
    facts: &mut Declarations,
    templates: &Checked<'_>,
    program: &mut proof::Builder,
    cleanup: &mut body::instantiate::Cleanup,
    request: Specialization<'_>,
) -> Result<usize, Error> {
    let error = |e: &str| diagnostic(report, Some(request.source), e.into());
    let mut materialized = body::instantiate::from_checked_caller(
        facts,
        request.template,
        request.instance,
        request.input,
        cleanup,
    )
    .map_err(error)?;
    materialized.resolve(facts).map_err(|e| error(&e))?;
    facts.prepare_opaque_storage().map_err(error)?;
    body::promote::instance(
        report,
        facts,
        &mut materialized.body,
        &materialized.destruction,
    )?;
    body::constants::instance(report, facts, &materialized.body)?;
    program
        .push(
            facts,
            templates,
            materialized.body,
            &materialized.contract,
            Some(materialized.destruction),
        )
        .map_err(|e| diagnostic(report, e.node, e.message))
}

fn initial_work(
    report: &Report,
    facts: &mut Declarations,
    roots: crate::lower::roots::Selected<'_>,
    program: &mut proof::Builder,
    ordinary: &Ordinary,
) -> Result<(Vec<Work>, Dependencies), Error> {
    let mut work = Vec::new();
    for definition in roots.functions.iter().rev() {
        if facts
            .foreign
            .as_deref()
            .and_then(|registry| registry.function(definition))
            .is_some_and(|function| function.linkage == crate::support::linkage::Linkage::Import)
        {
            continue;
        }
        let source = ordinary
            .functions
            .get(&ItemId(definition.0))
            .and_then(|&i| program.body(i).ok().map(|body| body.source))
            .ok_or_else(|| diagnostic(report, None, "missing root body".into()))?;
        let ty = facts
            .definitions
            .get(definition.0 as usize)
            .copied()
            .flatten()
            .ok_or_else(|| diagnostic(report, Some(source.into()), "missing root type".into()))?;
        let instance = Instance::new(&facts.types, ty)
            .map_err(|e| diagnostic(report, Some(source.into()), e.into()))?;
        work.push(Work::Enter(instance, source.into()));
    }
    let dependencies = Dependencies::roots(report, facts, program, roots.globals, &mut work)?;
    Ok((work, dependencies))
}

#[derive(Clone, Copy)]
enum DependencyRoot {
    Body(usize),
    Global(DefId),
}
impl Dependencies {
    fn roots(
        report: &Report,
        facts: &mut Declarations,
        program: &mut proof::Builder,
        globals: &[DefId],
        work: &mut Vec<Work>,
    ) -> Result<Self, Error> {
        let mut dependencies = Self::default();
        for &definition in globals {
            dependencies.schedule(
                report,
                facts,
                program,
                DependencyRoot::Global(definition),
                work,
            )?;
        }
        Ok(dependencies)
    }
    fn schedule(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        program: &mut proof::Builder,
        root: DependencyRoot,
        work: &mut Vec<Work>,
    ) -> Result<(), Error> {
        let (source, demands, cleanup) = match root {
            DependencyRoot::Body(index) => {
                let body = program
                    .body(index)
                    .map_err(|e| diagnostic(report, None, e.into()))?;
                let source = body.source;
                let demands = self
                    .collect(report, facts, body)
                    .map_err(|e| diagnostic(report, Some(source.into()), e))?;
                let cleanup = program
                    .cleanup_dependencies(facts, index)
                    .map_err(|e| diagnostic(report, Some(source.into()), e.into()))?;
                (source.root_site(), demands, cleanup)
            }
            DependencyRoot::Global(definition) => {
                let initializer = facts
                    .initializer_index(definition)
                    .and_then(|index| facts.initializers.get(index))
                    .ok_or_else(|| {
                        diagnostic(
                            report,
                            None,
                            "global export requires a local initializer".into(),
                        )
                    })?;
                let source = initializer.value;
                let demands = self
                    .global(report, facts, definition)
                    .map_err(|e| diagnostic(report, Some(source.into()), e))?;
                (source.into(), demands, Vec::new())
            }
        };
        let error = |e: &str| diagnostic(report, Some(source), e.into());
        work.extend(
            demands
                .adapters
                .into_iter()
                .rev()
                .map(|entry| Work::Adapter(entry, source)),
        );
        for ty in demands.closures {
            let instance = Instance::closure(&facts.types, ty).map_err(error)?;
            work.push(Work::Enter(instance, source));
        }
        let mut next = demands.calls;
        next.extend(
            program
                .object_dependencies(facts, &demands.object_types)
                .map_err(error)?,
        );
        next.extend(cleanup);
        work.extend(next.into_iter().rev().map(|i| Work::Enter(i, source)));
        Ok(())
    }
}

fn prepare_ordinary<C: Collect>(
    report: &Report,
    facts: &mut Declarations,
    originals: &mut Originals<'_, '_, C>,
    inputs: Vec<Contract>,
    concrete: HashMap<BodyKey, Body>,
    constants: Option<crate::pipeline::execution::Context>,
    globals: &[DefId],
) -> Result<(proof::Builder, Ordinary), Error> {
    let templates = originals.checked;
    let mut program = body::concrete_prepared(
        report,
        facts,
        |owner| templates.get(ItemId(owner.0)).is_some(),
        concrete,
    )?;
    let mut inputs: HashMap<_, _> = inputs.into_iter().map(|c| (c.key, c)).collect();
    for body in &mut program.bodies {
        let input = inputs.get_mut(&body.key()).ok_or_else(|| {
            diagnostic(
                report,
                Some(body.source.into()),
                "missing ordinary dispatch contract".into(),
            )
        })?;
        if let Some(service) = &mut originals.preparation {
            body::dispatch::resolve_prepared(facts, body, input, &mut **service).map_err(
                |failure| match failure {
                    crate::sema::dispatch::Failure::Invalid(message) => {
                        diagnostic(report, Some(body.source.into()), message)
                    }
                    crate::sema::dispatch::Failure::Preparation(error) => error,
                },
            )?;
        } else {
            body::dispatch::resolve(facts, body, input)
                .map_err(|e| diagnostic(report, Some(body.source.into()), e))?;
        }
    }
    body::promote::analyze(report, facts, &mut program)?;
    let pending = paid_owners::prepare(facts, templates, &program, &mut inputs)
        .map_err(|e| diagnostic(report, None, e.into()))?;
    if !pending.is_empty() {
        return paid_owners::build(
            report,
            facts,
            originals,
            program,
            inputs,
            pending,
            paid_owners::Demands {
                context: constants.as_ref().ok_or_else(|| {
                    diagnostic(
                        report,
                        None,
                        "conditional inline execution requires its checked provider".into(),
                    )
                })?,
                globals,
            },
        );
    }
    if let Some(context) = constants {
        context.constants(report, facts, &mut program, globals)?;
    } else {
        body::constants::analyze_bodies(report, facts, program.bodies.iter(), globals)?;
    }
    let (mut program, bodies) =
        proof::Builder::new(program).map_err(|e| diagnostic(report, None, e.into()))?;
    program
        .prepare_original_regions(facts, &inputs, &bodies)
        .map_err(|e| diagnostic(report, e.node, e.message))?;
    let mut ordinary = Ordinary::default();
    // Prove in program order while releasing the moved-from body records.
    let mut bodies = bodies;
    bodies.reverse();
    while let Some(body) = bodies.pop() {
        if bodies.len() < bodies.capacity() / 2 {
            bodies.shrink_to_fit();
        }
        let source = body.source;
        let closure = if let Owner::Closure { identity, .. } = body.owner {
            Some(identity)
        } else {
            None
        };
        let definition = if let Owner::Function { definition, .. } = body.owner {
            Some(definition)
        } else {
            None
        };
        let input = inputs.remove(&body.key()).ok_or_else(|| {
            diagnostic(
                report,
                Some(source.into()),
                "missing ordinary body contract".into(),
            )
        })?;
        let index = program.push_original(report, facts, originals, body, &input)?;
        #[cfg(test)]
        program
            .observe_original(facts, index, &input)
            .map_err(|e| diagnostic(report, e.node, e.message))?;
        if let Some(definition) = definition {
            ordinary.functions.insert(ItemId(definition.0), index);
        }
        if let Some(identity) = closure {
            ordinary.closures.insert(identity, index);
        }
    }
    Ok((program, ordinary))
}
