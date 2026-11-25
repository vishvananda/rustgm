//! Genuine declared results and helper operations retain their existing owners.
use super::{DeclarationRef, Declarations, DefId, Report, Stage, canonical_nodes::Context, g, s};
use crate::{
    support::{
        bindings::LangItem,
        hash::{HashMap, HashSet},
        retained,
    },
    types::{GenericArgs, ItemId, Kind, RegionId, TypeId, opaque::OpaqueId, projection},
};

#[derive(Default)]
pub(super) struct OpaqueQueue {
    contexts: HashMap<DefId, Context>,
    queued: HashSet<OpaqueId>,
    pending: Vec<(OpaqueId, DefId, DeclarationRef, u32)>,
}
impl OpaqueQueue {
    pub(super) fn require_empty(&self) -> Result<(), &'static str> {
        if !self.pending.is_empty() {
            return Err("source opaque declarations were queued after header completion");
        }
        Ok(())
    }
}

impl Stage {
    pub(super) fn record_opaque_context(
        &mut self,
        definition: DefId,
        context: Context,
    ) -> Result<(), &'static str> {
        self.reserve(retained::new_map::<DefId, Context>(1))?;
        self.opaques.contexts.insert(definition, context);
        Ok(())
    }

    pub(super) fn opaque_type(
        &mut self,
        facts: &Declarations,
        id: OpaqueId,
        captures: GenericArgs,
        context: Context,
    ) -> Result<s::Type, &'static str> {
        require_function_opaque(facts, id)?;
        let contract = facts
            .opaques
            .declarations
            .get(&id)
            .ok_or("opaque export lost its actual declaration")?;
        if facts.types.authored_future_template(id)?.is_some() {
            return self.opaque_future(facts, id, captures, context);
        }
        let function = facts
            .function_indices
            .get(&contract.owner)
            .and_then(|&index| facts.functions.get(index))
            .ok_or("opaque export lost its actual function")?;
        let schema = facts
            .types
            .opaque_declaration(id)
            .ok_or("opaque export lost its canonical declaration schema")?;
        let helper = facts
            .source_bindings
            .as_deref()
            .map(|catalog| catalog.validated_language(LangItem::AsyncDropInPlace))
            .transpose()?
            .flatten()
            .is_some_and(|role| role.definition == contract.owner);
        let required_async_trait = self.declaration_policies()
            && schema.trait_result
            && function.body.is_none()
            && facts.opaques.body_results.contains_key(&contract.owner);
        if helper || required_async_trait {
            return self.opaque_future(facts, id, captures, context);
        }
        if !self.declaration_policies() {
            return Err(
                "authored async body/storage export requires its complete typed ownership profile",
            );
        }
        self.opaque_result(facts, id, contract.owner, captures, context)
    }

    fn opaque_result(
        &mut self,
        facts: &Declarations,
        id: OpaqueId,
        owner: DefId,
        captures: GenericArgs,
        context: Context,
    ) -> Result<s::Type, &'static str> {
        let ordinal = facts
            .opaques
            .owners
            .get(&owner)
            .and_then(|ids| ids.iter().position(|&actual| actual == id))
            .ok_or("opaque export lost its actual owner ordinal")?;
        let ordinal = u32::try_from(ordinal).map_err(|_| "opaque ordinal overflow")?;
        let declaration = self.declaration(owner)?;
        // Export capture children first so applications stay in exact type-row order.
        let captures = self.arguments(facts, captures, context)?;
        self.reserve(4 * size_of::<s::OpaqueApplication>())?;
        let applications = &mut self.cold()?.opaques.applications;
        let reference = s::OpaqueResultRef(
            u32::try_from(applications.len()).map_err(|_| "opaque result recipe overflow")?,
        );
        applications.push(s::OpaqueApplication {
            declaration,
            ordinal,
            captures,
        });
        if self.opaques.queued.insert(id) {
            self.reserve(
                retained::new_map::<OpaqueId, ()>(1)
                    + 4 * size_of::<(OpaqueId, DefId, DeclarationRef, u32)>(),
            )?;
            self.opaques.pending.push((id, owner, declaration, ordinal));
        }
        Ok(s::Type::OpaqueResult(reference))
    }

    pub(super) fn complete_opaque_declarations(
        &mut self,
        facts: &Declarations,
    ) -> Result<(), &'static str> {
        while let Some((id, owner, declaration, ordinal)) = self.opaques.pending.pop() {
            self.charge(1)?;
            let contract = facts
                .opaques
                .declarations
                .get(&id)
                .ok_or("opaque declaration export lost its actual contract")?;
            let schema = facts
                .types
                .opaque_declaration(id)
                .ok_or("opaque declaration export lost its canonical schema")?;
            let context = self
                .opaques
                .contexts
                .get(&owner)
                .copied()
                .ok_or("opaque declaration export lost its owning function context")?;
            let application = self.ty(facts, contract.application, context)?;
            let source = contract
                .origin
                .source()
                .ok_or("opaque declaration export lost its actual source origin")?;
            let start = u32::try_from(self.rows.clauses.len())
                .map_err(|_| "opaque bound table overflow")?;
            for &value in &contract.bounds {
                self.clause(
                    facts,
                    crate::sema::declaration_inputs::Input {
                        source: source.into(),
                        value,
                    },
                    context,
                    None,
                )?;
            }
            let end = u32::try_from(self.rows.clauses.len())
                .map_err(|_| "opaque bound table overflow")?;
            let hidden = match contract.hidden {
                Some(ty) if self.transportable_hidden_type(facts, ty)? => {
                    Some(self.ty(facts, ty, context)?)
                }
                _ => None,
            };
            self.reserve(4 * size_of::<s::OpaqueDeclaration>())?;
            self.cold()?
                .opaques
                .declarations
                .push(s::OpaqueDeclaration {
                    declaration,
                    ordinal,
                    application,
                    bounds: g::Range { start, end },
                    hidden,
                    flags: s::OpaqueFlags::new(schema.sized, schema.trait_result)
                        .with_copies(schema.copies.copy, schema.copies.clone_),
                });
        }
        let count = self.rows.cold().opaques.declarations.len();
        if count > 1 {
            self.charge(count.saturating_mul(count.checked_ilog2().unwrap_or(0) as usize + 1))?;
            self.cold()?
                .opaques
                .declarations
                .sort_unstable_by_key(|row| (row.declaration.0, row.ordinal));
        }
        Ok(())
    }

    fn transportable_hidden_type(
        &mut self,
        facts: &Declarations,
        root: TypeId,
    ) -> Result<bool, &'static str> {
        let mut seen = HashSet::default();
        let mut pending = vec![root];
        while let Some(ty) = pending.pop() {
            if !seen.insert(ty) {
                continue;
            }
            self.charge(1)?;
            let Some(kind) = facts.types.kind(ty) else {
                return Ok(false);
            };
            if !self.transportable_hidden_kind(facts, kind, &mut pending)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn transportable_hidden_kind(
        &self,
        facts: &Declarations,
        kind: Kind,
        pending: &mut Vec<TypeId>,
    ) -> Result<bool, &'static str> {
        use crate::types::Constraint;
        let has_def = |item: ItemId| {
            self.definitions
                .get(item.0 as usize)
                .copied()
                .flatten()
                .is_some()
        };
        match kind {
            Kind::Primitive(_) | Kind::Never => Ok(true),
            Kind::Parameter(param) => Ok(self.graph.parameters.contains_key(&param)),
            Kind::Foreign(item) => Ok(has_def(item)),
            Kind::Reference {
                region, referent, ..
            } => {
                if region != RegionId::STATIC && region.bound_parts().is_none() {
                    return Ok(false);
                }
                pending.push(referent);
                Ok(true)
            }
            Kind::Pointer { pointee: child, .. }
            | Kind::Slice(child)
            | Kind::Array { element: child, .. } => {
                pending.push(child);
                Ok(true)
            }
            Kind::ArrayTerm { element, length } => {
                if !self.transportable_hidden_const(facts, length, pending) {
                    return Ok(false);
                }
                pending.push(element);
                Ok(true)
            }
            Kind::Pattern { base, constraint } => {
                if constraint == Constraint::NonNull
                    && !matches!(
                        facts.types.kind(base),
                        Some(Kind::Pointer { .. } | Kind::Parameter(_))
                    )
                {
                    return Ok(false);
                }
                pending.push(base);
                Ok(true)
            }
            Kind::Tuple(elements) => {
                let Some(list) = facts.types.list(elements) else {
                    return Ok(false);
                };
                pending.extend_from_slice(list);
                Ok(true)
            }
            Kind::Adt(item, arguments) => {
                Ok(has_def(item) && self.transportable_hidden_args(facts, arguments, pending))
            }
            Kind::Projection(app) => {
                let Some(trait_app) = facts.types.trait_application(app.trait_application) else {
                    return Ok(false);
                };
                if facts.types.projection_schema(app.item).is_none()
                    || !has_def(app.item)
                    || !has_def(trait_app.item)
                {
                    return Ok(false);
                }
                pending.push(app.subject);
                Ok(
                    self.transportable_hidden_args(facts, trait_app.arguments, pending)
                        && self.transportable_hidden_args(facts, app.arguments, pending),
                )
            }
            Kind::Opaque(id, arguments) => Ok(!facts.is_named_opaque(id)
                && facts.types.authored_future_template(id)?.is_none()
                && self.transportable_hidden_args(facts, arguments, pending)),
            Kind::Dynamic {
                shape,
                arguments,
                region,
                ..
            } => self.transportable_hidden_dynamic(facts, shape, arguments, region, pending),
            _ => Ok(false),
        }
    }

    fn transportable_hidden_dynamic(
        &self,
        facts: &Declarations,
        shape: crate::types::dynamic::ShapeId,
        arguments: GenericArgs,
        region: RegionId,
        pending: &mut Vec<TypeId>,
    ) -> Result<bool, &'static str> {
        let has_def = |item: ItemId| {
            self.definitions
                .get(item.0 as usize)
                .copied()
                .flatten()
                .is_some()
        };
        if region != RegionId::STATIC && region.bound_parts().is_none() {
            return Ok(false);
        }
        let Some(actual) = facts.types.dynamic_shape(shape) else {
            return Ok(false);
        };
        if actual.principal.is_some_and(|item| !has_def(item)) {
            return Ok(false);
        }
        let Some(bindings) = facts.types.dynamic_bindings(shape) else {
            return Ok(false);
        };
        if bindings
            .iter()
            .any(|b| !has_def(b.item) || !has_def(b.trait_item))
        {
            return Ok(false);
        }
        let Some(markers) = facts.types.dynamic_auto_traits(shape) else {
            return Ok(false);
        };
        if !markers.is_empty() {
            let Some(catalog) = facts.source_bindings.as_deref() else {
                return Ok(false);
            };
            for &marker in markers {
                if catalog
                    .auto_marker(marker.lang_item(), true)?
                    .is_none_or(|role| !has_def(ItemId(role.definition.0)))
                {
                    return Ok(false);
                }
            }
        }
        Ok(self.transportable_hidden_args(facts, arguments, pending))
    }

    fn transportable_hidden_args(
        &self,
        facts: &Declarations,
        arguments: GenericArgs,
        pending: &mut Vec<TypeId>,
    ) -> bool {
        use crate::types::arguments::Argument;
        let Some(actual) = facts.types.arguments(arguments) else {
            return false;
        };
        for argument in actual {
            match argument {
                Argument::Type(ty) => pending.push(ty),
                Argument::Lifetime(region) => {
                    if region != RegionId::STATIC && region.bound_parts().is_none() {
                        return false;
                    }
                }
                Argument::Const(term) => {
                    if !self.transportable_hidden_const(facts, term, pending) {
                        return false;
                    }
                }
            }
        }
        true
    }

    fn transportable_hidden_const(
        &self,
        facts: &Declarations,
        term: crate::types::arguments::ConstId,
        pending: &mut Vec<TypeId>,
    ) -> bool {
        use crate::types::arguments::{ConstKind, trees::Aggregate};
        match facts.types.const_kind(term) {
            Some(ConstKind::Value(value)) => {
                pending.push(value.ty);
                true
            }
            Some(ConstKind::Parameter(param)) => self.graph.parameters.contains_key(&param),
            Some(ConstKind::Aggregate(aggregate)) => {
                let Some(Aggregate::Fields {
                    ty,
                    variant: Some(ordinal),
                    values: &[],
                }) = facts.types.aggregate_constant(aggregate)
                else {
                    return false;
                };
                let Some(Kind::Adt(item, GenericArgs::EMPTY)) = facts.types.kind(ty) else {
                    return false;
                };
                let exported = facts
                    .members
                    .variants(item)
                    .get(ordinal as usize)
                    .and_then(|id| self.variants.get(id.0 as usize).copied().flatten())
                    .is_some();
                if exported {
                    pending.push(ty);
                }
                exported
            }
            None => false,
        }
    }

    pub(super) fn projection_recipe(
        &mut self,
        facts: &Declarations,
        application: projection::Application,
        context: Context,
    ) -> Result<s::Type, &'static str> {
        facts
            .types
            .projection_schema(application.item)
            .ok_or("projection export requires its actual completed schema")?;
        let declaration = self.address(DefId(application.item.0))?;
        let subject = self.ty(facts, application.subject, context)?;
        let trait_application = self.trait_application(
            facts,
            application.subject,
            application.trait_application,
            context,
        )?;
        let arguments = self.arguments(facts, application.arguments, context)?;
        self.reserve(4 * size_of::<s::ProjectionRecipe>())?;
        let reference = s::ProjectionRef(
            u32::try_from(self.rows.projections.len())
                .map_err(|_| "source projection recipe overflow")?,
        );
        self.rows.projections.push(s::ProjectionRecipe {
            declaration,
            subject,
            trait_application,
            arguments,
        });
        Ok(s::Type::Projection(reference))
    }
    pub(super) fn opaque_future(
        &mut self,
        facts: &Declarations,
        id: OpaqueId,
        captures: GenericArgs,
        context: Context,
    ) -> Result<s::Type, &'static str> {
        require_function_opaque(facts, id)?;
        let contract = facts
            .opaques
            .declarations
            .get(&id)
            .ok_or("opaque export lost its actual declaration")?;
        if facts.types.authored_future_template(id)?.is_some() {
            contract.require_current(facts, id)?;
            let function = facts
                .function_indices
                .get(&contract.owner)
                .and_then(|&index| facts.functions.get(index))
                .ok_or("authored async export lost its actual source function")?;
            facts
                .authored_async_body(function)?
                .ok_or("authored async export changed its actual source ownership")?;
            return Err(
                "authored async state-machine construction/storage export requires its complete typed ownership profile",
            );
        }
        let catalog = facts
            .source_bindings
            .as_deref()
            .ok_or("opaque export lost its source role catalog")?;
        let function = facts
            .function_indices
            .get(&contract.owner)
            .and_then(|&index| facts.functions.get(index))
            .ok_or("opaque export lost its actual function")?;
        let output = *facts
            .opaques
            .body_results
            .get(&contract.owner)
            .ok_or("opaque export requires its actual checked async Output")?;
        let schema = facts
            .types
            .opaque_declaration(id)
            .ok_or("opaque export lost its canonical declaration schema")?;
        require_opaque_owner(
            facts,
            catalog,
            contract,
            function,
            schema,
            self.declaration_policies(),
        )?;
        let ordinal = facts
            .opaques
            .owners
            .get(&contract.owner)
            .and_then(|ids| ids.iter().position(|&actual| actual == id))
            .ok_or("opaque export lost its actual owner ordinal")?;
        let future = catalog
            .validated_language(LangItem::Future)?
            .ok_or("opaque export lacks actual Future role")?;
        let member = catalog
            .validated_language(LangItem::FutureOutput)?
            .ok_or("opaque export lacks actual Output role")?;
        if !schema.sized
            || schema.copies != crate::types::copy::Properties::default()
            || contract.bounds.len() != 2
        {
            return Err("opaque export requires its complete supported declared Future bounds");
        }
        self.charge(contract.bounds.len())?;
        crate::sema::opaque::check_future_contract(
            &facts.types,
            contract,
            output,
            future.definition,
            member.definition,
        )?;
        let declaration = self.declaration(contract.owner)?;
        let captures = self.arguments(facts, captures, context)?;
        let output = self.ty(facts, output, context)?;
        let future_trait = self.address(future.definition)?;
        let output_member = self.address(member.definition)?;
        self.reserve(4 * size_of::<s::OpaqueFuture>())?;
        let reference = s::OpaqueFutureRef(
            u32::try_from(self.rows.opaque_futures.len())
                .map_err(|_| "opaque future recipe overflow")?,
        );
        self.rows.opaque_futures.push(s::OpaqueFuture {
            declaration,
            ordinal: u32::try_from(ordinal).map_err(|_| "opaque ordinal overflow")?,
            captures,
            output,
            future_trait,
            output_member,
            trait_result: schema.trait_result,
        });
        Ok(s::Type::OpaqueFuture(reference))
    }
    pub(super) fn associated_headers(
        &mut self,
        report: &Report,
        facts: &Declarations,
    ) -> Result<(), &'static str> {
        for member in facts.associated.members() {
            if facts.associated.type_contract(member.definition).is_none() {
                continue;
            }
            let contract = facts
                .associated
                .require_projection_header(member.definition)?;
            let parent=facts.associated.trait_owner(member.definition).ok_or("associated implementation result transport requires its complete definition profile")?;
            let family = facts.associated.families.get(&member.definition);
            if family.is_some() && !self.declaration_policies() {
                return Err("associated own-family transport requires its complete wider profile");
            }
            let captures = facts
                .associated
                .schema(parent)
                .ok_or("associated result lost its actual trait scope")?
                .captures;
            let regions = family.map_or(captures, |family| family.total);
            if !self.declaration_policies() && regions != 0 {
                return Err("associated captured lifetimes require their wider profile");
            }
            let schema = facts
                .types
                .projection_schema(ItemId(member.definition.0))
                .ok_or("associated result export lacks its completed schema")?;
            let declaration = self.declaration(member.definition)?;
            let parent = self.declaration(parent)?;
            let context =
                self.frame(declaration, None, s::BinderKind::DeclarationHeader, regions)?;
            let projection = self.ty(facts, contract.projection, context)?;
            let source = report
                .resolution
                .definitions
                .get(member.definition.0 as usize)
                .and_then(|row| row.node)
                .ok_or("associated result export lost its actual declaration source")?;
            let inputs =
                self.associated_family_inputs(facts, member.definition, family, context)?;
            let start = inputs.end;
            for &value in &contract.bounds {
                self.clause(
                    facts,
                    crate::sema::declaration_inputs::Input {
                        source: source.into(),
                        value,
                    },
                    context,
                    None,
                )?;
            }
            let bounds = g::Range {
                start,
                end: u32::try_from(self.rows.clauses.len())
                    .map_err(|_| "associated bound table overflow")?,
            };
            self.reserve(4 * size_of::<s::AssociatedTypeHeader>())?;
            self.rows.associated_types.push(s::AssociatedTypeHeader {
                declaration,
                parent,
                scope: declaration,
                projection,
                inputs,
                declared_inputs: inputs.end - inputs.start,
                bounds,
                sized: schema.sized,
                copy: schema.copies.copy,
                clone_: schema.copies.clone_,
            });
        }
        Ok(())
    }

    pub(super) fn associated_family_inputs(
        &mut self,
        facts: &Declarations,
        definition: DefId,
        family: Option<&crate::sema::associated::Family>,
        context: Context,
    ) -> Result<g::Range, &'static str> {
        let length = |count| u32::try_from(count).map_err(|_| "associated input table overflow");
        let start = length(self.rows.clauses.len())?;
        if let Some(family) = family {
            if !family.regions.is_empty() {
                return Err(
                    "associated written region relations require their complete wider profile",
                );
            }
            let actual = family
                .requirements
                .as_deref()
                .ok_or("associated family input contract is not complete")?;
            let predicates = facts
                .generics
                .predicates
                .get(&definition)
                .map_or(&[][..], Vec::as_slice);
            let inputs = predicates
                .get(..actual.len())
                .ok_or("associated family lost its actual input prefix")?;
            for (predicate, &required) in inputs.iter().zip(actual) {
                if predicate.value != required {
                    return Err("associated family input changed its actual source predicate");
                }
                self.clause(
                    facts,
                    crate::sema::declaration_inputs::Input {
                        source: predicate.source.into(),
                        value: predicate.value,
                    },
                    context,
                    None,
                )?;
            }
        }
        Ok(g::Range {
            start,
            end: length(self.rows.clauses.len())?,
        })
    }
}

