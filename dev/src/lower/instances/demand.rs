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
mod current_caller;
mod paid_owners;
pub(super) use current_caller::Caller;
#[path = "demand/append.rs"]
mod selected_originals;
use selected_originals::Proofs;
pub(super) use selected_originals::materialize as materialize_selected;

#[derive(Clone, Copy)]
enum Work {
    Adapter(body::invocation_entries::Entry, body::ir::ProofSite, Caller),
    Enter(Instance, body::ir::ProofSite, Caller),
    Future(
        super::dependencies::FutureDemand,
        body::ir::ProofSite,
        Caller,
    ),
    FutureDrop(crate::types::TypeId, body::ir::ProofSite),
    Leave(BodyKey),
}
const _: [(); 40] = [(); size_of::<Work>()];

#[derive(Default)]
pub(super) struct Ordinary {
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
/// Source operation descriptors do not authorize an ordinary callable original.
fn require_source_body(
    facts: &Declarations,
    instance: Instance,
    item: ItemId,
) -> Result<(), &'static str> {
    // Actual helpers still follow ordinary original/materialized proofs.
    // The typed descriptor only gates their later cleanup adapter.
    crate::sema::compiler_items::source_drop_glue(facts, instance)?;
    if let Some(kind) = facts
        .source_bindings
        .as_deref()
        .map(|c| c.intrinsic_kind(DefId(item.0)))
        .transpose()?
        .flatten()
        && kind != crate::support::bindings::SourceIntrinsic::ColdPath
        && !facts
            .source_bindings
            .as_deref()
            .ok_or("source intrinsic fallback catalog is missing")?
            .guaranteed_fallback(facts, instance)?
        && !facts
            .source_bindings
            .as_deref()
            .ok_or("source knowledge fallback catalog is missing")?
            .knowledge_fallback(facts, instance)?
        && !facts
            .source_bindings
            .as_deref()
            .ok_or("source vararg fallback catalog is missing")?
            .vararg_fallback(facts, instance)?
    {
        return Err(match kind {
            crate::support::bindings::SourceIntrinsic::Declaration(_) => {
                "source intrinsic executable lowering is not implemented"
            }
            crate::support::bindings::SourceIntrinsic::Layout(_) => {
                "source layout intrinsic runtime lowering is not implemented"
            }
            crate::support::bindings::SourceIntrinsic::NeedsDrop => {
                "source needs_drop intrinsic must be evaluated in a const block"
            }
            crate::support::bindings::SourceIntrinsic::OffsetOf => {
                "source offset_of intrinsic must be evaluated in a const block"
            }
            crate::support::bindings::SourceIntrinsic::ConstEvalSelect => {
                "source callback selection requires its proved reached runtime operation"
            }
            crate::support::bindings::SourceIntrinsic::ArithOffset => {
                "source arith_offset requires an authenticated direct call"
            }
            crate::support::bindings::SourceIntrinsic::Transmute
            | crate::support::bindings::SourceIntrinsic::TransmuteUnchecked => {
                "source transmute requires its proved direct runtime operation"
            }
            crate::support::bindings::SourceIntrinsic::ThreeWayCompare => {
                "three_way_compare requires its proved direct scalar comparison operation"
            }
            crate::support::bindings::SourceIntrinsic::DiscriminantValue => {
                "source discriminant requires its proved direct reference operation"
            }
            crate::support::bindings::SourceIntrinsic::AggregateRawPtr => {
                "source aggregate_raw_ptr requires its proved direct composition operation"
            }
            crate::support::bindings::SourceIntrinsic::PtrMetadata => {
                "source ptr_metadata requires its proved direct pointer operation"
            }
            crate::support::bindings::SourceIntrinsic::CallerLocation => {
                "source caller_location requires its proved direct caller operation"
            }
            crate::support::bindings::SourceIntrinsic::Abort => {
                "source abort requires its proved reached runtime operation"
            }
            crate::support::bindings::SourceIntrinsic::UbChecks => {
                "source ub_checks requires its proved direct runtime operation"
            }
            crate::support::bindings::SourceIntrinsic::UncheckedSub
            | crate::support::bindings::SourceIntrinsic::SubWithOverflow => {
                "source subtraction requires its actual checked direct operation"
            }
            crate::support::bindings::SourceIntrinsic::ColdPath => {
                "source cold_path requires its genuine source fallback"
            }
        });
    }
    Ok(())
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
                crate::body::coroutines::require_ordinary_original(facts, DefId(item.0))?;
                require_source_body(facts, instance, item)?;
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
/// The demand budget's fixed byte cap, rustc-style: it bounds instantiation by
/// output, since demanded bodies may legitimately grow superlinearly in input.
pub(in crate::lower) const BODY_BYTES: usize = 64_000_000;
pub(in crate::lower) const BODIES_EXCEEDED: &str = "generic instance bodies exceed 64 MB budget";
#[derive(Default)]
struct Budget {
    active: HashMap<BodyKey, usize>,
    bytes: usize,
    instances: usize,
    adapters: HashSet<crate::types::closure::Adapter>,
}
impl Budget {
    fn push_work(
        &mut self,
        facts: &mut Declarations,
        work: &mut Vec<Work>,
        value: Work,
    ) -> Result<(), &'static str> {
        let growth =
            crate::support::retained::Growth::vector::<Work>(work.len(), work.capacity(), 1);
        facts.body_work.canonical_steps = facts
            .body_work
            .canonical_steps
            .saturating_add(growth.work.saturating_add(1));
        self.bytes = self.bytes.saturating_add(growth.bytes);
        if self.bytes > 64_000_000 {
            return Err("generic instance work exceeds 64 MB budget");
        }
        work.push(value);
        Ok(())
    }
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
            .push_adapter(report, facts, templates, source, prepared, &mut self.bytes)
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
    fn future_constructor(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        templates: &Checked<'_>,
        program: &mut proof::Builder,
        instance: Instance,
    ) -> Result<usize, Error> {
        let prepared = crate::pipeline::future_constructor(facts, templates, instance)
            .map_err(|e| diagnostic(report, None, e))?;
        let family = prepared.body.key();
        self.enter(family, &prepared.body, &prepared.contract)
            .map_err(|e| diagnostic(report, Some(prepared.body.source.into()), e.into()))?;
        let index = program
            .push_future_factory(facts, templates, prepared, &mut self.bytes)
            .map_err(|e| diagnostic(report, e.node, e.message))?;
        self.leave(report, family)?;
        Ok(index)
    }
    fn future_drop(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        templates: &Checked<'_>,
        program: &mut proof::Builder,
        future: crate::types::TypeId,
    ) -> Result<(), Error> {
        program
            .require_current_store(&facts.types)
            .map_err(|e| diagnostic(report, None, e.into()))?;
        facts
            .checked_future_definition(future)
            .map_err(|e| diagnostic(report, None, e.into()))?;
        let key = BodyKey::FutureEntry(crate::types::future::EntryKey {
            future,
            operation: crate::types::future::Operation::Drop,
        });
        if self.active.get(&key) == Some(&0) {
            return Ok(());
        }
        let prepared = crate::pipeline::future_drop(facts, templates, future)
            .map_err(|e| diagnostic(report, None, e))?;
        if prepared.body.key() != key {
            return Err(diagnostic(
                report,
                None,
                "future Drop changed its complete scheduled family".into(),
            ));
        }
        self.enter(key, &prepared.body, &prepared.contract)
            .map_err(|e| diagnostic(report, Some(prepared.body.source.into()), e.into()))?;
        program
            .push_future_factory(facts, templates, prepared, &mut self.bytes)
            .map_err(|e| diagnostic(report, e.node, e.message))?;
        self.leave(report, key)
    }
    fn future_poll(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        templates: &Checked<'_>,
        program: &mut proof::Builder,
        method: crate::sema::coroutine_methods::FutureMethod,
    ) -> Result<usize, Error> {
        let prepared =
            crate::pipeline::future_poll(facts, templates, method, |facts, application| {
                program
                    .prove_future_application(facts, templates, application)
                    .map_err(|error| error.message)
            })
            .map_err(|message| diagnostic(report, None, message))?;
        let key = prepared.body.key();
        self.enter(key, &prepared.body, &prepared.contract)
            .map_err(|message| {
                diagnostic(report, Some(prepared.body.source.into()), message.into())
            })?;
        let index = program
            .push_future_factory(facts, templates, prepared, &mut self.bytes)
            .map_err(|error| diagnostic(report, error.node, error.message))?;
        self.leave(report, key)?;
        Ok(index)
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
            .push_factory(facts, templates, selected, body, &input, &mut self.bytes)
            .map_err(|e| diagnostic(report, e.node, e.message))?;
        Ok((index, family))
    }
    fn fn_pointer_factory(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        templates: &Checked<'_>,
        program: &mut proof::Builder,
        selected: crate::sema::compiler_items::pointers::Address,
    ) -> Result<(usize, BodyKey), Error> {
        let (body, input) = body::pointers::build(report, facts, selected)?;
        let family = body.key();
        self.enter(family, &body, &input)
            .map_err(|e| diagnostic(report, Some(body.source.into()), e.into()))?;
        let index = program
            .push_fn_pointer_factory(facts, templates, selected, body, &input, &mut self.bytes)
            .map_err(|e| diagnostic(report, e.node, e.message))?;
        Ok((index, family))
    }
    fn lending_factory(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        templates: &Checked<'_>,
        program: &mut proof::Builder,
        selected: crate::sema::lending_methods::Method,
    ) -> Result<(usize, BodyKey), Error> {
        let (body, input) = body::lending::build(report, facts, selected)?;
        let family = body.key();
        self.enter(family, &body, &input)
            .map_err(|e| diagnostic(report, Some(body.source.into()), e.into()))?;
        let index = program
            .push_lending_factory(facts, templates, selected, body, input, &mut self.bytes)
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
        if self.bytes > BODY_BYTES {
            return Err(BODIES_EXCEEDED);
        }
        *active += 1;
        Ok(())
    }
    /// Pay what one traversal action's demand scope collected: its work as
    /// canonical steps and its bytes against the fixed cap.
    fn pay_scope(
        &mut self,
        facts: &mut Declarations,
        [steps, bytes]: [usize; 2],
    ) -> Result<(), &'static str> {
        facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(steps);
        self.bytes = self.bytes.saturating_add(bytes);
        #[cfg(test)]
        record_demand(self.bytes, steps);
        if self.bytes > BODY_BYTES {
            return Err(BODIES_EXCEEDED);
        }
        Ok(())
    }
}
pub(super) fn prepare<C: Collect>(
    report: &Report,
    facts: &mut Declarations,
    roots: crate::lower::roots::Selected<'_>,
    mut originals: Originals<'_, '_, '_, C>,
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
    materialize(
        report,
        facts,
        roots,
        Proofs::Borrowed(templates),
        program,
        &ordinary,
        None,
    )
}

