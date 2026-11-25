//! Closure and inline-constant originals reserve child body rows after roots in canonical BFS order.
use super::{Context, DeclarationRef, Declarations, DefId, Report, Stage, g, s};
use crate::{
    resolve::{body_origin::BodyOrigin, model::ScopeId, unit::NodeKey},
    sema::{closures::Body as ClosureBody, inline_constants::Evaluation, nested_outputs::Link},
    support::{
        call_traits::CallTrait,
        crates::owner_catalog::{BodyKind, BodyOwner, BodyRef},
        hash::HashMap,
        retained,
    },
    types::closure::{CallKind, ClosureId},
};
use std::sync::Arc;

#[derive(Default)]
pub(super) struct Index {
    map: HashMap<ClosureId, (s::ClosureRef, BodyRef, Context)>,
    anonymous: HashMap<NodeKey, (u32, BodyRef, Context)>,
    pending_outputs: Vec<PendingOutput>,
    pending_anonymous_outputs: Vec<PendingAnonymousOutput>,
    finalized: bool,
}

struct PendingOutput {
    closure: s::ClosureRef,
    body: BodyRef,
    context: Context,
    link: Arc<Link>,
}

struct PendingAnonymousOutput {
    ordinal: u32,
    body: BodyRef,
    link: Arc<Link>,
}

#[derive(Clone, Copy)]
enum ParentLink {
    Root(BodyRef),
    Nested(usize),
}

enum KeptKind {
    Closure(ClosureBody),
    Anonymous {
        parent_body: NodeKey,
        universals: u32,
        deferred: bool,
    },
}

struct KeptNested {
    kind: KeptKind,
    source: NodeKey,
    owner: DefId,
    owner_body: BodyRef,
    sort_key: (u8, usize),
}

enum ReservedKind {
    Closure(ClosureBody),
    Anonymous { universals: u32, deferred: bool },
}

impl KeptNested {
    fn contract(&self, facts: &Declarations, body_ref: BodyRef) -> (BodyKind, s::BinderKind, u32) {
        let output_universals = facts
            .nested_outputs
            .as_deref()
            .and_then(|registry| registry.get(&self.source))
            .map_or(0, |link| link.universals);
        match self.kind {
            KeptKind::Closure(c) => (
                BodyKind::Closure,
                s::BinderKind::ClosureOriginal { body: body_ref },
                c.execution
                    .map_or(c.universals, |exec| exec.universals)
                    .max(output_universals),
            ),
            KeptKind::Anonymous { universals, .. } => (
                BodyKind::Anonymous,
                s::BinderKind::AnonymousOriginal { body: body_ref },
                universals.max(output_universals),
            ),
        }
    }
}

struct ReservedNested {
    kind: ReservedKind,
    body_ref: BodyRef,
    owner_decl: DeclarationRef,
    frame: Context,
}

impl Index {
    pub(super) fn require_finalized(&self) -> Result<(), &'static str> {
        if (!self.pending_outputs.is_empty() || !self.pending_anonymous_outputs.is_empty())
            && !self.finalized
        {
            return Err("source closure output links were not finalized before sealing");
        }
        Ok(())
    }
}

type CollectedFormation<'a> = (u32, Vec<s::ClosureFormationInput>, &'a [u16]);

fn append_closure_formation(
    table: &mut s::ClosureColumns,
    inherited: u32,
    inputs: Vec<s::ClosureFormationInput>,
    slots: &[u16],
) -> Result<s::ClosureFormation, &'static str> {
    let input_start = u32::try_from(table.formation_inputs.len())
        .map_err(|_| "source closure formation input table overflow")?;
    table.formation_inputs.extend(inputs);
    let input_end = u32::try_from(table.formation_inputs.len())
        .map_err(|_| "source closure formation input table overflow")?;
    let slot_start = u32::try_from(table.callable_slots.len())
        .map_err(|_| "source closure callable slot table overflow")?;
    table.callable_slots.extend_from_slice(slots);
    let slot_end = u32::try_from(table.callable_slots.len())
        .map_err(|_| "source closure callable slot table overflow")?;
    Ok(s::ClosureFormation {
        inherited,
        inputs: g::Range {
            start: input_start,
            end: input_end,
        },
        callable_slots: g::Range {
            start: slot_start,
            end: slot_end,
        },
    })
}

