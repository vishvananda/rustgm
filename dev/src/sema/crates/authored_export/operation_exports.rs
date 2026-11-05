//! Genuine declared results and helper operations retain their existing owners.
use super::{Declarations, DefId, Report, Stage, canonical_nodes::Context, g, s};
use crate::{
    support::bindings::LangItem,
    types::{GenericArgs, ItemId, opaque::OpaqueId, projection},
};

impl Stage {
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
        let contract = facts
            .opaques
            .declarations
            .get(&id)
            .ok_or("opaque export lost its actual declaration")?;
        let catalog = facts
            .source_bindings
            .as_deref()
            .ok_or("opaque export lost its source role catalog")?;
        let helper = catalog
            .validated_language(LangItem::AsyncDropInPlace)?
            .ok_or("opaque export requires its authenticated compiler-operation owner")?;
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
        if helper.definition != contract.owner
            || contract.origin.source() != Some(function.node)
            || !facts.typed_function_bodies.contains(&contract.owner)
            || contract.hidden.is_some()
            || contract.inferred.is_some()
            || !contract.free_regions.is_empty()
        {
            return Err(
                "authored async body/storage export requires its complete typed ownership profile",
            );
        }
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
        let schema = facts
            .types
            .opaque_declaration(id)
            .ok_or("opaque export lost its canonical declaration schema")?;
        if schema.trait_result
            || !schema.sized
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
            if facts.associated.families.contains_key(&member.definition)
                || facts
                    .generics
                    .parameters
                    .get(&member.definition)
                    .is_some_and(|parameters| !parameters.is_empty())
            {
                return Err("associated own-family transport requires its complete wider profile");
            }
            let schema = facts
                .types
                .projection_schema(ItemId(member.definition.0))
                .ok_or("associated result export lacks its completed schema")?;
            let declaration = self.declaration(member.definition)?;
            let parent = self.declaration(parent)?;
            let context = self.frame(declaration, None, s::BinderKind::DeclarationHeader, 0)?;
            let projection = self.ty(facts, contract.projection, context)?;
            let source = report
                .resolution
                .definitions
                .get(member.definition.0 as usize)
                .and_then(|row| row.node)
                .ok_or("associated result export lost its actual declaration source")?;
            let start = u32::try_from(self.rows.clauses.len())
                .map_err(|_| "associated input table overflow")?;
            let end = u32::try_from(self.rows.clauses.len())
                .map_err(|_| "associated input table overflow")?;
            let inputs = g::Range { start, end };
            let start = end;
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
}
