//! Imported opaque identities use the existing actual declaration/ordinal owner.
use super::{Context, Declarations, Error, Graph, Headers, error, s};
use crate::{
    resolve::{body_origin::DeclarationOrigin, model::DefId},
    sema::opaque::Declaration,
    types::{
        GenericArgs, ItemId, Kind, copy, opaque,
        parameters::{
            clauses::Clause,
            predicates::{Bound, Predicate},
        },
        projection, traits,
    },
};

pub(super) fn reserve(
    context: &Context<'_>,
    facts: &mut Declarations,
    budget: &mut Headers,
) -> Result<(), Error> {
    let rows = context.descriptor.rows();
    budget.charge(
        rows.associated_types
            .len()
            .saturating_add(rows.opaque_futures.len()),
    )?;
    for header in &rows.associated_types {
        let definition = context.definition(header.declaration, budget)?;
        let parent = context.definition(header.parent, budget)?;
        budget.reserve(crate::support::retained::new_map::<
            ItemId,
            projection::Schema,
        >(1))?;
        facts
            .types
            .declare_projection(
                ItemId(definition.0),
                projection::Schema {
                    trait_item: ItemId(parent.0),
                    sized: header.sized,
                    copies: copy::Properties {
                        copy: header.copy,
                        clone_: header.clone_,
                    },
                },
            )
            .map_err(error)?;
    }
    for row in &rows.opaque_futures {
        let owner = context.definition(row.declaration, budget)?;
        let ordinal = row.ordinal as usize;
        if facts
            .opaques
            .owners
            .get(&owner)
            .is_some_and(|ids| ordinal < ids.len())
        {
            continue;
        }
        if facts.opaques.owners.get(&owner).map_or(0, Vec::len) != ordinal {
            return Err(error(
                "imported opaque identity has a missing or reordered actual ordinal",
            ));
        }
        budget.reserve(
            crate::support::retained::new_map::<DefId, Vec<opaque::OpaqueId>>(1)
                + 4 * size_of::<opaque::OpaqueId>()
                + crate::types::Store::opaque_declaration_size(),
        )?;
        let id = facts
            .types
            .declare_opaque(opaque::Declaration {
                trait_result: row.trait_result,
                sized: true,
                copies: copy::Properties::default(),
            })
            .map_err(error)?;
        facts.opaques.owners.entry(owner).or_default().push(id);
    }
    Ok(())
}
impl Graph {
    pub(in crate::sema::crates::typed_headers) fn finish_opaques(
        &self,
        context: &Context<'_>,
        facts: &mut Declarations,
        budget: &mut Headers,
    ) -> Result<(), Error> {
        let rows = context.descriptor.rows();
        for row in &rows.opaque_futures {
            let owner = context.definition(row.declaration, budget)?;
            let id = *facts
                .opaques
                .owners
                .get(&owner)
                .and_then(|ids| ids.get(row.ordinal as usize))
                .ok_or_else(|| error("imported opaque lost its actual reserved identity"))?;
            if facts.opaques.declarations.contains_key(&id) {
                continue;
            }
            let Some(s::HeaderRef::Function(index)) =
                rows.header(row.declaration).map_err(error)?
            else {
                return Err(error(
                    "imported opaque owner lost its actual function header",
                ));
            };
            let application = self.ty(rows.functions[index as usize].original.result)?;
            if !matches!(facts.types.kind(application),Some(Kind::Opaque(actual,_)) if actual==id) {
                return Err(error(
                    "imported opaque canonical application belongs to another actual owner",
                ));
            }
            let output = self.ty(row.output)?;
            let future = context.definition(row.future_trait.declaration, budget)?;
            let member = context.definition(row.output_member.declaration, budget)?;
            let trait_application = facts
                .types
                .intern_trait_application(traits::Application {
                    item: ItemId(future.0),
                    arguments: GenericArgs::EMPTY,
                })
                .map_err(error)?;
            let projected = facts
                .types
                .intern(Kind::Projection(projection::Application {
                    item: ItemId(member.0),
                    subject: application,
                    trait_application,
                    arguments: GenericArgs::EMPTY,
                }))
                .map_err(error)?;
            let bounds = vec![
                Clause::Predicate(Predicate {
                    ty: application,
                    bound: Bound::Trait(trait_application),
                }),
                Clause::Predicate(Predicate {
                    ty: projected,
                    bound: Bound::ProjectionEqual(output),
                }),
            ];
            let origin = context.catalog.declaration(owner).map_err(error)?;
            budget.reserve(
                crate::support::retained::new_map::<opaque::OpaqueId, Declaration>(1)
                    + bounds.capacity() * size_of::<Clause>(),
            )?;
            facts.opaques.declarations.insert(
                id,
                Declaration {
                    owner,
                    origin: DeclarationOrigin::Imported(origin),
                    application,
                    bounds,
                    hidden: None,
                    inferred: None,
                    free_regions: Vec::new(),
                },
            );
        }
        Ok(())
    }
}

