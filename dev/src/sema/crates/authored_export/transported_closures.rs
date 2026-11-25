//! Closure originals reserve child body rows after roots in canonical BFS order.
use super::{Context, DeclarationRef, Declarations, DefId, Report, Stage, g, s};
use crate::{
    resolve::{model::ScopeId, unit::NodeKey},
    sema::{closures::Body as ClosureBody, nested_outputs::Link},
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
    pending_outputs: Vec<PendingOutput>,
    finalized: bool,
}

struct PendingOutput {
    closure: s::ClosureRef,
    body: BodyRef,
    context: Context,
    link: Arc<Link>,
}

#[derive(Clone, Copy)]
enum ParentLink {
    Root(BodyRef),
    Closure(usize),
}

struct KeptClosure {
    body: ClosureBody,
    owner: DefId,
    owner_body: BodyRef,
    ordinal: usize,
}

struct ReservedClosure {
    body: ClosureBody,
    body_ref: BodyRef,
    owner_decl: DeclarationRef,
    frame: Context,
}

impl Index {
    pub(super) fn require_finalized(&self) -> Result<(), &'static str> {
        if !self.pending_outputs.is_empty() && !self.finalized {
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
        if !self.declaration_policies() || facts.closures.all().is_empty() {
            return Ok(());
        }
        let (kept, parents) = self.collect_kept_closures(report, facts)?;
        if kept.is_empty() {
            return Ok(());
        }
        let levels = bfs_levels(&kept, &parents)?;
        let reserved = self.reserve_closure_levels(&kept, &parents, &levels)?;
        self.emit_closure_headers(facts, reserved)
    }

