//! Each distinct CFG Call owns a complete request, even at a shared source site.
use super::{Body, Context, Declarations, ItemId, NodeKey, Sites, Source, g, ir, s};
use crate::{sema::model::Operation, types::Kind as Type};

pub(in super::super) fn emit(
    source: &mut Source,
    facts: &mut Declarations,
    body: &Body,
    context: Context,
    raw: &mut s::Original,
    sites: &mut Sites,
) -> Result<(), &'static str> {
    let owner = body
        .source
        .source()
        .ok_or("source call export has no actual semantic body")?;
    source
        .stage
        .reserve(body.values.len().saturating_mul(size_of::<bool>()))?;
    let mut seen = vec![false; body.values.len()];
    let mut ports = Ports {
        source,
        facts,
        body,
        context,
        raw,
        sites,
        owner,
    };
    for statement in &body.statements {
        ports.source.stage.charge(1)?;
        let ir::Action::Evaluate(value) = statement.action else {
            continue;
        };
        let actual = body
            .values
            .get(value.0 as usize)
            .ok_or("source evaluated value is unknown")?;
        let ir::Rvalue::Call {
            signature, callee, ..
        } = actual.kind
        else {
            continue;
        };
        if std::mem::replace(&mut seen[value.0 as usize], true) {
            return Err("source call value has duplicate evaluation ownership");
        }
        if matches!(
            ports.raw.values.get(value.0 as usize).map(|row| row.kind),
            Some(s::ValueKind::PointerCall { .. })
        ) {
            ports.pointer(statement, value, signature)?;
        } else {
            ports.one(statement, value, signature, callee)?;
        }
    }
    for (index, value) in body.values.iter().enumerate() {
        ports.source.stage.charge(1)?;
        if matches!(value.kind, ir::Rvalue::Call { .. }) && !seen[index] {
            return Err("source Call has no exact evaluated request coverage");
        }
    }
    Ok(())
}

struct CompletedCall {
    key: NodeKey,
    source_use: crate::sema::model::SourceCallUse,
    check_method: bool,
    // Safety was discharged against the semantic signature before dispatch.
    semantic_signature: crate::types::TypeId,
    protocol_callable: Option<crate::types::TypeId>,
}