impl Stage {
    pub(crate) fn has_closure(&self, id: ClosureId) -> bool {
        self.closures.map.contains_key(&id)
    }

    pub(crate) fn has_anonymous(&self, source: NodeKey) -> bool {
        self.closures.anonymous.contains_key(&source)
    }

    pub(crate) fn anonymous_body(&mut self, source: NodeKey) -> Result<BodyRef, &'static str> {
        self.charge(1)?;
        self.closures
            .anonymous
            .get(&source)
            .map(|&(_, body, _)| body)
            .ok_or("source inline constant names an unreserved anonymous body")
    }

    pub(crate) fn anonymous_context(&mut self, source: NodeKey) -> Result<Context, &'static str> {
        self.charge(1)?;
        let &(_, body, context) = self
            .closures
            .anonymous
            .get(&source)
            .ok_or("source anonymous original has no reserved body")?;
        if !matches!(self.functions.originals.get(body.0 as usize), Some(None)) {
            return Err("source anonymous original is already collected or has no reserved row");
        }
        Ok(context)
    }

    pub(crate) fn record_anonymous_contract(
        &mut self,
        source: NodeKey,
        output: Option<&Arc<Link>>,
    ) -> Result<(), &'static str> {
        self.charge(1)?;
        let &(ordinal, body, _) = self
            .closures
            .anonymous
            .get(&source)
            .ok_or("source anonymous contract names an unreserved body")?;
        if let Some(link) = output {
            if link.formation.is_some() || link.creator_only_from.is_some() {
                return Err("source anonymous output cannot carry closure formation data");
            }
            self.reserve(4 * size_of::<PendingAnonymousOutput>())?;
            self.closures
                .pending_anonymous_outputs
                .push(PendingAnonymousOutput {
                    ordinal,
                    body,
                    link: Arc::clone(link),
                });
        }
        Ok(())
    }

    pub(crate) fn closure_ref(&mut self, id: ClosureId) -> Result<s::ClosureRef, &'static str> {
        self.charge(1)?;
        self.closures
            .map
            .get(&id)
            .map(|&(reference, _, _)| reference)
            .ok_or("source closure type names an unreserved closure identity")
    }

    pub(crate) fn closure_context(&mut self, id: ClosureId) -> Result<Context, &'static str> {
        self.charge(1)?;
        let &(_, body, context) = self
            .closures
            .map
            .get(&id)
            .ok_or("source closure original has no reserved body")?;
        if !matches!(self.functions.originals.get(body.0 as usize), Some(None)) {
            return Err("source closure original is already collected or has no reserved row");
        }
        Ok(context)
    }

    pub(crate) fn record_closure_contract(
        &mut self,
        id: ClosureId,
        require_result_formation: bool,
        output: Option<&Arc<Link>>,
    ) -> Result<(), &'static str> {
        self.charge(1)?;
        let &(closure, body, context) = self
            .closures
            .map
            .get(&id)
            .ok_or("source closure contract names an unreserved closure")?;
        let header = self
            .cold()?
            .closures
            .as_deref_mut()
            .and_then(|table| table.headers.get_mut(closure.0 as usize))
            .ok_or("source closure contract lost its reserved header")?;
        header.require_result_formation = require_result_formation;
        if let Some(link) = output {
            self.reserve(4 * size_of::<PendingOutput>())?;
            self.closures.pending_outputs.push(PendingOutput {
                closure,
                body,
                context,
                link: Arc::clone(link),
            });
        }
        Ok(())
    }

    pub(crate) fn finalize_closure_outputs(
        &mut self,
        facts: &Declarations,
    ) -> Result<(), &'static str> {
        self.closures.finalized = true;
        let pending = std::mem::take(&mut self.closures.pending_outputs);
        for item in &pending {
            self.finalize_one_closure_output(facts, item)?;
        }
        let anonymous = std::mem::take(&mut self.closures.pending_anonymous_outputs);
        for item in &anonymous {
            self.finalize_one_anonymous_output(item)?;
        }
        Ok(())
    }

    fn finalize_one_anonymous_output(
        &mut self,
        item: &PendingAnonymousOutput,
    ) -> Result<(), &'static str> {
        self.charge(1)?;
        let parent_body = self
            .rows
            .body_owners
            .get(item.body.0 as usize)
            .and_then(|owner| owner.parent)
            .ok_or("source anonymous output link lost its reserved parent body")?;
        let parent_ctx = *self
            .functions
            .contexts
            .get(parent_body.0 as usize)
            .ok_or("source anonymous output link lost its parent body context")?;
        let link = &item.link;
        let mut bridges_vec = Vec::with_capacity(link.bridges.len());
        for &bridge in &link.bridges {
            self.charge(1)?;
            bridges_vec.push(s::ClosureBridge {
                slot: bridge.slot,
                parent: self.region(bridge.parent, parent_ctx)?,
                output: bridge.output,
            });
        }
        self.reserve(
            bridges_vec
                .len()
                .saturating_mul(4 * size_of::<s::ClosureBridge>()),
        )?;
        let table = self
            .cold()?
            .closures
            .as_deref_mut()
            .ok_or("source anonymous output lost its closure columns")?;
        let bridge_start = u32::try_from(table.anonymous_bridges.len())
            .map_err(|_| "source anonymous bridge table overflow")?;
        table.anonymous_bridges.extend(bridges_vec);
        let bridge_end = u32::try_from(table.anonymous_bridges.len())
            .map_err(|_| "source anonymous bridge table overflow")?;
        let header = table
            .anonymous
            .get_mut(item.ordinal as usize)
            .ok_or("source anonymous output lost its header row")?;
        header.output = Some(s::ClosureOutput {
            revision: link.revision,
            universals: link.universals,
            bridges: g::Range {
                start: bridge_start,
                end: bridge_end,
            },
            formation: None,
            creator_only_from: None,
        });
        Ok(())
    }

    fn finalize_one_closure_output(
        &mut self,
        facts: &Declarations,
        item: &PendingOutput,
    ) -> Result<(), &'static str> {
        self.charge(1)?;
        let parent_body = self
            .rows
            .body_owners
            .get(item.body.0 as usize)
            .and_then(|owner| owner.parent)
            .ok_or("source closure output link lost its reserved parent body")?;
        let parent_ctx = *self
            .functions
            .contexts
            .get(parent_body.0 as usize)
            .ok_or("source closure output link lost its parent body context")?;
        let link = &item.link;
        let mut bridges_vec = Vec::with_capacity(link.bridges.len());
        for &bridge in &link.bridges {
            self.charge(1)?;
            bridges_vec.push(s::ClosureBridge {
                slot: bridge.slot,
                parent: self.region(bridge.parent, parent_ctx)?,
                output: bridge.output,
            });
        }
        let formation = self.collect_closure_formation(facts, item, parent_ctx)?;
        let extra = bridges_vec
            .len()
            .saturating_mul(4 * size_of::<s::ClosureBridge>())
            .saturating_add(formation.as_ref().map_or(0, |(_, inputs, slots)| {
                inputs
                    .len()
                    .saturating_mul(4 * size_of::<s::ClosureFormationInput>())
                    .saturating_add(slots.len().saturating_mul(4 * size_of::<u16>()))
            }));
        self.reserve(extra)?;
        let table = self
            .cold()?
            .closures
            .as_deref_mut()
            .ok_or("source closure output lost its closure columns")?;
        let bridge_start = u32::try_from(table.bridges.len())
            .map_err(|_| "source closure bridge table overflow")?;
        table.bridges.extend(bridges_vec);
        let bridge_end = u32::try_from(table.bridges.len())
            .map_err(|_| "source closure bridge table overflow")?;
        let formation = formation
            .map(|(inherited, inputs, slots)| {
                append_closure_formation(table, inherited, inputs, slots)
            })
            .transpose()?;
        let header = table
            .headers
            .get_mut(item.closure.0 as usize)
            .ok_or("source closure output lost its header row")?;
        header.output = Some(s::ClosureOutput {
            revision: link.revision,
            universals: link.universals,
            bridges: g::Range {
                start: bridge_start,
                end: bridge_end,
            },
            formation,
            creator_only_from: link.creator_only_from,
        });
        Ok(())
    }

    fn collect_closure_formation<'a>(
        &mut self,
        facts: &Declarations,
        item: &'a PendingOutput,
        parent_ctx: Context,
    ) -> Result<Option<CollectedFormation<'a>>, &'static str> {
        let Some(formation) = &item.link.formation else {
            return Ok(None);
        };
        if self.closure_ref(formation.identity)? != item.closure {
            return Err("source closure formation link changed its closure identity");
        }
        let mut inputs = Vec::with_capacity(formation.inputs.len());
        for &input in &formation.inputs {
            self.charge(1)?;
            let child = self.ty(facts, input.child, item.context)?;
            let parent = self.ty(facts, input.parent, parent_ctx)?;
            inputs.push(s::ClosureFormationInput {
                root: input.root.raw(),
                child,
                parent,
            });
        }
        Ok(Some((
            formation.inherited,
            inputs,
            formation.callable_slots.as_ref(),
        )))
    }

    pub(super) fn reserve_closures(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
    ) -> Result<(), &'static str> {
        if !self.declaration_policies()
            || (facts.closures.all().is_empty()
                && facts
                    .inline_constants
                    .as_deref()
                    .is_none_or(|r| r.len() == 0))
        {
            return Ok(());
        }
        let (kept, parents) = self.collect_kept_nested(report, facts)?;
        if kept.is_empty() {
            return Ok(());
        }
        let levels = bfs_levels(&kept, &parents)?;
        let reserved = self.reserve_nested_levels(facts, &kept, &parents, &levels)?;
        self.emit_nested_headers(facts, reserved)
    }

    fn root_body_map(
        &mut self,
        facts: &Declarations,
    ) -> Result<HashMap<NodeKey, BodyRef>, &'static str> {
        let mut root_bodies: HashMap<NodeKey, BodyRef> = HashMap::default();
        for f in &facts.functions {
            self.charge(1)?;
            if let Some(&body_ref) = self.functions.bodies.get(&f.definition)
                && let Some(body_key) = f.body
                && !self
                    .rows
                    .compiler_operations
                    .iter()
                    .any(|op| op.body == body_ref)
            {
                root_bodies.insert(body_key, body_ref);
            }
        }
        for i in &facts.initializers {
            self.charge(1)?;
            if let Some(&body_ref) = self.functions.bodies.get(&i.definition)
                && self.rows.body_owners[body_ref.0 as usize].kind == BodyKind::Constant
            {
                root_bodies.insert(i.semantic_source(), body_ref);
            }
        }
        Ok(root_bodies)
    }

    fn exported_owner_body(&self, owner: DefId) -> Option<BodyRef> {
        let &owner_body = self.functions.bodies.get(&owner)?;
        if !matches!(
            self.rows.body_owners[owner_body.0 as usize].kind,
            BodyKind::Function | BodyKind::Constant
        ) || self
            .rows
            .compiler_operations
            .iter()
            .any(|op| op.body == owner_body)
        {
            return None;
        }
        Some(owner_body)
    }

    fn collect_kept_nested(
        &mut self,
        report: &Report,
        facts: &Declarations,
    ) -> Result<(Vec<KeptNested>, Vec<ParentLink>), &'static str> {
        let root_bodies = self.root_body_map(facts)?;
        let mut kept = Vec::new();
        let mut by_source: HashMap<NodeKey, usize> = HashMap::default();
        let mut by_scope: HashMap<ScopeId, usize> = HashMap::default();
        for (ordinal, &body) in facts.closures.all().iter().enumerate() {
            self.charge(1)?;
            if facts
                .crates
                .as_deref()
                .and_then(|linkage| linkage.promoted_closure(body.identity))
                .is_some()
            {
                continue;
            }
            let Some(owner) = body.owner else {
                continue;
            };
            let Some(owner_body) = self.exported_owner_body(owner) else {
                continue;
            };
            let index = kept.len();
            let source = body.semantic_source();
            by_source.insert(source, index);
            record_nested_scope(report, owner, body.source.node, index, &mut by_scope);
            kept.push(KeptNested {
                kind: KeptKind::Closure(body),
                source,
                owner,
                owner_body,
                sort_key: (0, ordinal),
            });
        }
        if let Some(inline) = facts.inline_constants.as_deref() {
            for (ordinal, row) in inline.bodies().enumerate() {
                self.charge(1)?;
                let BodyOrigin::Source(source) = row.origin else {
                    continue;
                };
                if row.evaluation == Evaluation::SuppressedZeroRepeat || row.result.is_none() {
                    continue;
                }
                let Some(owner) = row.declaration else {
                    continue;
                };
                let Some(owner_body) = self.exported_owner_body(owner) else {
                    continue;
                };
                let index = kept.len();
                by_source.insert(source, index);
                record_nested_scope(report, owner, source.node, index, &mut by_scope);
                kept.push(KeptNested {
                    kind: KeptKind::Anonymous {
                        parent_body: row.parent_body,
                        universals: row.universals,
                        deferred: row.evaluation == Evaluation::DeferredTemplate,
                    },
                    source,
                    owner,
                    owner_body,
                    sort_key: (1, ordinal),
                });
            }
        }
        let mut raw_parents = Vec::with_capacity(kept.len());
        for (index, item) in kept.iter().enumerate() {
            self.charge(1)?;
            raw_parents.push(resolve_parent(
                report,
                facts,
                &kept,
                &root_bodies,
                &by_source,
                &by_scope,
                index,
                item,
            )?);
        }
        prune_unrooted_nested(kept, raw_parents)
    }

    fn reserve_nested_levels(
        &mut self,
        facts: &Declarations,
        kept: &[KeptNested],
        parents: &[ParentLink],
        levels: &[Vec<usize>],
    ) -> Result<Vec<ReservedNested>, &'static str> {
        let mut assigned = vec![None; kept.len()];
        let mut next_ordinal: HashMap<BodyRef, u32> = HashMap::default();
        let mut reserved = Vec::with_capacity(kept.len());
        let mut closure_count = 0_u32;
        let mut anonymous_count = 0_u32;
        for level in levels {
            let mut batch: Vec<(BodyRef, (u8, usize), usize)> = Vec::with_capacity(level.len());
            for &index in level {
                let parent_body = match parents[index] {
                    ParentLink::Root(body) => body,
                    ParentLink::Nested(parent_index) => assigned[parent_index]
                        .ok_or("source nested parent was not reserved before child")?,
                };
                batch.push((parent_body, kept[index].sort_key, index));
            }
            batch.sort_unstable_by_key(|&(parent, sort_key, _)| (parent.0, sort_key));
            for (parent_body, _, index) in batch {
                let item = &kept[index];
                let owner_decl = self.declaration(item.owner)?;
                let ordinal_slot = next_ordinal.entry(parent_body).or_insert(0);
                let ordinal = *ordinal_slot;
                *ordinal_slot = ordinal
                    .checked_add(1)
                    .ok_or("source nested sibling ordinal overflow")?;
                let body_ref = BodyRef(
                    u32::try_from(self.rows.body_owners.len())
                        .map_err(|_| "source nested body owner table overflow")?,
                );
                let (body_kind, binder_kind, contract_universals) = item.contract(facts, body_ref);
                let frame = self.frame(owner_decl, None, binder_kind, contract_universals)?;
                let body_ctx = Context {
                    body: Some(body_ref),
                    ..frame
                };
                self.reserve(
                    4 * (size_of::<BodyOwner>()
                        + size_of::<Context>()
                        + size_of::<Option<(s::Original, s::BodyInputs)>>())
                        + retained::new_map::<ClosureId, (s::ClosureRef, BodyRef, Context)>(1)
                        + retained::new_map::<NodeKey, (u32, BodyRef, Context)>(1),
                )?;
                self.rows.body_owners.push(BodyOwner {
                    declaration: owner_decl,
                    parent: Some(parent_body),
                    kind: body_kind,
                    ordinal,
                });
                self.functions.contexts.push(body_ctx);
                self.functions.originals.push(None);
                let kind = match item.kind {
                    KeptKind::Closure(c) => {
                        let closure_ref = s::ClosureRef(closure_count);
                        closure_count = closure_count
                            .checked_add(1)
                            .ok_or("source closure header table overflow")?;
                        if self
                            .closures
                            .map
                            .insert(c.identity, (closure_ref, body_ref, body_ctx))
                            .is_some()
                        {
                            return Err("source closure identity reserved twice");
                        }
                        ReservedKind::Closure(c)
                    }
                    KeptKind::Anonymous {
                        universals,
                        deferred,
                        ..
                    } => {
                        let anon_ordinal = anonymous_count;
                        anonymous_count = anonymous_count
                            .checked_add(1)
                            .ok_or("source anonymous header table overflow")?;
                        if self
                            .closures
                            .anonymous
                            .insert(item.source, (anon_ordinal, body_ref, body_ctx))
                            .is_some()
                        {
                            return Err("source anonymous identity reserved twice");
                        }
                        ReservedKind::Anonymous {
                            universals,
                            deferred,
                        }
                    }
                };
                assigned[index] = Some(body_ref);
                reserved.push(ReservedNested {
                    kind,
                    body_ref,
                    owner_decl,
                    frame,
                });
            }
        }
        Ok(reserved)
    }

    fn emit_nested_headers(
        &mut self,
        facts: &mut Declarations,
        reserved: Vec<ReservedNested>,
    ) -> Result<(), &'static str> {
        self.reserve(size_of::<s::ClosureColumns>().saturating_add(
            reserved.len().saturating_mul(
                4 * size_of::<s::ClosureHeader>().max(size_of::<s::AnonymousHeader>()),
            ),
        ))?;
        for item in reserved {
            self.charge(1)?;
            let binder = self.binder(item.frame)?;
            match item.kind {
                ReservedKind::Closure(c) => {
                    let closed = Context {
                        owner: item.owner_decl,
                        frame: None,
                        body: None,
                    };
                    self.prepare_type_arena(facts, c.signature)?;
                    let signature = self.ty(facts, c.signature, closed)?;
                    let public_binders = crate::sema::closures::actual::call_binders(facts, c)?;
                    let call_kind = call_trait(
                        facts
                            .types
                            .closure_kind(c.identity)
                            .ok_or("source closure lost its completed call permission")?,
                    );
                    let execution = match c.execution {
                        Some(exec) => {
                            self.prepare_type_arena(facts, exec.self_ty)?;
                            let self_ty = self.ty(facts, exec.self_ty, item.frame)?;
                            self.prepare_type_arena(facts, exec.signature)?;
                            let exec_sig = self.ty(facts, exec.signature, closed)?;
                            Some(s::ClosureExecution {
                                self_ty,
                                signature: exec_sig,
                                universals: exec.universals,
                                receiver: call_trait(exec.receiver),
                            })
                        }
                        None => None,
                    };
                    self.cold()?
                        .closures
                        .get_or_insert_with(|| Box::new(s::ClosureColumns::default()))
                        .headers
                        .push(s::ClosureHeader {
                            body: item.body_ref,
                            owner: item.owner_decl,
                            binder,
                            signature,
                            universals: c.universals,
                            inherited: c.inherited,
                            public_binders,
                            call_kind,
                            constant: c.constant.is_some(),
                            execution,
                            output: None,
                            require_result_formation: false,
                        });
                }
                ReservedKind::Anonymous {
                    universals,
                    deferred,
                } => {
                    self.cold()?
                        .closures
                        .get_or_insert_with(|| Box::new(s::ClosureColumns::default()))
                        .anonymous
                        .push(s::AnonymousHeader {
                            body: item.body_ref,
                            owner: item.owner_decl,
                            binder,
                            universals,
                            deferred,
                            output: None,
                        });
                }
            }
        }
        Ok(())
    }
}