fn materialize(
    report: &Report,
    facts: &mut Declarations,
    roots: crate::lower::roots::Selected<'_>,
    mut templates: Proofs<'_, '_>,
    mut program: proof::Builder,
    ordinary: &Ordinary,
    mut preparation: Option<&mut dyn crate::sema::dispatch::Preparation>,
) -> Result<proof::Builder, Error> {
    let mut budget = Budget::default();
    #[cfg(test)]
    {
        budget.bytes = DEMAND.get()[0];
    }
    let (work, dependencies) =
        initial_work(report, facts, roots, &mut program, ordinary, &mut budget)?;
    // Every proof appended from here on is demanded.
    let first = program.proved();
    let mut traversal = Traversal {
        program,
        ordinary,
        work,
        dependencies,
        instances: HashMap::default(),
        visited: HashSet::default(),
        budget,
        cleanup: body::instantiate::Cleanup::new(facts),
    };
    while let Some(action) = traversal.work.pop() {
        if let Work::Enter(instance, _, _) = &action
            && let Some(service) = preparation.as_deref_mut()
        {
            templates.demand(report, facts, &mut traversal.program, *instance, service)?;
        }
        // Demanded work pays the demand budget, never lowering's account.
        let (stepped, charged) = facts.pay_demand_traversal(|facts| {
            templates.with_checked(report, |templates| {
                traversal.step(report, facts, templates, action)
            })
        });
        let paid = traversal.budget.pay_scope(facts, charged);
        stepped?;
        paid.map_err(|e| diagnostic(report, None, e.into()))?;
    }
    #[cfg(test)]
    record_demand(traversal.budget.bytes, 0);
    facts.carry_demand(first, traversal.budget.bytes);
    Ok(traversal.program)
}