impl Graph {
    pub(in crate::sema) fn compiler_operation_role(
        &self,
        facts: &Declarations,
        role: crate::sema::compiler_items::Role,
        kind: crate::support::bindings::LangItem,
        steps: &mut usize,
    ) -> Result<(), &'static str> {
        use crate::resolve::{body_origin::DeclarationOrigin, model::DefKind};
        use crate::support::bindings::LangItem;
        self.require_store(&facts.types)?;
        let descriptor = self.descriptor(&self.origins)?;
        if !descriptor.profile().compiler_operations() {
            return Err(
                "declared future/operation roles require their explicit version-eleven imported profile",
            );
        }
        let DeclarationOrigin::Imported(id) = role.origin else {
            return Err("imported compiler role requires its actual catalog declaration");
        };
        let expected = match kind {
            LangItem::Future => DefKind::Trait,
            LangItem::FutureOutput => DefKind::TypeAlias,
            LangItem::FuturePoll | LangItem::AsyncDropInPlace => DefKind::Function,
            _ => return Err("invalid imported compiler-operation role"),
        };
        self.origins
            .validate_declaration(id, role.definition, expected)?;
        let (owner, row, _) = self.origins.declaration_coordinate(id)?;
        let rows = descriptor.rows();
        if owner != descriptor.identity()
            || !rows.roles.iter().any(|binding| {
                binding.role == kind
                    && binding.declaration.declaration == row
                    && binding.declaration.artifact.0 == 0
            })
        {
            return Err("imported compiler role changed its actual provider binding");
        }
        let headers = facts
            .crates
            .as_deref()
            .and_then(|linkage| linkage.originals.as_deref())
            .ok_or("imported compiler role lost its actual canonical header owner")?;
        self.readiness(&self.origins, facts, headers, role.definition, steps)?;
        if kind == LangItem::Future {
            let owner = facts
                .associated
                .owner(role.definition)
                .ok_or("imported Future lost its actual member owner")?;
            for &member in facts
                .associated
                .owner_members(owner)
                .ok_or("imported Future lost its complete actual member inventory")?
            {
                self.readiness(&self.origins, facts, headers, member, steps)?;
            }
        }
        if kind == LangItem::FutureOutput {
            let Some(s::HeaderRef::AssociatedType(index)) = rows.header(row)? else {
                return Err("imported Output lacks its actual associated header");
            };
            if role.parent
                != Some(
                    self.origins
                        .definition_at(owner, rows.associated_types[index as usize].parent)?,
                )
            {
                return Err("imported Output changed its actual trait association");
            }
        }
        Ok(())
    }
}

