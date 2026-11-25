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
) -> Result<Vec<crate::types::closure::ClosureId>, Error> {
    let rows = context.descriptor.rows();
    let closure_count = rows.closures().map_or(0, |closures| closures.headers.len());
    budget.charge(
        rows.associated_types
            .len()
            .saturating_add(rows.opaque_futures.len())
            .saturating_add(rows.cold().opaques.declarations.len())
            .saturating_add(closure_count),
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
    for row in &rows.cold().opaques.declarations {
        let owner = context.definition(row.declaration, budget)?;
        let ordinal = row.ordinal as usize;
        if facts.opaques.owners.get(&owner).map_or(0, Vec::len) != ordinal {
            return Err(error(
                "imported opaque declaration has a missing or reordered actual ordinal",
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
                trait_result: row.flags.trait_result(),
                sized: row.flags.sized(),
                copies: copy::Properties {
                    copy: row.flags.copy(),
                    clone_: row.flags.clone_(),
                },
            })
            .map_err(error)?;
        facts.opaques.owners.entry(owner).or_default().push(id);
    }
    reserve_closures(rows, facts, budget)
}

fn reserve_closures(
    rows: s::ValidatedView<'_>,
    facts: &mut Declarations,
    budget: &mut Headers,
) -> Result<Vec<crate::types::closure::ClosureId>, Error> {
    let Some(table) = rows.closures() else {
        return Ok(Vec::new());
    };
    let count = table.headers.len();
    let mut closures = Vec::with_capacity(count);
    budget.reserve(count.saturating_mul(
        size_of::<crate::types::closure::ClosureId>()
            + size_of::<crate::types::closure::Declaration>(),
    ))?;
    for header in &table.headers {
        let id = facts.types.reserve_closure().map_err(error)?;
        let kind = crate::types::closure::CallKind::from_trait(header.call_kind);
        if header.constant {
            facts
                .types
                .complete_const_closure(id, kind)
                .map_err(error)?;
        } else {
            facts.types.complete_closure(id, kind).map_err(error)?;
        }
        closures.push(id);
    }
    Ok(closures)
}
impl Graph {
    pub(in crate::sema::crates::typed_headers) fn finish_closures(
        &self,
        context: &Context<'_>,
        facts: &mut Declarations,
        budget: &mut Headers,
    ) -> Result<(), Error> {
        let rows = context.descriptor.rows();
        let Some(table) = rows.closures() else {
            return Ok(());
        };
        budget.charge(table.headers.len())?;
        let dummy = crate::resolve::unit::NodeKey {
            file: crate::resolve::unit::FileId(0),
            node: crate::syntax::ast::NodeId::new(0),
        };
        for (index, header) in table.headers.iter().enumerate() {
            let reference = s::ClosureRef(
                u32::try_from(index).map_err(|_| error("imported closure header overflow"))?,
            );
            let identity = self.closure_id(reference).map_err(error)?;
            let owner = context.definition(header.owner, budget)?;
            let body_id = context
                .catalog
                .body_at(context.descriptor.identity(), header.body)
                .map_err(error)?;
            let origin = context.catalog.origin(body_id).map_err(error)?;
            let signature = self.ty(header.signature)?;
            let execution = header
                .execution
                .map(|exec| {
                    Ok::<_, Error>(crate::sema::closures::Execution {
                        self_ty: self.ty(exec.self_ty)?,
                        signature: self.ty(exec.signature)?,
                        universals: exec.universals,
                        receiver: crate::types::closure::CallKind::from_trait(exec.receiver),
                    })
                })
                .transpose()?;
            facts
                .closures
                .insert_imported(
                    crate::sema::closures::Body {
                        identity,
                        source: dummy,
                        constant: None,
                        body: dummy,
                        parameters: crate::syntax::ast::List { start: 0, end: 0 },
                        signature,
                        internal_result: None,
                        owner: Some(owner),
                        universals: header.universals,
                        inherited: header.inherited,
                        execution,
                    },
                    origin,
                    header.public_binders,
                    header.constant,
                )
                .map_err(error)?;
        }
        Ok(())
    }
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

    pub(in crate::sema::crates::typed_headers) fn finish_opaque_results(
        &self,
        context: &Context<'_>,
        remap: &mut super::super::predicate_remap::Context<'_>,
        facts: &mut Declarations,
        budget: &mut Headers,
    ) -> Result<(), Error> {
        let rows = context.descriptor.rows();
        for row in &rows.cold().opaques.declarations {
            let owner = context.definition(row.declaration, budget)?;
            let id = *facts
                .opaques
                .owners
                .get(&owner)
                .and_then(|ids| ids.get(row.ordinal as usize))
                .ok_or_else(|| error("imported opaque result lost its reserved identity"))?;
            if facts.opaques.declarations.contains_key(&id) {
                return Err(error(
                    "imported opaque result collided with an existing declaration",
                ));
            }
            let application = self.ty(row.application)?;
            if !matches!(facts.types.kind(application), Some(Kind::Opaque(actual, _)) if actual == id)
            {
                return Err(error(
                    "imported opaque result application belongs to another actual owner",
                ));
            }
            let bounds = remap.opaque_bounds(
                row.declaration,
                owner,
                row.bounds,
                &mut facts.types,
                budget,
            )?;
            let hidden = row.hidden.map(|ty| self.ty(ty)).transpose()?;
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
                    hidden,
                    inferred: None,
                    free_regions: Vec::new(),
                },
            );
        }
        Ok(())
    }
}