fn require_opaque_owner(
    facts: &Declarations,
    catalog: &crate::sema::compiler_items::Catalog,
    contract: &crate::sema::opaque::Declaration,
    function: &crate::sema::model::Function,
    schema: crate::types::opaque::Declaration,
    declaration_policies: bool,
) -> Result<(), &'static str> {
    let required_trait = declaration_policies
        && schema.trait_result
        && function.body.is_none()
        && facts.associated.trait_owner(contract.owner).is_some();
    if !required_trait {
        let helper = catalog
            .validated_language(LangItem::AsyncDropInPlace)?
            .ok_or("opaque export requires its authenticated compiler-operation owner")?;
        if helper.definition != contract.owner
            || schema.trait_result
            || !facts.typed_function_bodies.contains(&contract.owner)
        {
            return Err(
                "authored async body/storage export requires its complete typed ownership profile",
            );
        }
    }
    if contract.origin.source() != Some(function.node)
        || contract.hidden.is_some()
        || contract.inferred.is_some()
        || !contract.free_regions.is_empty()
    {
        return Err(
            "authored async body/storage export requires its complete typed ownership profile",
        );
    }
    Ok(())
}

fn require_function_opaque(facts: &Declarations, id: OpaqueId) -> Result<(), &'static str> {
    if facts.is_named_opaque(id) {
        return Err("named opaque source metadata publication is outside the supported profile");
    }
    Ok(())
}