#[cfg(test)]
thread_local! {
    /// Test-only: the bytes a demand budget starts from, so a control can
    /// reach the fixed cap without 64 MB of input; the bytes the last demand
    /// budget on this thread held; and the work its demand scopes paid.
    static DEMAND: std::cell::Cell<[usize; 3]> = const { std::cell::Cell::new([0, 0, 0]) };
    /// Test-only: work and bytes every specialized instance also charges.
    static FORGED: std::cell::Cell<[usize; 2]> = const { std::cell::Cell::new([0, 0]) };
}
/// Test-only: start every later demand budget on this thread at `bytes`.
#[cfg(test)]
pub(in crate::lower) fn preload_demand(bytes: usize) {
    DEMAND.set([bytes, 0, 0]);
}
/// Test-only: the demand budget's bytes and the work its scopes paid.
#[cfg(test)]
pub(in crate::lower) fn demand_costs() -> [usize; 2] {
    let [_, bytes, steps] = DEMAND.get();
    [bytes, steps]
}
/// Test-only: record the demand budget's bytes and work one scope paid.
#[cfg(test)]
pub(in crate::lower) fn record_demand(bytes: usize, steps: usize) {
    let [preload, _, paid] = DEMAND.get();
    DEMAND.set([preload, bytes, paid.saturating_add(steps)]);
}
/// Test-only: make every later specialized instance on this thread also
/// charge `work` and `bytes` through its caller-input payer.
#[cfg(test)]
pub(in crate::lower) fn forge_instance_charges(work: usize, bytes: usize) {
    FORGED.set([work, bytes]);
}