impl Graph {
    pub(in crate::sema) fn associated_language_role(
        &self,
        facts: &Declarations,
        role: crate::sema::compiler_items::Role,
        kind: crate::support::bindings::LangItem,
        steps: &mut usize,
    ) -> Result<(), &'static str> {
        use crate::{resolve::model::DefKind, support::bindings::LangItem};
        self.require_store(&facts.types)?;
        let descriptor = self.descriptor(&self.origins)?;
        if !descriptor.profile().declaration_policies() {
            return Err("imported associated language roles require the version-fourteen profile");
        }
        let expected = match kind {
            LangItem::PointeeTrait | LangItem::DiscriminantKind => DefKind::Trait,
            LangItem::MetadataType | LangItem::DiscriminantType => DefKind::TypeAlias,
            LangItem::DynMetadata => DefKind::Struct,
            _ => return Err("invalid imported associated language role"),
        };
        *steps = steps.saturating_add(descriptor.rows().roles.len());
        let (row, _) = self.bound_role(facts, role, kind, expected, steps)?;
        if matches!(kind, LangItem::MetadataType | LangItem::DiscriminantType) {
            let rows = descriptor.rows();
            let Some(s::HeaderRef::AssociatedType(index)) = rows.header(row)? else {
                return Err("imported language member lacks its actual associated header");
            };
            if rows.associated_types[index as usize].declared_inputs != 0 {
                return Err("imported language member has unsupported own family inputs");
            }
        }
        Ok(())
    }

    pub(in crate::sema) fn compiler_operation_role(
        &self,
        facts: &Declarations,
        role: crate::sema::compiler_items::Role,
        kind: crate::support::bindings::LangItem,
        steps: &mut usize,
    ) -> Result<(), &'static str> {
        use crate::resolve::model::DefKind;
        use crate::support::bindings::LangItem;
        self.require_store(&facts.types)?;
        let descriptor = self.descriptor(&self.origins)?;
        if !descriptor.profile().compiler_operations() {
            return Err(
                "declared future/operation roles require their explicit version-eleven imported profile",
            );
        }
        let expected = match kind {
            LangItem::Future => DefKind::Trait,
            LangItem::AsyncDrop if descriptor.profile().declaration_policies() => DefKind::Trait,
            LangItem::FutureOutput => DefKind::TypeAlias,
            LangItem::FuturePoll | LangItem::AsyncDropInPlace => DefKind::Function,
            _ => return Err("invalid imported compiler-operation role"),
        };
        let (row, headers) = self.bound_role(facts, role, kind, expected, steps)?;
        let rows = descriptor.rows();
        if matches!(kind, LangItem::Future | LangItem::AsyncDrop) {
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
                != Some(self.origins.definition_at(
                    descriptor.identity(),
                    rows.associated_types[index as usize].parent,
                )?)
            {
                return Err("imported Output changed its actual trait association");
            }
        }
        Ok(())
    }

    /// The version-thirteen `panic` role names this exact declaration. Binding
    /// and readiness are proved here; callers re-authenticate its header.
    pub(in crate::sema) fn declared_panic_role(
        &self,
        facts: &Declarations,
        role: crate::sema::compiler_items::Role,
        kind: crate::support::bindings::LangItem,
        steps: &mut usize,
    ) -> Result<(), &'static str> {
        use crate::{resolve::model::DefKind, support::bindings::LangItem};
        self.require_store(&facts.types)?;
        let descriptor = self.descriptor(&self.origins)?;
        if !descriptor.profile().body_nodes() {
            return Err("imported panic role requires its version-thirteen profile");
        }
        *steps = steps.saturating_add(descriptor.rows().roles.len());
        if kind != LangItem::Panic
            && !(descriptor.profile().declaration_policies()
                && (LangItem::GUARD_PANICS.contains(&kind)
                    || LangItem::FORMAT_PANICS.contains(&kind)))
        {
            return Err("invalid imported panic role");
        }
        self.bound_role(facts, role, kind, DefKind::Function, steps)
            .map(|_| ())
    }

    /// The row names this exact declaration of this provider, and its header is
    /// installed.
    fn bound_role<'a>(
        &self,
        facts: &'a Declarations,
        role: crate::sema::compiler_items::Role,
        kind: crate::support::bindings::LangItem,
        expected: crate::resolve::model::DefKind,
        steps: &mut usize,
    ) -> Result<
        (
            crate::support::crates::owner_catalog::DeclarationRef,
            &'a super::super::Headers,
        ),
        &'static str,
    > {
        let descriptor = self.descriptor(&self.origins)?;
        let DeclarationOrigin::Imported(id) = role.origin else {
            return Err("imported compiler role requires its actual catalog declaration");
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
        Ok((row, headers))
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
        if descriptor.profile().declaration_policies()
            && let Some(ordinal) = facts
                .opaques
                .owners
                .get(&declaration.owner)
                .and_then(|ids| ids.iter().position(|&actual| actual == id))
                .and_then(|ordinal| u32::try_from(ordinal).ok())
            && let Ok(decl_index) = rows
                .cold()
                .opaques
                .declarations
                .binary_search_by_key(&(row.0, ordinal), |entry| {
                    (entry.declaration.0, entry.ordinal)
                })
        {
            return self.require_opaque_result(
                facts,
                id,
                declaration,
                rows.cold().opaques.declarations[decl_index],
            );
        }
        let function = &rows.functions[index as usize];
        let Some(s::Type::OpaqueFuture(reference)) =
            rows.types.get(function.original.result.0 as usize)
        else {
            return Err("imported opaque lost its canonical result recipe");
        };
        let future = rows
            .opaque_futures
            .get(reference.0 as usize)
            .ok_or("imported opaque canonical recipe is unknown")?;
        require_opaque_body(
            rows,
            descriptor.profile().declaration_policies(),
            row,
            function,
            future,
        )?;
        let schema = facts
            .types
            .opaque_declaration(id)
            .ok_or("imported opaque lost its declared canonical schema")?;
        if future.declaration != row
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

    fn require_opaque_result(
        &self,
        facts: &Declarations,
        id: opaque::OpaqueId,
        declaration: &Declaration,
        expected: s::OpaqueDeclaration,
    ) -> Result<(), &'static str> {
        let schema = facts
            .types
            .opaque_declaration(id)
            .ok_or("imported opaque result lost its declared canonical schema")?;
        let application = self
            .ty(expected.application)
            .map_err(|_| "imported opaque result canonical application is unavailable")?;
        let hidden = expected
            .hidden
            .map(|ty| {
                self.ty(ty)
                    .map_err(|_| "imported opaque result hidden type is unavailable")
            })
            .transpose()?;
        let bound_count = expected.bounds.end.saturating_sub(expected.bounds.start) as usize;
        if declaration.application != application
            || declaration.hidden != hidden
            || declaration.inferred.is_some()
            || !declaration.free_regions.is_empty()
            || declaration.bounds.len() != bound_count
            || schema.trait_result != expected.flags.trait_result()
            || schema.sized != expected.flags.sized()
            || schema.copies.copy != expected.flags.copy()
            || schema.copies.clone_ != expected.flags.clone_()
        {
            return Err(
                "imported opaque result changed its actual application, schema, hidden type or bounds",
            );
        }
        Ok(())
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
        if !future.trait_result {
            let actual_helper = catalog
                .validated_role(LangItem::AsyncDropInPlace)?
                .ok_or("imported opaque helper role is incomplete")?;
            if actual_helper.definition != declaration.owner
                || actual_helper.origin != declaration.origin
            {
                return Err(
                    "imported opaque changed its actual completed Future/Output/operation roles",
                );
            }
        }
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
        if actual_future.definition != expected_future
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

fn require_opaque_body(
    rows: s::ValidatedView<'_>,
    declaration_policies: bool,
    row: crate::support::crates::owner_catalog::DeclarationRef,
    function: &s::FunctionHeader,
    future: &s::OpaqueFuture,
) -> Result<(), &'static str> {
    if future.trait_result {
        if !declaration_policies
            || function.body != s::FunctionBody::Absent
            || !function.parent.is_some_and(|parent| {
                matches!(rows.header(parent), Ok(Some(s::HeaderRef::Trait(_))))
            })
        {
            return Err("imported trait opaque lost its actual required trait declaration");
        }
        return Ok(());
    }
    let s::FunctionBody::CompilerOperation(operation) = function.body else {
        return Err("imported opaque lost its actual compiler-operation body");
    };
    let operation = rows
        .compiler_operations
        .get(operation.0 as usize)
        .ok_or("imported opaque operation is outside its actual table")?;
    if operation.declaration != row
        || operation.future != function.original.result
        || operation.output != future.output
    {
        return Err(
            "imported opaque changed its actual owner, ordinal, captures, schema or storage authority",
        );
    }
    Ok(())
}