    fn collect_kept_closures(
        &mut self,
        report: &Report,
        facts: &Declarations,
    ) -> Result<(Vec<KeptClosure>, Vec<ParentLink>), &'static str> {
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
            let Some(&owner_body) = self.functions.bodies.get(&owner) else {
                continue;
            };
            if self
                .rows
                .compiler_operations
                .iter()
                .any(|op| op.body == owner_body)
            {
                continue;
            }
            let index = kept.len();
            by_source.insert(body.semantic_source(), index);
            let owner_scope = report.resolution.definitions[owner.0 as usize].scope;
            let module = report.resolution.scopes[owner_scope.0 as usize].module;
            if let Some(&scope) = report
                .resolution
                .node_scopes
                .get(&(module, body.source.node))
            {
                by_scope.insert(scope, index);
            }
            kept.push(KeptClosure {
                body,
                owner,
                owner_body,
                ordinal,
            });
        }
        let mut parents = Vec::with_capacity(kept.len());
        for (index, item) in kept.iter().enumerate() {
            self.charge(1)?;
            let parent = resolve_parent(
                report,
                facts,
                &kept,
                &root_bodies,
                &by_source,
                &by_scope,
                index,
                item,
            )?;
            parents.push(parent);
        }
        Ok((kept, parents))
    }

    fn reserve_closure_levels(
        &mut self,
        kept: &[KeptClosure],
        parents: &[ParentLink],
        levels: &[Vec<usize>],
    ) -> Result<Vec<ReservedClosure>, &'static str> {
        let mut assigned = vec![None; kept.len()];
        let mut next_ordinal: HashMap<BodyRef, u32> = HashMap::default();
        let mut reserved = Vec::with_capacity(kept.len());
        for level in levels {
            let mut batch: Vec<(BodyRef, usize, usize)> = Vec::with_capacity(level.len());
            for &index in level {
                let parent_body = match parents[index] {
                    ParentLink::Root(body) => body,
                    ParentLink::Closure(parent_index) => assigned[parent_index]
                        .ok_or("source closure parent was not reserved before child")?,
                };
                batch.push((parent_body, kept[index].ordinal, index));
            }
            batch.sort_unstable_by_key(|&(parent, ordinal, _)| (parent.0, ordinal));
            for (parent_body, _, index) in batch {
                let item = &kept[index];
                let owner_decl = self.declaration(item.owner)?;
                let ordinal_slot = next_ordinal.entry(parent_body).or_insert(0);
                let ordinal = *ordinal_slot;
                *ordinal_slot = ordinal
                    .checked_add(1)
                    .ok_or("source closure sibling ordinal overflow")?;
                let body_ref = BodyRef(
                    u32::try_from(self.rows.body_owners.len())
                        .map_err(|_| "source closure body owner table overflow")?,
                );
                let contract_universals = item
                    .body
                    .execution
                    .map_or(item.body.universals, |exec| exec.universals);
                let frame = self.frame(
                    owner_decl,
                    None,
                    s::BinderKind::ClosureOriginal { body: body_ref },
                    contract_universals,
                )?;
                let body_ctx = Context {
                    body: Some(body_ref),
                    ..frame
                };
                self.reserve(
                    4 * (size_of::<BodyOwner>()
                        + size_of::<Context>()
                        + size_of::<Option<(s::Original, s::BodyInputs)>>())
                        + retained::new_map::<ClosureId, (s::ClosureRef, BodyRef, Context)>(1),
                )?;
                self.rows.body_owners.push(BodyOwner {
                    declaration: owner_decl,
                    parent: Some(parent_body),
                    kind: BodyKind::Closure,
                    ordinal,
                });
                self.functions.contexts.push(body_ctx);
                self.functions.originals.push(None);
                let closure_ref = s::ClosureRef(
                    u32::try_from(reserved.len())
                        .map_err(|_| "source closure header table overflow")?,
                );
                if self
                    .closures
                    .map
                    .insert(item.body.identity, (closure_ref, body_ref, body_ctx))
                    .is_some()
                {
                    return Err("source closure identity reserved twice");
                }
                assigned[index] = Some(body_ref);
                reserved.push(ReservedClosure {
                    body: item.body,
                    body_ref,
                    owner_decl,
                    frame,
                });
            }
        }
        Ok(reserved)
    }

    fn emit_closure_headers(
        &mut self,
        facts: &mut Declarations,
        reserved: Vec<ReservedClosure>,
    ) -> Result<(), &'static str> {
        self.reserve(
            size_of::<s::ClosureColumns>().saturating_add(
                reserved
                    .len()
                    .saturating_mul(4 * size_of::<s::ClosureHeader>()),
            ),
        )?;
        for item in reserved {
            self.charge(1)?;
            let c = item.body;
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
            let binder = self.binder(item.frame)?;
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
        Ok(())
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "parent resolution consults the report, declarations and indexed closure tables"
)]
fn resolve_parent(
    report: &Report,
    facts: &Declarations,
    kept: &[KeptClosure],
    root_bodies: &HashMap<NodeKey, BodyRef>,
    by_source: &HashMap<NodeKey, usize>,
    by_scope: &HashMap<ScopeId, usize>,
    index: usize,
    item: &KeptClosure,
) -> Result<ParentLink, &'static str> {
    if let Some(link) = facts
        .nested_outputs
        .as_deref()
        .and_then(|registry| registry.get(&item.body.semantic_source()))
    {
        let parent_key = link
            .parent
            .source()
            .ok_or("source closure output link lost its source parent")?;
        if let Some(&parent_index) = by_source.get(&parent_key) {
            return Ok(ParentLink::Closure(parent_index));
        }
        if let Some(&root) = root_bodies.get(&parent_key) {
            return Ok(ParentLink::Root(root));
        }
        return Err("source closure parent body is outside the exported body catalog");
    }
    let owner_scope = report.resolution.definitions[item.owner.0 as usize].scope;
    let module = report.resolution.scopes[owner_scope.0 as usize].module;
    let mut cursor = report
        .resolution
        .node_scopes
        .get(&(module, item.body.source.node))
        .and_then(|&scope| report.resolution.scopes[scope.0 as usize].parent);
    while let Some(scope) = cursor {
        if let Some(&parent_index) = by_scope.get(&scope)
            && parent_index != index
            && kept[parent_index].owner == item.owner
        {
            return Ok(ParentLink::Closure(parent_index));
        }
        if scope == owner_scope {
            break;
        }
        cursor = report.resolution.scopes[scope.0 as usize].parent;
    }
    Ok(ParentLink::Root(item.owner_body))
}

fn bfs_levels(
    kept: &[KeptClosure],
    parents: &[ParentLink],
) -> Result<Vec<Vec<usize>>, &'static str> {
    let mut depths = vec![0_usize; kept.len()];
    let mut max_depth = 0_usize;
    for start in 0..kept.len() {
        let mut depth = 1_usize;
        let mut current = parents[start];
        while let ParentLink::Closure(parent) = current {
            depth = depth
                .checked_add(1)
                .filter(|&d| d <= kept.len())
                .ok_or("source closure nesting contains a cycle")?;
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