/// One ordinary DFS owns the same program, work stack and instance accounting.
struct Traversal<'a> {
    program: proof::Builder,
    ordinary: &'a Ordinary,
    work: Vec<Work>,
    dependencies: Dependencies,
    instances: HashMap<Instance, usize>,
    visited: HashSet<usize>,
    budget: Budget,
    cleanup: body::instantiate::Cleanup,
}
impl Traversal<'_> {
    fn adapter(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        templates: &Checked<'_>,
        entry: body::invocation_entries::Entry,
        source: body::ir::ProofSite,
        caller: Caller,
    ) -> Result<(), Error> {
        let error = |e: &str| diagnostic(report, Some(source), e.into());
        let original = self
            .ordinary
            .adapter_source(&facts.types, templates, &self.instances, entry)
            .map_err(|e| diagnostic(report, Some(source), e.into()))?;
        let Some(original) = original else {
            self.budget
                .push_work(facts, &mut self.work, Work::Adapter(entry, source, caller))
                .map_err(error)?;
            self.budget
                .push_work(
                    facts,
                    &mut self.work,
                    Work::Enter(entry.instance(), source, caller),
                )
                .map_err(error)?;
            return Ok(());
        };
        if let Some((index, family)) = self.budget.adapter(
            report,
            facts,
            templates,
            &mut self.program,
            original,
            (entry, source),
        )? {
            self.budget
                .push_work(facts, &mut self.work, Work::Leave(family))
                .map_err(error)?;
            self.dependencies.schedule(
                report,
                facts,
                &mut self.program,
                DependencyRoot::Body(index),
                &mut self.work,
                &mut self.budget,
            )?;
        }
        Ok(())
    }
    fn step(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        templates: &Checked<'_>,
        action: Work,
    ) -> Result<(), Error> {
        let (instance, source, caller) = match action {
            Work::FutureDrop(future, source) => {
                self.budget
                    .future_drop(report, facts, templates, &mut self.program, future)
                    .map_err(|e| diagnostic(report, Some(source), e.to_string()))?;
                return Ok(());
            }
            Work::Adapter(entry, source, caller) => {
                return self.adapter(report, facts, templates, entry, source, caller);
            }
            Work::Future(request, source, caller) => {
                let error = |e: &str| diagnostic(report, Some(source), e.into());
                let actual = caller
                    .future_caller(facts, &self.program, source, request)
                    .map_err(error)?;
                let signature = actual
                    .require_current(&facts.types, request.instance)
                    .map_err(error)?;
                if crate::sema::coroutine_methods::constructor_definition(facts, request.instance)
                    .map_err(error)?
                    .is_some()
                {
                    let Some(Kind::FunctionItem {
                        signature: public, ..
                    }) = facts.types.kind(request.instance.ty())
                    else {
                        return Err(error("future constructor lost its actual public item"));
                    };
                    if public != signature {
                        return Err(error(
                            "future constructor requires its actual opened input transport",
                        ));
                    }
                    if !self.instances.contains_key(&request.instance) {
                        let index = self.budget.future_constructor(
                            report,
                            facts,
                            templates,
                            &mut self.program,
                            request.instance,
                        )?;
                        self.instances.insert(request.instance, index);
                    }
                    return Ok(());
                }
                let method = facts
                    .primitive_methods
                    .as_deref()
                    .and_then(|registry| registry.get_future(request.instance))
                    .ok_or_else(|| {
                        error("future demand lost its actual selected poll operation")
                    })?;
                method.validate(facts).map_err(error)?;
                if !self.instances.contains_key(&request.instance) {
                    let index = self.budget.future_poll(
                        report,
                        facts,
                        templates,
                        &mut self.program,
                        method,
                    )?;
                    self.instances.insert(request.instance, index);
                }
                return Ok(());
            }
            Work::Enter(instance, source, caller) => (instance, source, caller),
            Work::Leave(item) => {
                self.budget.leave(report, item)?;
                return Ok(());
            }
        };
        self.selected_instance(report, facts, templates, (instance, source, caller))
    }
    fn selected_instance(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        templates: &Checked<'_>,
        request: (Instance, body::ir::ProofSite, Caller),
    ) -> Result<(), Error> {
        let (instance, source, _) = request;
        let error = |e: &str| diagnostic(report, Some(source), e.into());
        require_future_runtime(facts, instance).map_err(error)?;
        let fn_pointer =
            crate::sema::compiler_items::pointers::address(facts, instance).map_err(error)?;
        let index = if let Some(selected) = fn_pointer {
            if self.instances.contains_key(&instance) {
                return Ok(());
            }
            let (index, family) = self.budget.fn_pointer_factory(
                report,
                facts,
                templates,
                &mut self.program,
                selected,
            )?;
            self.instances.insert(instance, index);
            self.budget
                .push_work(facts, &mut self.work, Work::Leave(family))
                .map_err(error)?;
            index
        } else if let Some(selected) = facts
            .primitive_methods
            .as_deref()
            .and_then(|registry| registry.get(instance))
        {
            if self.instances.contains_key(&instance) {
                return Ok(());
            }
            let (index, family) =
                self.budget
                    .factory(report, facts, templates, &mut self.program, selected)?;
            self.instances.insert(instance, index);
            self.budget
                .push_work(facts, &mut self.work, Work::Leave(family))
                .map_err(error)?;
            index
        } else if let Some(selected) = facts
            .primitive_methods
            .as_deref()
            .and_then(|registry| registry.get_lending(instance))
        {
            if self.instances.contains_key(&instance) {
                return Ok(());
            }
            let (index, family) = self.budget.lending_factory(
                report,
                facts,
                templates,
                &mut self.program,
                selected,
            )?;
            self.instances.insert(instance, index);
            self.budget
                .push_work(facts, &mut self.work, Work::Leave(family))
                .map_err(error)?;
            index
        } else {
            let Some(index) = self.ordinary_instance(report, facts, templates, request)? else {
                return Ok(());
            };
            index
        };
        self.dependencies.schedule(
            report,
            facts,
            &mut self.program,
            DependencyRoot::Body(index),
            &mut self.work,
            &mut self.budget,
        )?;
        Ok(())
    }
    /// Ordinary source demand keeps its checked template and lexical caller;
    /// compiler factories use their separate validated body/permit path above.
    fn ordinary_instance(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        templates: &Checked<'_>,
        request: (Instance, body::ir::ProofSite, Caller),
    ) -> Result<Option<usize>, Error> {
        let (instance, source, caller) = request;
        let error = |message: &str| diagnostic(report, Some(source), message.into());
        let Some(selected) = self
            .ordinary
            .select(facts, templates, instance)
            .map_err(error)?
        else {
            return Ok(None);
        };
        let Some((template, input)) = selected.template else {
            return selected.concrete(&mut self.visited).map_err(error);
        };
        if self.instances.contains_key(&instance) {
            return Ok(None);
        }
        self.budget
            .enter(selected.family, &template.body, input)
            .map_err(error)?;
        let request = Specialization {
            template,
            input,
            instance,
            source,
            caller,
        };
        let index = specialize(
            report,
            facts,
            templates,
            (&mut self.program, &mut self.budget.bytes),
            &mut self.cleanup,
            request,
        )?;
        self.instances.insert(instance, index);
        self.budget
            .push_work(facts, &mut self.work, Work::Leave(selected.family))
            .map_err(error)?;
        Ok(Some(index))
    }
}

