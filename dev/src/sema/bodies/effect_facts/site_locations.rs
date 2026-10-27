//! Prepare immutable site data before the current call-use/node transaction commits.
use super::{CallContexts, Checker, Error, MAX_RETAINED, MAX_ROWS, MAX_TRANSIENT, SourceCallUse};
use crate::{
    resolve::unit::NodeKey,
    sema::{
        model::{Declarations, Operation},
        values::{Evaluator, LocationId, Reader},
    },
    support::{bindings::SourceIntrinsic, retained},
    syntax::ast::{Kind as Syntax, NodeId},
    types::{Kind, TypeId, infer::Solved},
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct Site {
    context: SourceCallUse,
    node: NodeId,
    callable: TypeId,
    location: LocationId,
}
impl Site {
    fn key(self) -> (u32, u32, u32) {
        (self.context.0.get(), self.node.get(), self.callable.0)
    }
}
impl CallContexts {
    fn site_range(&self, context: SourceCallUse) -> std::ops::Range<usize> {
        self.sites
            .partition_point(|site| site.context.0 < context.0)
            ..self
                .sites
                .partition_point(|site| site.context.0 <= context.0)
    }
    pub(super) fn reserve_sites(&mut self, count: usize) -> Result<(), &'static str> {
        if self.sites.len().saturating_add(count) > MAX_ROWS {
            return Err("caller-site inventory exhausted");
        }
        let growth =
            retained::Growth::vector::<Site>(self.sites.len(), self.sites.capacity(), count);
        let current = self.bytes(self.rows.capacity())?;
        let projected = if growth.bytes == 0 {
            current
        } else {
            current
                .saturating_sub(retained::vector(&self.sites))
                .saturating_add(growth.bytes)
        };
        if projected > MAX_RETAINED || current.saturating_add(growth.bytes) > MAX_TRANSIENT {
            return Err("caller-site transient storage budget exceeded");
        }
        self.sites
            .try_reserve(count)
            .map_err(|_| "caller-site allocation failed")?;
        if self.bytes(self.rows.capacity())? > MAX_RETAINED {
            return Err("caller-site retained storage budget exceeded");
        }
        Ok(())
    }
}
impl SourceCallUse {
    pub(crate) fn caller_site(
        self,
        facts: &Declarations,
        owner: NodeKey,
        node: NodeId,
        callable: TypeId,
    ) -> Result<Option<LocationId>, &'static str> {
        self.effect_for(facts, owner)?;
        let contexts = facts
            .source_bindings
            .as_deref()
            .and_then(|catalog| catalog.call_contexts.as_deref())
            .ok_or("caller site has no current source context")?;
        let key = (self.0.get(), node.get(), callable.0);
        Ok(contexts
            .sites
            .binary_search_by_key(&key, |site| site.key())
            .ok()
            .map(|index| contexts.sites[index].location))
    }
}
#[derive(Clone, Copy)]
struct Request {
    node: NodeId,
    callable: TypeId,
}
impl Checker<'_> {
    fn retain_site_request(
        &mut self,
        requests: &mut Vec<Request>,
        node: NodeId,
        callable: TypeId,
    ) -> Result<(), Error> {
        let Some(Kind::FunctionItem { item, .. }) = self.facts.types.kind(callable) else {
            return Ok(());
        };
        let definition = crate::resolve::model::DefId(item.0);
        let tracked = self
            .facts
            .checked_caller_tracking(self.report, definition)?;
        let intrinsic = self
            .facts
            .source_bindings
            .as_deref()
            .ok_or_else(|| self.error(node, "caller site lost its source catalog"))?
            .intrinsic_kind(definition)
            .map_err(|error| self.error(node, error))?;
        if !tracked && intrinsic != Some(SourceIntrinsic::CallerLocation) {
            return Ok(());
        }
        let growth = retained::Growth::vector::<Request>(requests.len(), requests.capacity(), 1);
        self.facts
            .charge_caller_inputs(growth.work.saturating_add(1), growth.bytes)
            .map_err(|error| self.error(node, error))?;
        requests
            .try_reserve(1)
            .map_err(|_| self.error(node, "caller-site staging allocation failed"))?;
        requests.push(Request { node, callable });
        Ok(())
    }
    fn ordinary_site_requests(
        &mut self,
        solved: &Solved,
        requests: &mut Vec<Request>,
    ) -> Result<(), Error> {
        for index in 0..self.state.calls.len() {
            let (node, _) = self.state.calls[index];
            if self.is_try_call(node) {
                if let Some(Operation::Try { plan }) = self
                    .facts
                    .node(self.key(node))
                    .and_then(|facts| facts.operation)
                {
                    let plan = plan
                        .current(self.facts, self.state.body, self.key(node))
                        .map_err(|e| self.error(node, e))?;
                    for call in plan.calls().into_iter().flatten() {
                        self.retain_site_request(requests, node, call.callable)?;
                    }
                }
                continue;
            }
            if matches!(self.kind(node), Syntax::For { .. }) || self.erased_source_unary(node)? {
                continue;
            }
            let callable = if let Some(call) = self.facts.methods.calls.get(&self.key(node)) {
                call.callable
            } else if let Syntax::Call { callee, .. } = self.kind(node) {
                let term = self.term(callee)?;
                *solved
                    .types
                    .get(term.0 as usize)
                    .ok_or_else(|| self.error(node, "caller site lost solved callee"))?
            } else {
                continue;
            };
            self.retain_site_request(requests, node, callable)?;
        }
        Ok(())
    }
    pub(super) fn prepare_caller_sites(
        &mut self,
        solved: &Solved,
        context: Option<SourceCallUse>,
        warm: bool,
    ) -> Result<Vec<Site>, Error> {
        let mut requests = Vec::new();
        self.ordinary_site_requests(solved, &mut requests)?;
        for index in 0..self.state.call_types.try_plans.len() {
            let open = self.state.call_types.try_plans[index];
            let calls = match open.kind {
                crate::sema::bodies::try_control::OpenKind::Block { call, .. } => {
                    [Some(call), None]
                }
                crate::sema::bodies::try_control::OpenKind::Branch {
                    branch, residual, ..
                } => [Some(branch), Some(residual)],
            };
            for call in calls.into_iter().flatten() {
                let call = call.solved(solved).map_err(|e| self.error(open.node, e))?;
                self.retain_site_request(&mut requests, open.node, call.callable)?;
            }
        }
        if requests.is_empty() {
            return Ok(Vec::new());
        }
        let context = context.ok_or_else(|| {
            self.error(
                self.state.body.node,
                "caller sites lack a current body context",
            )
        })?;
        let count = requests.len();
        self.facts
            .charge_caller_inputs(
                count.saturating_mul(count.ilog2() as usize + 2),
                count.saturating_mul(4 * size_of::<Site>()),
            )
            .map_err(|e| self.error(self.state.body.node, e))?;
        requests.sort_unstable_by_key(|request| (request.node.get(), request.callable.0));
        requests.dedup_by_key(|request| (request.node, request.callable));
        let mut sites = Vec::new();
        sites.try_reserve_exact(requests.len()).map_err(|_| {
            self.error(self.state.body.node, "caller-site result allocation failed")
        })?;
        for request in requests {
            let node = self.key(request.node);
            let previous = if warm {
                context
                    .caller_site(self.facts, self.state.body, request.node, request.callable)
                    .map_err(|e| self.error(request.node, e))?
            } else {
                None
            };
            let location = if let Some(location) = previous {
                Evaluator::new(self.report, self.facts)
                    .validate_source_location(node, location)
                    .map_err(|e| self.error(request.node, e.message()))?;
                location
            } else {
                if warm {
                    return Err(self.error(
                        request.node,
                        "warm caller publication changed its required site data",
                    ));
                }
                Evaluator::new(self.report, self.facts)
                    .prepare_source_location(node)
                    .map_err(|e| self.error(request.node, e.message()))?
            };
            Reader::new(self.facts)
                .caller_location_value(location)
                .map_err(|e| self.error(request.node, e))?;
            sites.push(Site {
                context,
                node: request.node,
                callable: request.callable,
                location,
            });
        }
        Ok(sites)
    }
    pub(super) fn preflight_site_commit(
        &mut self,
        context: Option<SourceCallUse>,
        sites: &[Site],
        warm: bool,
    ) -> Result<(), Error> {
        let Some(context) = context else {
            return Ok(());
        };
        let contexts = self
            .facts
            .source_bindings
            .as_deref()
            .and_then(|catalog| catalog.call_contexts.as_deref())
            .ok_or_else(|| {
                self.error(
                    self.state.body.node,
                    "caller sites lost their current owner",
                )
            })?;
        if warm {
            if contexts.sites[contexts.site_range(context)] != *sites {
                return Err(self.error(self.state.body.node, "warm caller site inventory changed"));
            }
        } else if contexts
            .sites
            .last()
            .is_some_and(|site| site.context.0 >= context.0)
        {
            return Err(self.error(
                self.state.body.node,
                "caller sites changed append-only body order",
            ));
        }
        if sites.is_empty() {
            return Ok(());
        }
        let growth = retained::Growth::vector::<Site>(
            contexts.sites.len(),
            contexts.sites.capacity(),
            sites.len(),
        );
        self.facts
            .charge_caller_inputs(growth.work, growth.bytes)
            .map_err(|e| self.error(self.state.body.node, e))?;
        if !warm {
            self.facts
                .source_bindings
                .as_deref_mut()
                .and_then(|catalog| catalog.call_contexts.as_deref_mut())
                .ok_or_else(|| {
                    Declarations::error(
                        self.report,
                        self.state.body,
                        "caller site owner disappeared",
                    )
                })?
                .reserve_sites(sites.len())
                .map_err(|e| Declarations::error(self.report, self.state.body, e))?;
        }
        Ok(())
    }
    pub(super) fn commit_caller_sites(&mut self, sites: &[Site], warm: bool) {
        if !warm
            && !sites.is_empty()
            && let Some(contexts) = self
                .facts
                .source_bindings
                .as_deref_mut()
                .and_then(|catalog| catalog.call_contexts.as_deref_mut())
        {
            contexts.sites.extend_from_slice(sites);
        }
    }
}