fn record_nested_scope(
    report: &Report,
    owner: DefId,
    node: crate::syntax::ast::NodeId,
    index: usize,
    by_scope: &mut HashMap<ScopeId, usize>,
) {
    let owner_scope = report.resolution.definitions[owner.0 as usize].scope;
    let module = report.resolution.scopes[owner_scope.0 as usize].module;
    if let Some(&scope) = report.resolution.node_scopes.get(&(module, node))
        && scope != owner_scope
    {
        by_scope.insert(scope, index);
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "parent resolution consults the report, declarations and indexed nested tables"
)]
fn resolve_parent(
    report: &Report,
    facts: &Declarations,
    kept: &[KeptNested],
    root_bodies: &HashMap<NodeKey, BodyRef>,
    by_source: &HashMap<NodeKey, usize>,
    by_scope: &HashMap<ScopeId, usize>,
    index: usize,
    item: &KeptNested,
) -> Result<Option<ParentLink>, &'static str> {
    if let Some(link) = facts
        .nested_outputs
        .as_deref()
        .and_then(|registry| registry.get(&item.source))
    {
        let parent_key = link
            .parent
            .source()
            .ok_or("source nested output link lost its source parent")?;
        if let Some(&parent_index) = by_source.get(&parent_key) {
            return Ok(Some(ParentLink::Nested(parent_index)));
        }
        if let Some(&root) = root_bodies.get(&parent_key) {
            return Ok(Some(ParentLink::Root(root)));
        }
        return match item.kind {
            KeptKind::Anonymous { .. } => Ok(None),
            KeptKind::Closure(_) => {
                Err("source closure parent body is outside the exported body catalog")
            }
        };
    }
    if let KeptKind::Anonymous { parent_body, .. } = item.kind {
        if let Some(&parent_index) = by_source.get(&parent_body) {
            return Ok(Some(ParentLink::Nested(parent_index)));
        }
        if let Some(&root) = root_bodies.get(&parent_body) {
            return Ok(Some(ParentLink::Root(root)));
        }
        return Ok(None);
    }
    let KeptKind::Closure(closure) = item.kind else {
        return Ok(None);
    };
    let owner_scope = report.resolution.definitions[item.owner.0 as usize].scope;
    let module = report.resolution.scopes[owner_scope.0 as usize].module;
    let mut cursor = report
        .resolution
        .node_scopes
        .get(&(module, closure.source.node))
        .and_then(|&scope| report.resolution.scopes[scope.0 as usize].parent);
    while let Some(scope) = cursor {
        if let Some(&parent_index) = by_scope.get(&scope)
            && parent_index != index
            && kept[parent_index].owner == item.owner
        {
            return Ok(Some(ParentLink::Nested(parent_index)));
        }
        if scope == owner_scope {
            break;
        }
        cursor = report.resolution.scopes[scope.0 as usize].parent;
    }
    Ok(Some(ParentLink::Root(item.owner_body)))
}