fn require_future_runtime(
    facts: &mut Declarations,
    instance: Instance,
) -> Result<(), &'static str> {
    let Some(selected) = facts
        .primitive_methods
        .as_deref()
        .and_then(|registry| registry.get_future(instance))
    else {
        return Ok(());
    };
    selected.validate(facts)?;
    if selected.key().operation != crate::types::future::Operation::Poll {
        return Err("future method demand has a different generated operation");
    }
    Err("future poll requires its checked generated runtime family and state ownership")
}

#[derive(Clone, Copy)]
pub(super) struct Specialization<'a> {
    pub(super) template: &'a body::templates::Template,
    pub(super) input: &'a Contract,
    pub(super) instance: Instance,
    pub(super) source: body::ir::ProofSite,
    pub(super) caller: Caller,
}
// Entered dependencies come only from a body sealed by Builder::push.
pub(super) fn specialize(
    report: &Report,
    facts: &mut Declarations,
    templates: &Checked<'_>,
    (program, demand): (&mut proof::Builder, &mut usize),
    cleanup: &mut body::instantiate::Cleanup,
    request: Specialization<'_>,
) -> Result<usize, Error> {
    let error = |e: &str| diagnostic(report, Some(request.source), e.into());
    #[cfg(test)]
    {
        let [work, bytes] = FORGED.get();
        if work != 0 || bytes != 0 {
            facts.charge_caller_inputs(work, bytes).map_err(error)?;
        }
    }
    body::assembly::prepay_instance(report, &request.template.body)?;
    let caller = request
        .caller
        .environment(facts, templates, program, request.source)
        .map_err(error)?;
    let mut materialized = body::instantiate::from_checked_caller(
        facts,
        request.template,
        request.instance,
        request.input,
        caller,
        cleanup,
    )
    .map_err(error)?;
    // This demand selects native execution only after authenticating the exact
    // checked original, instance and caller above. Its ordinary cleanup still
    // needs the full materialized region/loan proof. The original template and
    // separate CTFE materializations retain their actual const obligations.
    if matches!(materialized.body.owner, body::ir::Owner::Function { .. }) {
        materialized.body.const_context = false;
    }
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
        .push_instance(
            report,
            facts,
            templates,
            (materialized.body, &materialized.contract),
            Some(materialized.destruction),
            demand,
        )
        .map_err(|e| diagnostic(report, e.node, e.message))
}

