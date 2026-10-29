//! Required members retain absence; every actual provided body reserves one original.
mod formals;
use super::{
    DeclarationRef, Declarations, DefId, OwnerKind, Report, Stage, canonical_nodes::Context, g, s,
};
use crate::{
    support::crates::owner_catalog::{BodyKind, BodyOwner, BodyRef},
    support::{constness::Constness, hash::HashMap, retained},
    types::Kind,
};

#[derive(Default)]
pub(super) struct Index {
    pub(super) bodies: HashMap<DefId, BodyRef>,
    pub(super) contexts: Vec<Context>,
    pub(super) originals: Vec<Option<(s::Original, s::BodyInputs)>>,
    headers: HashMap<crate::types::ItemId, u32>,
    pub(super) exact: bool,
}

impl Stage {
    pub(crate) fn callable_binder(
        &mut self,
        item: crate::types::ItemId,
    ) -> Result<(g::BinderRef, u32), &'static str> {
        self.charge(3)?;
        let index = self
            .functions
            .headers
            .get(&item)
            .copied()
            .ok_or("source call target has no staged actual function header")?;
        let binder = self
            .rows
            .functions
            .get(index as usize)
            .ok_or("source callable header index is stale")?
            .callable
            .binder;
        let count = self
            .rows
            .binders
            .get(binder.0 as usize)
            .ok_or("source callable frame is unknown")?
            .count;
        Ok((binder, count))
    }
    pub(super) fn functions(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
    ) -> Result<(), &'static str> {
        let mut count = 0usize;
        for index in 0..self.definitions.len() {
            self.charge(1)?;
            let definition =
                DefId(u32::try_from(index).map_err(|_| "source function identity overflow")?);
            let Some(&index) = facts.function_indices.get(&definition) else {
                continue;
            };
            let function = *facts
                .functions
                .get(index)
                .ok_or("source function header index is stale")?;
            if function.definition != definition {
                return Err("source function header index names another declaration");
            }
            if facts.target_feature_functions.contains_key(&definition)
                || !self.exact_linkage()
                    && facts
                        .foreign
                        .as_deref()
                        .and_then(|registry| registry.function(definition))
                        .is_some_and(|function| {
                            function.naming != crate::support::linkage::Naming::Rust
                                || function.linkage != crate::support::linkage::Linkage::Internal
                        })
            {
                return Err(
                    "source named/foreign/target-feature function metadata requires its wider profile",
                );
            }
            if facts
                .source_bindings
                .as_deref()
                .map(|c| c.intrinsic_kind(definition))
                .transpose()?
                .flatten()
                .is_some_and(|kind| {
                    matches!(
                        kind,
                        crate::support::bindings::SourceIntrinsic::ArithOffset
                            | crate::support::bindings::SourceIntrinsic::UncheckedSub
                            | crate::support::bindings::SourceIntrinsic::SubWithOverflow
                            | crate::support::bindings::SourceIntrinsic::ColdPath
                    )
                })
            {
                return Err(
                    "source intrinsic declaration semantics require a wider metadata profile",
                );
            }
            let declaration = self.declaration(definition)?;
            self.prepare_signature_arena(facts, function.signature)?;
            self.function(report, facts, &function, declaration)?;
            self.exact_callable(facts, &function, declaration)?;
            count = count
                .checked_add(1)
                .ok_or("source function count overflow")?;
        }
        if count != facts.functions.len() {
            return Err("source function index omitted an actual declaration header");
        }
        Ok(())
    }

    fn function(
        &mut self,
        report: &Report,
        facts: &Declarations,
        function: &crate::sema::model::Function,
        declaration: DeclarationRef,
    ) -> Result<(), &'static str> {
        let scheme = facts
            .lifetimes
            .functions
            .get(&function.definition)
            .ok_or("source function has no completed lifetime scheme")?;
        if scheme.captures != 0 || !scheme.early.is_empty() {
            return Err(
                "source function early/captured callable frame transport is not implemented",
            );
        }
        let parent = Context {
            owner: declaration,
            frame: None,
            body: None,
        };
        let (original, context) = self.signature(facts, function.signature, parent, false)?;
        if self.rows.binders[original.binder.0 as usize].count != scheme.total {
            return Err("source original signature differs from its actual lifetime scheme");
        }
        let Some(Kind::FunctionItem {
            item, signature, ..
        }) = facts
            .definitions
            .get(function.definition.0 as usize)
            .copied()
            .flatten()
            .and_then(|ty| facts.types.kind(ty))
        else {
            return Err("source function has no actual public callable declaration");
        };
        if item.0 != function.definition.0 {
            return Err("source public callable belongs to another declaration");
        }
        let (callable, _) = self.signature(facts, signature, parent, true)?;
        let regions = self.function_regions(scheme, context)?;
        let inputs = self.function_inputs(facts, function.definition, context)?;
        let (formals, receiver) = self.formals(report, facts, function, context)?;
        let body = self.function_body(function, declaration, context)?;
        // Lexical containment is already in Declaration.definition.parent.
        // This is the distinct actual associated owner, not the root module.
        let parent = facts
            .associated
            .owner(function.definition)
            .and_then(|owner| facts.associated.get(owner))
            .map(|owner| {
                let owner = owner.source_owner()?;
                match owner.kind {
                    OwnerKind::Trait { definition, .. } => Ok(definition),
                    OwnerKind::Impl { .. } => Ok(owner.self_definition),
                    OwnerKind::ImportedImpl(_) => {
                        Err("source function cannot acquire imported syntax ownership")
                    }
                }
            })
            .transpose()?
            .map(|owner| self.declaration(owner))
            .transpose()?;
        let header = s::FunctionHeader {
            declaration,
            scope: declaration,
            parent,
            callable,
            original,
            regions,
            inputs,
            declared_inputs: u32::try_from(
                facts
                    .generics
                    .declared_predicates(function.definition)
                    .len(),
            )
            .map_err(|_| "source declared clause count overflow")?,
            formals,
            receiver,
            constness: if facts.is_const_function(function.definition) {
                Constness::Const
            } else {
                Constness::Ordinary
            },
            body,
        };
        self.reserve(4 * size_of::<s::FunctionHeader>())?;
        self.reserve(retained::new_map::<crate::types::ItemId, u32>(1))?;
        let row = u32::try_from(self.rows.functions.len())
            .map_err(|_| "source function header table overflow")?;
        if self
            .functions
            .headers
            .insert(crate::types::ItemId(function.definition.0), row)
            .is_some()
        {
            return Err("source function header has duplicate actual identity");
        }
        self.rows.functions.push(header);
        Ok(())
    }

    fn function_inputs(
        &mut self,
        facts: &Declarations,
        definition: DefId,
        context: Context,
    ) -> Result<g::Range, &'static str> {
        let start =
            u32::try_from(self.rows.clauses.len()).map_err(|_| "source input table overflow")?;
        for input in crate::sema::declaration_inputs::own(
            &facts.generics,
            facts.crates.as_deref(),
            definition,
        )? {
            self.clause(facts, input, context, None)?;
        }
        Ok(g::Range {
            start,
            end: u32::try_from(self.rows.clauses.len())
                .map_err(|_| "source input table overflow")?,
        })
    }

    fn function_body(
        &mut self,
        function: &crate::sema::model::Function,
        declaration: DeclarationRef,
        context: Context,
    ) -> Result<s::FunctionBody, &'static str> {
        if function.body.is_some() {
            self.reserve(
                4 * (size_of::<BodyOwner>()
                    + size_of::<Context>()
                    + size_of::<Option<(s::Original, s::BodyInputs)>>())
                    + retained::new_map::<DefId, BodyRef>(1),
            )?;
            let id = BodyRef(
                u32::try_from(self.rows.body_owners.len())
                    .map_err(|_| "source original owner table overflow")?,
            );
            self.rows.body_owners.push(BodyOwner {
                declaration,
                parent: None,
                kind: BodyKind::Function,
                ordinal: 0,
            });
            if self
                .functions
                .bodies
                .insert(function.definition, id)
                .is_some()
            {
                return Err("source function has duplicate original bodies");
            }
            self.functions.contexts.push(Context {
                body: Some(id),
                ..context
            });
            self.functions.originals.push(None);
            Ok(s::FunctionBody::Original(id))
        } else {
            Ok(s::FunctionBody::Absent)
        }
    }

    fn function_regions(
        &mut self,
        scheme: &crate::sema::lifetimes::FunctionRegions,
        context: Context,
    ) -> Result<s::RegionScheme, &'static str> {
        let start = u32::try_from(self.rows.region_slots.len())
            .map_err(|_| "source function slot table overflow")?;
        self.reserve(
            scheme
                .late
                .len()
                .saturating_mul(4 * size_of::<s::SlotRef>()),
        )?;
        for &index in &scheme.late {
            self.charge(1)?;
            let slot = self
                .inputs
                .regions
                .get(&(context.owner, u32::from(index)))
                .copied()
                .ok_or("source callable late slot is not declared by this original")?;
            self.rows.region_slots.push(slot);
        }
        let late = g::Range {
            start,
            end: u32::try_from(self.rows.region_slots.len())
                .map_err(|_| "source function slot table overflow")?,
        };
        Ok(s::RegionScheme {
            total: scheme.total,
            captures: scheme.captures,
            early: g::Range { start, end: start },
            late,
            bounds: self.region_bounds(&scheme.bounds, context)?,
        })
    }
}