impl Graph {
    /// A warm result memo cannot replace current catalog, owner and capture proof.
    pub(in crate::sema) fn require_opaque(
        &self,
        facts: &Declarations,
        id: opaque::OpaqueId,
        declaration: &Declaration,
    ) -> Result<(), &'static str> {
        use crate::resolve::{body_origin::DeclarationOrigin, model::DefKind};
        self.require_store(&facts.types)?;
        let descriptor = self.descriptor(&self.origins)?;
        if !descriptor.profile().compiler_operations() {
            return Err("imported opaque requires its explicit compiler-operation profile");
        }
        let DeclarationOrigin::Imported(origin) = declaration.origin else {
            return Err("imported opaque lost its actual declaration origin");
        };
        self.origins
            .validate_declaration(origin, declaration.owner, DefKind::Function)?;
        let (owner, row, _) = self.origins.declaration_coordinate(origin)?;
        if owner != descriptor.identity() {
            return Err("imported opaque belongs to another actual provider");
        }
        let rows = descriptor.rows();
        let Some(s::HeaderRef::Function(index)) = rows.header(row)? else {
            return Err("imported opaque lost its actual function header");
        };
        let function = &rows.functions[index as usize];
        let s::FunctionBody::CompilerOperation(operation) = function.body else {
            return Err("imported opaque lost its actual compiler-operation body");
        };
        let operation = rows
            .compiler_operations
            .get(operation.0 as usize)
            .ok_or("imported opaque operation is outside its actual table")?;
        let Some(s::Type::OpaqueFuture(reference)) =
            rows.types.get(function.original.result.0 as usize)
        else {
            return Err("imported opaque lost its canonical result recipe");
        };
        let future = rows
            .opaque_futures
            .get(reference.0 as usize)
            .ok_or("imported opaque canonical recipe is unknown")?;
        let schema = facts
            .types
            .opaque_declaration(id)
            .ok_or("imported opaque lost its declared canonical schema")?;
        if future.declaration != row
            || operation.declaration != row
            || operation.future != function.original.result
            || operation.output != future.output
            || declaration.application
                != self
                    .ty(function.original.result)
                    .map_err(|_| "imported opaque canonical application is unavailable")?
            || facts
                .opaques
                .owners
                .get(&declaration.owner)
                .and_then(|ids| ids.get(future.ordinal as usize))
                != Some(&id)
            || schema.trait_result != future.trait_result
            || !schema.sized
            || schema.copies != copy::Properties::default()
            || declaration.hidden.is_some()
            || declaration.inferred.is_some()
            || !declaration.free_regions.is_empty()
        {
            return Err(
                "imported opaque changed its actual owner, ordinal, captures, schema or storage authority",
            );
        }
        let (expected_future, expected_output) =
            self.opaque_future_roles(facts, declaration, owner, future)?;
        crate::sema::opaque::check_future_contract(
            &facts.types,
            declaration,
            self.ty(future.output)
                .map_err(|_| "imported opaque Output is unavailable")?,
            expected_future,
            expected_output,
        )
    }
    fn opaque_future_roles(
        &self,
        facts: &Declarations,
        declaration: &Declaration,
        owner: &crate::support::crates::CrateIdentity,
        future: &s::OpaqueFuture,
    ) -> Result<(DefId, DefId), &'static str> {
        use crate::support::bindings::LangItem;
        let catalog = facts
            .source_bindings
            .as_deref()
            .ok_or("imported opaque lost its current role catalog")?;
        let actual_helper = catalog
            .validated_role(LangItem::AsyncDropInPlace)?
            .ok_or("imported opaque helper role is incomplete")?;
        let actual_future = catalog
            .validated_role(LangItem::Future)?
            .ok_or("imported opaque Future role is incomplete")?;
        let actual_output = catalog
            .validated_role(LangItem::FutureOutput)?
            .ok_or("imported opaque Output role is incomplete")?;
        let expected_future = self
            .origins
            .definition_at(owner, future.future_trait.declaration)?;
        let expected_output = self
            .origins
            .definition_at(owner, future.output_member.declaration)?;
        if actual_helper.definition != declaration.owner
            || actual_helper.origin != declaration.origin
            || actual_future.definition != expected_future
            || actual_output.definition != expected_output
            || actual_output.parent != Some(expected_future)
            || actual_future.origin
                != DeclarationOrigin::Imported(self.origins.declaration(expected_future)?)
            || actual_output.origin
                != DeclarationOrigin::Imported(self.origins.declaration(expected_output)?)
        {
            return Err(
                "imported opaque changed its actual completed Future/Output/operation roles",
            );
        }
        let headers = facts
            .crates
            .as_deref()
            .and_then(|linkage| linkage.originals.as_deref())
            .ok_or("imported opaque lost its canonical header owner")?;
        self.readiness(&self.origins, facts, headers, expected_output, &mut 0)?;
        Ok((expected_future, expected_output))
    }
}