fn initial_work(
    report: &Report,
    facts: &mut Declarations,
    roots: crate::lower::roots::Selected<'_>,
    program: &mut proof::Builder,
    ordinary: &Ordinary,
    budget: &mut Budget,
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
        budget
            .push_work(
                facts,
                &mut work,
                Work::Enter(instance, source.into(), Caller::ROOT),
            )
            .map_err(|e| diagnostic(report, Some(source.into()), e.into()))?;
    }
    let dependencies =
        Dependencies::roots(report, facts, program, roots.globals, &mut work, budget)?;
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
        budget: &mut Budget,
    ) -> Result<Self, Error> {
        let mut dependencies = Self::default();
        for &definition in globals {
            dependencies.schedule(
                report,
                facts,
                program,
                DependencyRoot::Global(definition),
                work,
                budget,
            )?;
        }
        Ok(dependencies)
    }
    fn inputs(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        program: &mut proof::Builder,
        root: DependencyRoot,
    ) -> Result<
        (
            body::ir::ProofSite,
            super::dependencies::Demands,
            proof::CleanupCalls,
            Caller,
        ),
        Error,
    > {
        Ok(match root {
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
                (
                    source.root_site(),
                    demands,
                    cleanup,
                    Caller::body(index)
                        .map_err(|e| diagnostic(report, Some(source.root_site()), e.into()))?,
                )
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
                (
                    source.into(),
                    demands,
                    proof::CleanupCalls::default(),
                    Caller::global(definition)
                        .map_err(|e| diagnostic(report, Some(source.into()), e.into()))?,
                )
            }
        })
    }
    fn schedule(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        program: &mut proof::Builder,
        root: DependencyRoot,
        work: &mut Vec<Work>,
        budget: &mut Budget,
    ) -> Result<(), Error> {
        let (source, demands, cleanup, caller) = self.inputs(report, facts, program, root)?;
        let error = |e: &str| diagnostic(report, Some(source), e.into());
        if let Some(futures) = demands.futures {
            for future in futures.requests.into_iter().rev() {
                budget
                    .push_work(facts, work, Work::Future(future, source, caller))
                    .map_err(error)?;
            }
        }
        for entry in demands.adapters.into_iter().rev() {
            budget
                .push_work(facts, work, Work::Adapter(entry, source, caller))
                .map_err(error)?;
        }
        for ty in demands.closures {
            let instance = Instance::closure(&facts.types, ty).map_err(error)?;
            budget
                .push_work(facts, work, Work::Enter(instance, source, caller))
                .map_err(error)?;
        }
        let objects = program
            .object_dependencies(facts, &demands.object_types)
            .map_err(error)?;
        for requests in [cleanup.futures, objects.futures].into_iter().flatten() {
            for future in requests.types.into_iter().rev() {
                budget
                    .push_work(facts, work, Work::FutureDrop(future, source))
                    .map_err(error)?;
            }
        }
        let mut next = demands.calls;
        next.extend(objects.ordinary);
        next.extend(cleanup.ordinary);
        for instance in next.into_iter().rev() {
            budget
                .push_work(facts, work, Work::Enter(instance, source, caller))
                .map_err(error)?;
        }
        Ok(())
    }
}