fn prune_unrooted_nested(
    kept: Vec<KeptNested>,
    raw_parents: Vec<Option<ParentLink>>,
) -> Result<(Vec<KeptNested>, Vec<ParentLink>), &'static str> {
    let mut alive = vec![false; kept.len()];
    for start in 0..kept.len() {
        let mut steps = 0_usize;
        let mut current = raw_parents[start];
        let rooted = loop {
            match current {
                Some(ParentLink::Root(_)) => break true,
                Some(ParentLink::Nested(parent)) => {
                    steps = steps
                        .checked_add(1)
                        .filter(|&d| d <= kept.len())
                        .ok_or("source nested body hierarchy contains a cycle")?;
                    current = raw_parents[parent];
                }
                None => break false,
            }
        };
        alive[start] = rooted;
    }
    let mut remap = vec![None; kept.len()];
    let mut filtered = Vec::with_capacity(kept.len());
    for (old, item) in kept.into_iter().enumerate() {
        if alive[old] {
            remap[old] = Some(filtered.len());
            filtered.push(item);
        }
    }
    let mut parents = Vec::with_capacity(filtered.len());
    for (old, link) in raw_parents.into_iter().enumerate() {
        if !alive[old] {
            continue;
        }
        let parent = match link {
            Some(ParentLink::Root(body)) => ParentLink::Root(body),
            Some(ParentLink::Nested(parent)) => ParentLink::Nested(
                remap[parent].ok_or("source nested parent was pruned unexpectedly")?,
            ),
            None => return Err("source nested pruning lost its rooted parent"),
        };
        parents.push(parent);
    }
    Ok((filtered, parents))
}

fn bfs_levels(
    kept: &[KeptNested],
    parents: &[ParentLink],
) -> Result<Vec<Vec<usize>>, &'static str> {
    let mut depths = vec![0_usize; kept.len()];
    let mut max_depth = 0_usize;
    for start in 0..kept.len() {
        let mut depth = 1_usize;
        let mut current = parents[start];
        while let ParentLink::Nested(parent) = current {
            depth = depth
                .checked_add(1)
                .filter(|&d| d <= kept.len())
                .ok_or("source nested body hierarchy contains a cycle")?;
            current = parents[parent];
        }
        depths[start] = depth;
        max_depth = max_depth.max(depth);
    }
    let mut levels = vec![Vec::new(); max_depth];
    for (index, depth) in depths.into_iter().enumerate() {
        levels[depth - 1].push(index);
    }
    Ok(levels)
}

const fn call_trait(kind: CallKind) -> CallTrait {
    match kind {
        CallKind::Shared => CallTrait::Fn,
        CallKind::Mutable => CallTrait::FnMut,
        CallKind::Once => CallTrait::FnOnce,
    }
}