struct Ports<'a> {
    source: &'a mut Source,
    facts: &'a mut Declarations,
    body: &'a Body,
    context: Context,
    raw: &'a mut s::Original,
    sites: &'a mut Sites,
    owner: NodeKey,
}
impl Ports<'_> {
    /// Version thirteen: a call through a fn pointer owns no request row. Its
    /// evaluation supplies its site, it runs only in a runtime original, and
    /// its value row carries the lexical unsafe context of its completed use.
    fn pointer(
        &mut self,
        statement: &ir::Statement,
        value: ir::ValueId,
        signature: crate::types::TypeId,
    ) -> Result<(), &'static str> {
        let (key, source_use) = self.completed(statement, signature)?;
        let effect = source_use.effect_for(self.facts, self.owner)?;
        if self.raw.context != s::CallContext::Runtime || effect != self.raw.context.request() {
            return Err("source pointer Call requires a runtime original context");
        }
        self.source.stage.charge(32)?;
        let safety = source_use.call_safety(self.facts, self.owner, key.node, signature)?;
        let Some(s::ValueKind::PointerCall { safety: slot, .. }) = self
            .raw
            .values
            .get_mut(value.0 as usize)
            .map(|row| &mut row.kind)
        else {
            return Err("source emitted Call value changed kind");
        };
        *slot = safety;
        Ok(())
    }
    /// The completed call operation at an evaluation's source site.
    fn completed(
        &mut self,
        statement: &ir::Statement,
        signature: crate::types::TypeId,
    ) -> Result<(NodeKey, crate::sema::model::SourceCallUse), &'static str> {
        let key = self
            .body
            .source
            .proof_site(statement.source)?
            .source()
            .ok_or("source Call has no actual source operation")?;
        self.source.stage.charge(2)?;
        let Some(Operation::Call {
            signature: retained,
            source_use: Some(source_use),
        }) = self.facts.node(key).and_then(|facts| facts.operation)
        else {
            return Err(
                "source Call has no exact completed owner/effect publication; ForPlan transport is not implemented",
            );
        };
        if signature != retained {
            return Err("source Call changed its completed typed signature");
        }
        self.source.stage.charge(1)?;
        Ok((key, source_use))
    }
    /// Resolve a direct or protocol-plan call operation at an evaluation's source site.
    fn completed_call(
        &mut self,
        statement: &ir::Statement,
        callable: crate::types::TypeId,
        signature: crate::types::TypeId,
    ) -> Result<CompletedCall, &'static str> {
        if !self.source.stage.declaration_policies() {
            let (key, source_use) = self.completed(statement, signature)?;
            return Ok(CompletedCall {
                key,
                source_use,
                check_method: true,
                semantic_signature: signature,
                protocol_callable: None,
            });
        }
        let key = self
            .body
            .source
            .proof_site(statement.source)?
            .source()
            .ok_or("source Call has no actual source operation")?;
        self.source.stage.charge(2)?;
        if let Some((source_use, original)) =
            self.conversion_or_index_step(key, callable, signature)?
        {
            return Ok(CompletedCall {
                key,
                source_use,
                check_method: false,
                semantic_signature: signature,
                protocol_callable: Some(original),
            });
        }
        let operation = self.facts.node(key).and_then(|facts| facts.operation);
        if let Some(Operation::Call {
            signature: retained,
            source_use: Some(source_use),
        }) = operation
        {
            if signature != retained
                && self.facts.methods.dispatched(self.owner, key, retained) != Some(signature)
            {
                return Err("source Call changed its completed typed signature");
            }
            self.source.stage.charge(1)?;
            return Ok(CompletedCall {
                key,
                source_use,
                check_method: true,
                semantic_signature: retained,
                protocol_callable: None,
            });
        }
        let (key, source_use, check_method, original) =
            self.protocol_plan_call(key, operation, callable, signature)?;
        Ok(CompletedCall {
            key,
            source_use,
            check_method,
            semantic_signature: signature,
            protocol_callable: Some(original),
        })
    }
    fn protocol_plan_call(
        &mut self,
        key: NodeKey,
        operation: Option<Operation>,
        callable: crate::types::TypeId,
        signature: crate::types::TypeId,
    ) -> Result<
        (
            NodeKey,
            crate::sema::model::SourceCallUse,
            bool,
            crate::types::TypeId,
        ),
        &'static str,
    > {
        use crate::sema::protocol_plans::Kind;
        match operation {
            Some(Operation::Field { plan: Some(id), .. }) => {
                let plan = id.current(self.facts, self.owner, key)?;
                let Kind::Field { steps, .. } = plan.kind else {
                    return Err("source field plan changed its protocol kind");
                };
                self.matches_access_steps(key, plan, steps, callable, signature)?
                    .map(|original| (key, plan.source_use, false, original))
                    .ok_or("source field protocol call changed its completed signature or target")
            }
            Some(Operation::MethodReceiver { plan: id }) => {
                let plan = id.current(self.facts, self.owner, key)?;
                let Kind::Receiver { steps, call, .. } = plan.kind else {
                    return Err("source method receiver plan changed its protocol kind");
                };
                if let Some(original) = self.matches_plan_call(key, call, callable, signature)? {
                    return Ok((key, plan.source_use, true, original));
                }
                self.matches_access_steps(key, plan, steps, callable, signature)?
                    .map(|original| (key, plan.source_use, false, original))
                    .ok_or("source method receiver protocol call changed its completed signature or target")
            }
            Some(Operation::Try { plan: id }) => {
                let plan = id.current(self.facts, self.owner, key)?;
                let calls = match plan.kind {
                    Kind::Block { from_output, .. } => [Some(from_output), None],
                    Kind::Branch {
                        branch, residual, ..
                    } => [Some(branch), Some(residual)],
                    _ => return Err("source Try plan changed its protocol kind"),
                };
                self.matches_calls(key, calls.into_iter().flatten(), callable, signature)?
                    .map(|original| (key, plan.source_use, false, original))
                    .ok_or("source Try protocol call changed its completed signature or target")
            }
            Some(Operation::Iteration { plan: id }) => {
                let plan = id.current(self.facts, self.owner, key)?;
                let Kind::Iteration { plan: for_plan, .. } = plan.kind else {
                    return Err("source iteration plan changed its protocol kind");
                };
                self.matches_calls(
                    key,
                    [for_plan.into_iter, for_plan.next],
                    callable,
                    signature,
                )?
                .map(|original| (key, plan.source_use, false, original))
                .ok_or("source iteration protocol call changed its completed signature or target")
            }
            Some(Operation::OffsetOf { plan: id }) => {
                let plan = id.current(self.facts, self.owner, key)?;
                self.matches_plan_call(key, plan.call, callable, signature)?
                    .map(|original| (key, plan.source_use, false, original))
                    .ok_or("source offset_of call changed its completed signature or target")
            }
            _ => Err(
                "source Call has no exact completed owner/effect publication; ForPlan transport is not implemented",
            ),
        }
    }
    fn conversion_or_index_step(
        &mut self,
        key: NodeKey,
        callable: crate::types::TypeId,
        signature: crate::types::TypeId,
    ) -> Result<Option<(crate::sema::model::SourceCallUse, crate::types::TypeId)>, &'static str>
    {
        let conversion = self
            .facts
            .reference_adjustments
            .get(&key)
            .and_then(|row| row.plan);
        let indexed = self.facts.methods.index_operand(key);
        for id in [conversion, indexed].into_iter().flatten() {
            let plan = id.current(self.facts, self.owner, key)?;
            let (crate::sema::protocol_plans::Kind::Coercion { steps, .. }
            | crate::sema::protocol_plans::Kind::Index { steps, .. }) = plan.kind
            else {
                return Err("source conversion or index plan changed its protocol kind");
            };
            if let Some(original) =
                self.matches_access_steps(key, plan, steps, callable, signature)?
            {
                return Ok(Some((plan.source_use, original)));
            }
        }
        Ok(None)
    }
    fn matches_access_steps(
        &mut self,
        key: NodeKey,
        plan: crate::sema::protocol_plans::Plan,
        steps: crate::sema::protocol_plans::access::Range,
        callable: crate::types::TypeId,
        signature: crate::types::TypeId,
    ) -> Result<Option<crate::types::TypeId>, &'static str> {
        let calls = crate::sema::protocol_plans::access::rows(self.facts, plan, self.owner, steps)?
            .iter()
            .map(|step| step.call)
            .collect::<Vec<_>>();
        self.matches_calls(key, calls, callable, signature)
    }
    fn matches_calls(
        &mut self,
        key: NodeKey,
        calls: impl IntoIterator<Item = crate::sema::iteration::Call>,
        callable: crate::types::TypeId,
        signature: crate::types::TypeId,
    ) -> Result<Option<crate::types::TypeId>, &'static str> {
        for planned in calls {
            if let Some(original) = self.matches_plan_call(key, planned, callable, signature)? {
                return Ok(Some(original));
            }
        }
        Ok(None)
    }
    fn matches_plan_call(
        &mut self,
        key: NodeKey,
        planned: crate::sema::iteration::Call,
        callable: crate::types::TypeId,
        signature: crate::types::TypeId,
    ) -> Result<Option<crate::types::TypeId>, &'static str> {
        self.source.stage.charge(1)?;
        if planned.signature != signature {
            return Ok(None);
        }
        if planned.callable == callable {
            return Ok(Some(planned.callable));
        }
        if let Some(selected) = self
            .facts
            .methods
            .dispatched(self.owner, key, planned.callable)
        {
            return Ok((selected == callable).then_some(planned.callable));
        }
        let selected = self.source.stage.paying(|pay| {
            crate::sema::dispatch::selected_callable_with_projections(
                self.facts,
                key,
                planned.callable,
                pay,
            )
        })?;
        Ok((selected == Some(callable)).then_some(planned.callable))
    }
    fn one(
        &mut self,
        statement: &ir::Statement,
        value: ir::ValueId,
        signature: crate::types::TypeId,
        callee: ir::ValueId,
    ) -> Result<(), &'static str> {
        let callable = self
            .body
            .values
            .get(callee.0 as usize)
            .ok_or("source Call has an unknown callee value")?;
        let Some(Type::FunctionItem { item, .. }) = self.facts.types.kind(callable.ty) else {
            return Err("source Call requires its actual direct function-item application");
        };
        let callable_type = callable.ty;
        let CompletedCall {
            key,
            source_use,
            check_method,
            semantic_signature,
            protocol_callable,
        } = self.completed_call(statement, callable_type, signature)?;
        let effect = source_use.effect_for(self.facts, self.owner)?;
        self.source.stage.charge(32)?;
        let safety =
            source_use.call_safety(self.facts, self.owner, key.node, semantic_signature)?;
        if effect != self.raw.context.request() {
            return Err("source Call effect differs from its actual original defining context");
        }
        if check_method {
            self.method_target(key, item, callable_type)?;
        }
        let callable_type = self.request_callable(
            key,
            callee,
            callable_type,
            protocol_callable,
            signature != semantic_signature,
        )?;
        let Some(Type::FunctionItem {
            item,
            arguments,
            signature: declared,
        }) = self.facts.types.kind(callable_type)
        else {
            return Err("source request lost its complete function item");
        };
        let target =
            self.source
                .stage
                .function_application(self.facts, item, arguments, self.context)?;
        let method = self
            .source
            .stage
            .method_request(self.facts, callable_type, self.context)?;
        let s::ValueKind::Call {
            signature: wire_signature,
            ..
        } = self
            .raw
            .values
            .get(value.0 as usize)
            .ok_or("source Call value row is absent")?
            .kind
        else {
            return Err("source emitted Call value changed kind");
        };
        let opening = self.opening(key, item, declared, signature)?;
        self.source.stage.reserve(4 * size_of::<s::CallRequest>())?;
        self.raw.calls.push(s::CallRequest {
            value: value.0,
            site: self.sites.body(&mut self.source.stage, statement.source)?,
            target,
            method,
            signature: wire_signature,
            opening,
            effect,
            safety,
        });
        Ok(())
    }
    /// Preserve the semantic operand while its checked signature is unchanged.
    /// A rewritten higher-order call keeps its exact selected application;
    /// each fresh consumer performs ordinary dispatch and its lifetime proof.
    fn request_callable(
        &mut self,
        key: NodeKey,
        callee: ir::ValueId,
        selected: crate::types::TypeId,
        protocol_callable: Option<crate::types::TypeId>,
        signature_changed: bool,
    ) -> Result<crate::types::TypeId, &'static str> {
        if !self.source.stage.declaration_policies() {
            return Ok(selected);
        }
        // Matching the completed method or protocol plan already authenticated
        // its full selected target. Each protocol call keeps its own operand.
        let Some(original) = protocol_callable.or_else(|| {
            self.facts
                .methods
                .calls
                .get(&key)
                .map(|method| method.callable)
        }) else {
            return Ok(selected);
        };
        if original == selected || signature_changed {
            return Ok(selected);
        }
        let Some(Type::FunctionItem {
            item, arguments, ..
        }) = self.facts.types.kind(original)
        else {
            return Err("source request original is not a function item");
        };
        let application =
            self.source
                .stage
                .function_application(self.facts, item, arguments, self.context)?;
        let ty = self.source.stage.ty(self.facts, original, self.context)?;
        self.sites.function_formation(
            &mut self.source.stage,
            self.body.source.root_body_site(),
            original,
            ty,
        )?;
        let row = self
            .raw
            .values
            .get_mut(callee.0 as usize)
            .ok_or("source request callee is absent")?;
        if !matches!(row.kind, s::ValueKind::Function(_)) {
            return Err("source request requires its direct function operand");
        }
        row.ty = ty;
        row.kind = s::ValueKind::Function(application);
        Ok(original)
    }
    fn method_target(
        &mut self,
        key: NodeKey,
        item: ItemId,
        callable: crate::types::TypeId,
    ) -> Result<(), &'static str> {
        let Some(method) = self.facts.methods.calls.get(&key).copied() else {
            return Ok(());
        };
        self.source.stage.charge(1)?;
        if !matches!(self.facts.types.kind(method.callable),Some(Type::FunctionItem{item,..}) if item.0==method.method.0)
        {
            return Err("source method Call changed its actual semantic declaration");
        }
        if method.method.0 == item.0 && method.callable == callable {
            return Ok(());
        }
        if !self.source.stage.conversion_roles() {
            return Err(
                "source method Call changed its actual selected declaration or application",
            );
        }
        let allow_projections = self.source.stage.declaration_policies();
        if allow_projections
            && let Some(selected) = self
                .facts
                .methods
                .dispatched(self.owner, key, method.callable)
        {
            return if selected == callable {
                Ok(())
            } else {
                Err("source method Call changed its completed original dispatch target")
            };
        }
        let selected = self.source.stage.paying(|pay| {
            if allow_projections {
                crate::sema::dispatch::selected_callable_with_projections(
                    self.facts,
                    key,
                    method.callable,
                    pay,
                )
            } else {
                crate::sema::dispatch::selected_callable(self.facts, key, method.callable, pay)
            }
        })?;
        if selected != Some(callable) {
            return Err(
                "source method Call changed its actual selected declaration or application",
            );
        }
        Ok(())
    }
    fn opening(
        &mut self,
        key: NodeKey,
        item: ItemId,
        declared: crate::types::TypeId,
        signature: crate::types::TypeId,
    ) -> Result<Option<s::Opening>, &'static str> {
        let Some(Type::Function { bound_regions, .. }) = self.facts.types.kind(declared) else {
            return Err("source callee signature is not a function");
        };
        // Version thirteen: a called tuple-variant constructor has no function
        // header and binds no late region; its variant row is its declaration.
        if self.source.stage.body_nodes()
            && self
                .facts
                .members
                .constructor(crate::resolve::model::DefId(item.0))
                .is_some()
        {
            self.source.stage.charge(1)?;
            return if bound_regions == 0 {
                Ok(None)
            } else {
                Err("source constructor call changed its actual late binder")
            };
        }
        let (binder, count) = self.source.stage.callable_binder(item)?;
        if count != bound_regions {
            return Err("source application changed its actual late binder");
        }
        self.source.stage.charge(1)?;
        let recorded_len = self
            .source
            .openings
            .get(&(self.owner, key))
            .map_or(0, Vec::len);
        if bound_regions == 0 && (self.source.stage.declaration_policies() || recorded_len == 0) {
            return Ok(None);
        }
        let regions: Vec<crate::types::RegionId> = if recorded_len == bound_regions as usize {
            self.source
                .openings
                .get(&(self.owner, key))
                .map_or_else(Vec::new, Clone::clone)
        } else if self.source.stage.declaration_policies() {
            self.shared_site_opening(key, declared, signature, bound_regions)?
        } else {
            return Err("source Call has incomplete or foreign late-opening allocations");
        };
        let start = u32::try_from(self.raw.openings.len())
            .map_err(|_| "source late-opening table overflow")?;
        self.source
            .stage
            .reserve(regions.len().saturating_mul(4 * size_of::<s::RegionRef>()))?;
        for region in regions {
            self.source.stage.charge(1)?;
            self.raw
                .openings
                .push(self.source.stage.region(region, self.context)?);
        }
        let end = u32::try_from(self.raw.openings.len())
            .map_err(|_| "source late-opening table overflow")?;
        Ok((bound_regions != 0).then_some(s::Opening {
            binder,
            regions: g::Range { start, end },
        }))
    }
    fn shared_site_opening(
        &mut self,
        key: NodeKey,
        declared: crate::types::TypeId,
        signature: crate::types::TypeId,
        bound_regions: u32,
    ) -> Result<Vec<crate::types::RegionId>, &'static str> {
        let extracted = self
            .source
            .stage
            .paying(|pay| {
                crate::types::callable::late_result::opening_regions(
                    &self.facts.types,
                    declared,
                    signature,
                    pay,
                )
            })?
            .ok_or("source Call has incomplete or foreign late-opening allocations")?;
        if extracted.len() != bound_regions as usize {
            return Err("source Call has incomplete or foreign late-opening allocations");
        }
        let recorded = self
            .source
            .openings
            .get(&(self.owner, key))
            .map_or(&[][..], Vec::as_slice);
        let mut fallback = recorded
            .iter()
            .copied()
            .filter(|region| !extracted.contains(&Some(*region)));
        let mut resolved = Vec::with_capacity(extracted.len());
        for &slot in &extracted {
            let region = match slot {
                Some(region) => region,
                None => fallback
                    .next()
                    .ok_or("source Call has incomplete or foreign late-opening allocations")?,
            };
            resolved.push(region);
        }
        Ok(resolved)
    }
}

#[cfg(test)]
#[path = "../../../../tests/invariants/override_openings.rs"]
mod tests;