pub(super) fn prepare_ordinary<C: Collect>(
    report: &Report,
    facts: &mut Declarations,
    originals: &mut Originals<'_, '_, '_, C>,
    inputs: Vec<Contract>,
    concrete: HashMap<BodyKey, Body>,
    constants: Option<crate::pipeline::execution::Context>,
    globals: &[DefId],
) -> Result<(proof::Builder, Ordinary), Error> {
    let templates = originals.checked;
    crate::pipeline::checked_future_storage(facts, templates)
        .map_err(|e| diagnostic(report, None, e))?;
    let mut program = body::concrete_selected(
        report,
        facts,
        |owner| checked_callable(facts, templates, owner),
        concrete,
        originals.selection,
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
        resolve_ordinary(report, facts, body, input, &mut originals.preparation)?;
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
    let (mut program, mut bodies) = proof::Builder::new(report, facts, program)
        .map_err(|e| diagnostic(report, None, e.into()))?;
    program
        .bind_future_store(facts, templates)
        .map_err(|e| diagnostic(report, None, e.into()))?;
    program
        .prepare_original_regions(facts, &inputs, bodies.bodies())
        .map_err(|e| diagnostic(report, e.node, e.message))?;
    let mut ordinary = Ordinary::default();
    // Prove in program order while releasing the moved-from body records.
    bodies
        .reverse(facts)
        .map_err(|e| diagnostic(report, None, e.into()))?;
    while let Some((body, requirements)) = bodies
        .pop()
        .map_err(|e| diagnostic(report, None, e.into()))?
    {
        bodies
            .shrink(facts)
            .map_err(|e| diagnostic(report, None, e.into()))?;
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
        let index =
            program.push_original(report, facts, originals, (body, requirements), &input)?;
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

fn checked_callable(facts: &Declarations, templates: &Checked<'_>, owner: DefId) -> bool {
    if templates.get(ItemId(owner.0)).is_some() {
        return true;
    }
    facts
        .opaques
        .owners
        .get(&owner)
        .and_then(|identities| identities.last())
        .and_then(|&identity| templates.async_body(identity))
        .is_some_and(|(template, original, _)| {
            templates.future_store(&facts.types).is_ok()
                && crate::body::coroutines::original_template(facts, template, original)
                    == Ok(owner)
        })
}

fn resolve_ordinary(
    report: &Report,
    facts: &mut Declarations,
    body: &mut Body,
    input: &mut Contract,
    preparation: &mut Option<&mut dyn crate::sema::dispatch::Preparation>,
) -> Result<(), Error> {
    if let Some(service) = preparation {
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
    Ok(())
}

#[cfg(test)]
pub(super) fn caller_work_layouts() -> (usize, usize, usize) {
    #[expect(
        dead_code,
        reason = "the exact original queue sum is retained only for ABI comparison"
    )]
    enum Original {
        Adapter(body::invocation_entries::Entry, body::ir::ProofSite),
        Enter(Instance, body::ir::ProofSite),
        Leave(BodyKey),
    }
    (
        size_of::<Original>(),
        size_of::<Work>(),
        size_of::<Caller>(),
    )
}

#[cfg(test)]
pub(super) fn caller_work_payments(
    facts: &mut Declarations,
    key: BodyKey,
) -> Result<(), &'static str> {
    let mut budget = Budget::default();
    let mut work = Vec::new();
    let before = facts.body_work.canonical_steps;
    budget.push_work(facts, &mut work, Work::Leave(key))?;
    let cold = budget.bytes;
    assert!(cold >= work.capacity() * size_of::<Work>());
    assert!(cold >= size_of::<Work>());
    assert_eq!(facts.body_work.canonical_steps - before, 1);
    budget.push_work(facts, &mut work, Work::Leave(key))?;
    assert_eq!(budget.bytes, cold);
    assert_eq!(facts.body_work.canonical_steps - before, 2);
    let mut rejected = Vec::new();
    budget.bytes = 64_000_000;
    let attempted = facts.body_work.canonical_steps;
    assert!(
        budget
            .push_work(facts, &mut rejected, Work::Leave(key))
            .is_err()
    );
    assert!(rejected.is_empty());
    assert_eq!(rejected.capacity(), 0);
    assert!(budget.bytes > 64_000_000);
    assert_eq!(facts.body_work.canonical_steps - attempted, 1);
    Ok(())
}
