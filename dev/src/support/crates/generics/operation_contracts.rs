//! Deferred recipes retain canonical contracts without publishing executable code.
use super::{
    Argument, BodylessIntrinsic, CompilerOperationKind, FunctionBody, HeaderIndex, HeaderRef,
    ParameterKind, Profile, Region, SourceRows, Type,
};
use crate::support::{
    bindings::LangItem,
    constness::Constness,
    crates::{
        Scalar,
        generics::{Convention, Work},
    },
};

pub(super) fn require_profile(
    rows: &SourceRows,
    profile: Profile,
    work: &mut Work,
) -> Result<(), &'static str> {
    work.charge(rows.types.len().saturating_add(rows.functions.len()))?;
    bodyless_intrinsics(rows, profile, work)?;
    declared_intrinsics(rows, profile, work)?;
    comptime_functions(rows, profile, work)?;
    // Version thirteen carries declared results and their projections; only
    // version eleven carries opaque futures and symbolic operations.
    let results = profile.compiler_operations() || profile.body_nodes();
    if !profile.body_nodes() && !rows.associated_definitions().is_empty() {
        return Err(
            "associated result definitions require their explicit version-thirteen profile",
        );
    }
    if !profile.declaration_policies()
        && (!rows.cold().opaques.is_empty()
            || rows
                .types
                .iter()
                .any(|ty| matches!(ty, Type::OpaqueResult(_))))
    {
        return Err("source return-position opaque transport requires version fourteen");
    }
    if !results
        && (!rows.associated_types.is_empty()
            || !rows.projections.is_empty()
            || rows
                .types
                .iter()
                .any(|ty| matches!(ty, Type::Projection(_))))
        || !profile.compiler_operations()
            && (!rows.opaque_futures.is_empty()
                || !rows.compiler_operations.is_empty()
                || rows
                    .types
                    .iter()
                    .any(|ty| matches!(ty, Type::OpaqueFuture(_)))
                || rows
                    .functions
                    .iter()
                    .any(|row| matches!(row.body, FunctionBody::CompilerOperation(_))))
    {
        return Err(
            "compiler operations and declared results require their explicit version-eleven profile",
        );
    }
    Ok(())
}

fn bodyless_intrinsics(
    rows: &SourceRows,
    profile: Profile,
    work: &mut Work,
) -> Result<(), &'static str> {
    work.charge(rows.functions.len())?;
    let mut seen = [false; BodylessIntrinsic::ALL.len()];
    for header in &rows.functions {
        let FunctionBody::Intrinsic(operation) = header.body else {
            continue;
        };
        if operation == BodylessIntrinsic::CallerLocation {
            work.charge(rows.roles.len())?;
            role(rows, LangItem::PanicLocation)?;
        }
        if operation == BodylessIntrinsic::OffsetOf {
            work.charge(rows.roles.len())?;
            let address = role(rows, LangItem::OffsetOf)?;
            if address.artifact.0 != 0 || address.declaration != header.declaration {
                return Err(
                    "source offset_of intrinsic changed its actual bound language declaration",
                );
            }
        }
        if !profile.declaration_policies()
            || std::mem::replace(&mut seen[operation as usize], true)
            || !bodyless_header(rows, header, operation)
            || !bodyless_inputs(rows, header, operation, work)?
        {
            return Err(match operation {
                BodylessIntrinsic::CallerLocation => {
                    "source caller_location intrinsic requires its exact version-fourteen header"
                }
                BodylessIntrinsic::PtrMetadata => {
                    "source ptr_metadata intrinsic requires its exact version-fourteen header"
                }
                BodylessIntrinsic::OffsetOf => {
                    "source offset_of intrinsic requires its exact version-fourteen header"
                }
                BodylessIntrinsic::FieldOffset => {
                    "source field_offset intrinsic requires its exact version-fourteen header"
                }
                _ => "source layout intrinsic requires its exact version-fourteen header",
            });
        }
    }
    Ok(())
}

fn declared_intrinsics(
    rows: &SourceRows,
    profile: Profile,
    work: &mut Work,
) -> Result<(), &'static str> {
    let intrinsics = &rows.cold().declared_intrinsics;
    if intrinsics.is_empty() {
        return Ok(());
    }
    if !profile.declaration_policies() {
        return Err("source intrinsic declaration semantics require a wider metadata profile");
    }
    work.charge(intrinsics.len())?;
    let mut previous = None;
    for entry in intrinsics {
        if previous.is_some_and(|prior| prior >= entry.declaration.0) {
            return Err("source declared intrinsics are duplicated or out of order");
        }
        previous = Some(entry.declaration.0);
        let Ok(index) = rows
            .functions
            .binary_search_by_key(&entry.declaration.0, |row| row.declaration.0)
        else {
            return Err("source declared intrinsic lacks its actual function header");
        };
        let header = &rows.functions[index];
        let decl = rows
            .declarations
            .get(entry.declaration.0 as usize)
            .ok_or("source declared intrinsic lacks its declaration row")?;
        let module_owned = decl.definition.parent.is_some_and(|parent| {
            rows.declarations
                .get(parent.0 as usize)
                .is_some_and(|owner| owner.definition.kind == super::DefinitionKind::Module)
        });
        let Some(kind) = decl
            .name
            .as_deref()
            .and_then(crate::support::bindings::SourceIntrinsic::from_name)
        else {
            return Err("source declared intrinsic names an unknown intrinsic");
        };
        let special = BodylessIntrinsic::ALL
            .into_iter()
            .any(|op| op.kind() == kind);
        let valid_body = match header.body {
            FunctionBody::Absent => !entry.source_fallback && !special,
            FunctionBody::Original(_) => {
                !entry.source_fallback || header.constness == Constness::Const
            }
            FunctionBody::Intrinsic(_) | FunctionBody::CompilerOperation(_) => false,
        };
        let valid_signature =
            |sig: super::super::Signature| sig.convention == Convention::Rust && !sig.variadic;
        if !module_owned
            || header.parent.is_some()
            || header.receiver.is_some()
            || !valid_body
            || !valid_signature(header.original)
            || !valid_signature(header.callable)
        {
            return Err("source declared intrinsic changed its checked declaration contract");
        }
    }
    Ok(())
}

fn comptime_functions(
    rows: &SourceRows,
    profile: Profile,
    work: &mut Work,
) -> Result<(), &'static str> {
    let comptime = &rows.cold().comptime_functions;
    if comptime.is_empty() {
        return Ok(());
    }
    if !profile.declaration_policies() {
        return Err("always-comptime source contracts require a wider transport profile");
    }
    work.charge(comptime.len())?;
    let mut previous = None;
    for &declaration in comptime {
        if previous.is_some_and(|prior| prior >= declaration.0) {
            return Err("source comptime declarations are duplicated or out of order");
        }
        previous = Some(declaration.0);
        let Ok(index) = rows
            .functions
            .binary_search_by_key(&declaration.0, |row| row.declaration.0)
        else {
            return Err("source comptime declaration lacks its actual function header");
        };
        let header = &rows.functions[index];
        let valid_owner = match header.parent {
            None => true,
            Some(parent) => rows
                .implementations
                .binary_search_by_key(&parent.0, |row| row.self_definition.0)
                .is_ok_and(|idx| rows.implementations[idx].trait_application.is_none()),
        };
        let clauses = header.inputs.slice(&rows.clauses)?;
        work.charge(clauses.len())?;
        let has_conditional = clauses.iter().any(|clause| {
            matches!(
                clause.requirement,
                super::Requirement::Trait {
                    effect: Constness::Conditional,
                    ..
                }
            )
        });
        if header.constness != Constness::Const || !valid_owner || has_conditional {
            return Err("source comptime declaration changed its checked effect or owner");
        }
    }
    Ok(())
}

fn bodyless_header(
    rows: &SourceRows,
    header: &super::FunctionHeader,
    operation: BodylessIntrinsic,
) -> bool {
    let module_declaration = rows
        .declarations
        .get(header.declaration.0 as usize)
        .is_some_and(|decl| {
            decl.name.as_deref() == Some(operation.name())
                && decl.definition.parent.is_some_and(|parent| {
                    rows.declarations
                        .get(parent.0 as usize)
                        .is_some_and(|owner| owner.definition.kind == super::DefinitionKind::Module)
                })
        });
    let signature = |signature: super::super::Signature| {
        signature.convention == Convention::Rust
            && !signature.unsafe_
            && !signature.variadic
            && match operation {
                BodylessIntrinsic::PtrMetadata => metadata_signature(rows, signature, header.scope),
                BodylessIntrinsic::OffsetOf => {
                    signature.parameters.slice(&rows.roots).is_ok_and(|params| {
                        matches!(params, [first, last] if first == last
                            && rows.types.get(first.0 as usize) == Some(&Type::Scalar(Scalar::U32)))
                    }) && rows.types.get(signature.result.0 as usize)
                        == Some(&Type::Scalar(Scalar::Usize))
                }
                _ => {
                    signature.parameters.start == signature.parameters.end
                        && (operation == BodylessIntrinsic::CallerLocation
                            || rows.types.get(signature.result.0 as usize)
                                == Some(&Type::Scalar(Scalar::Usize)))
                }
            }
    };
    module_declaration
        && header.parent.is_none()
        && header.receiver.is_none()
        && header.regions.total == 0
        && header.formals.end.checked_sub(header.formals.start)
            == Some(match operation {
                BodylessIntrinsic::PtrMetadata => 1,
                BodylessIntrinsic::OffsetOf => 2,
                _ => 0,
            })
        && header.constness == Constness::Const
        && signature(header.original)
        && signature(header.callable)
}

fn metadata_signature(
    rows: &SourceRows,
    signature: super::super::Signature,
    scope: crate::support::crates::owner_catalog::DeclarationRef,
) -> bool {
    let Ok([input]) = signature.parameters.slice(&rows.roots) else {
        return false;
    };
    let Some(Type::RawPointer {
        mutable: false,
        pointee,
    }) = rows.types.get(input.0 as usize)
    else {
        return false;
    };
    [*pointee, signature.result].into_iter().enumerate().all(|(index, ty)| {
        let Some(Type::Parameter(slot)) = rows.types.get(ty.0 as usize) else {
            return false;
        };
        rows.slots.get(slot.0 as usize).is_some_and(|slot| slot.owner == scope
            && slot.index as usize == index
            && matches!(slot.kind, ParameterKind::Type { sized } if sized.is_some() == (index == 1)))
    })
}

fn bodyless_inputs(
    rows: &SourceRows,
    header: &super::FunctionHeader,
    operation: BodylessIntrinsic,
    work: &mut Work,
) -> Result<bool, &'static str> {
    let inputs = header.inputs.slice(&rows.clauses)?;
    if operation == BodylessIntrinsic::CallerLocation {
        return Ok(inputs.is_empty());
    }
    if operation == BodylessIntrinsic::PtrMetadata {
        work.charge(inputs.len())?;
        return Ok(inputs.iter().all(|input| input.owner == header.declaration)
            && inputs.iter().any(|input| {
                matches!(
                    input.requirement,
                    super::Requirement::ProjectionEqual { .. }
                )
            }));
    }
    if operation == BodylessIntrinsic::OffsetOf {
        let [input] = inputs else { return Ok(false) };
        if header.declared_inputs != 1 {
            return Ok(false);
        }
        work.charge(rows.roles.len())?;
        return Ok(sizing_input(
            rows,
            header,
            input,
            role(rows, LangItem::PointeeSized)?,
            None,
        ));
    }
    if operation == BodylessIntrinsic::FieldOffset {
        work.charge(
            inputs
                .len()
                .saturating_add(rows.roles.len().saturating_mul(2)),
        )?;
        let field = role(rows, LangItem::Field)?;
        let sized = role(rows, LangItem::Sized)?;
        return Ok(header.declared_inputs > 0
            && inputs.iter().all(|input| input.owner == header.declaration)
            && inputs
                .iter()
                .any(|input| sizing_input(rows, header, input, field, Some(sized))));
    }
    if inputs.len() != 2 || header.declared_inputs != 1 {
        return Ok(false);
    }
    work.charge(rows.roles.len().saturating_mul(3))?;
    let meta = role(rows, LangItem::MetaSized)?;
    let pointee = role(rows, LangItem::PointeeSized)?;
    let sized = role(rows, LangItem::Sized)?;
    Ok(inputs
        .iter()
        .zip([meta, pointee])
        .all(|(clause, expected)| sizing_input(rows, header, clause, expected, Some(sized))))
}

fn sizing_input(
    rows: &SourceRows,
    header: &super::FunctionHeader,
    clause: &super::Clause,
    expected: super::DeclarationAddress,
    sized: Option<super::DeclarationAddress>,
) -> bool {
    let super::Requirement::Trait {
        application,
        effect: Constness::Ordinary,
    } = clause.requirement
    else {
        return false;
    };
    let Some(application) = rows.trait_applications.get(application.0 as usize) else {
        return false;
    };
    clause.owner == header.declaration
        && clause.binder.is_none()
        && application.declaration == expected
        && application.arguments.start == application.arguments.end
        && matches!(rows.types.get(application.subject.0 as usize), Some(Type::Parameter(slot))
            if rows.slots.get(slot.0 as usize).is_some_and(|parameter| parameter.owner == header.scope
                && parameter.index == 0 && parameter.kind == ParameterKind::Type { sized }))
}
fn role(rows: &SourceRows, role: LangItem) -> Result<super::DeclarationAddress, &'static str> {
    rows.roles
        .iter()
        .find(|row| row.role == role)
        .map(|row| row.declaration)
        .ok_or("deferred recipe lacks its actual language role")
}
pub(super) fn validate(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    work: &mut Work,
) -> Result<(), &'static str> {
    for header in &rows.associated_types {
        associated(rows, headers, header, work)?;
    }
    for projection in &rows.projections {
        projection_contract(rows, headers, projection, false, work)?;
    }
    validate_operations(rows, headers, false, work)
}
pub(super) fn validate_operations(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    trait_results: bool,
    work: &mut Work,
) -> Result<(), &'static str> {
    for opaque in &rows.opaque_futures {
        opaque_contract(rows, headers, opaque, trait_results, work)?;
    }
    for operation in &rows.compiler_operations {
        operation_contract(rows, headers, operation, work)?;
    }
    opaque_declarations(rows, headers, trait_results, work)
}
pub(super) fn function_signatures(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    function: &super::FunctionHeader,
    work: &mut Work,
) -> Result<bool, &'static str> {
    let FunctionBody::CompilerOperation(reference) = function.body else {
        return Ok(false);
    };
    let operation = rows
        .compiler_operations
        .get(reference.0 as usize)
        .ok_or("unknown function compiler-operation recipe")?;
    if operation.declaration != function.declaration {
        return Err("compiler-operation signatures belong to another actual function");
    }
    operation_contract(rows, headers, operation, work)?;
    Ok(true)
}
fn operation_contract(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    operation: &super::CompilerOperation,
    work: &mut Work,
) -> Result<(), &'static str> {
    work.charge(4)?;
    let Some(HeaderRef::Function(index)) = headers.header(operation.declaration, work)? else {
        return Err("compiler operation has no actual function header");
    };
    let function = &rows.functions[index as usize];
    let CompilerOperationKind::AsyncDropInPlace { target_formal } = operation.kind;
    let own = headers
        .scopes()
        .declared_slots(function.scope, work)?
        .slice(&rows.slots)?;
    if own.len() != 2
        || target_formal != 0
        || headers.scopes().complete_slots(function.scope, work)? != 2
        || own
            .get(1)
            .is_none_or(|slot| slot.kind != ParameterKind::Region || slot.index != 0)
        || function.regions.late.slice(&rows.region_slots)?.len() != 1
        || function.regions.bounds.start != function.regions.bounds.end
    {
        return Err("compiler operation changed its exact type/lifetime input inventory");
    }
    let late = function.regions.late.slice(&rows.region_slots)?;
    if late
        .first()
        .and_then(|slot| rows.slots.get(slot.0 as usize))
        != own.get(1)
    {
        return Err("compiler operation changed its actual late lifetime owner");
    }
    let target = own
        .get(target_formal as usize)
        .filter(|slot| slot.kind == ParameterKind::Type { sized: None })
        .ok_or("async cleanup operation lost its actual unsized target formal")?;
    if role(rows, LangItem::AsyncDropInPlace)?.declaration != operation.declaration
        || operation.future != function.original.result
        || function.parent.is_some()
        || function.receiver.is_some()
        || function.constness != Constness::Ordinary
        || function.regions.total != 1
        || function.regions.captures != 0
        || function.regions.early.start != function.regions.early.end
        || operation.output.0 as usize >= rows.types.len()
        || rows.types[operation.output.0 as usize] != Type::Scalar(Scalar::Unit)
    {
        return Err("compiler operation changed its actual helper owner or Output contract");
    }
    for signature in [function.original, function.callable] {
        constructor_signature(rows, operation, target, signature)?;
    }
    let Some(Type::OpaqueFuture(reference)) = rows.types.get(operation.future.0 as usize) else {
        return Err("compiler operation has no genuine opaque future result");
    };
    let opaque = rows
        .opaque_futures
        .get(reference.0 as usize)
        .ok_or("compiler operation lost its opaque future row")?;
    if opaque.declaration != operation.declaration || opaque.output != operation.output {
        return Err("compiler operation borrowed another opaque owner or Output");
    }
    Ok(())
}

fn associated(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    header: &super::AssociatedTypeHeader,
    work: &mut Work,
) -> Result<(), &'static str> {
    let Some(HeaderRef::Trait(index)) = headers.header(header.parent, work)? else {
        return Err("associated result has no actual parent trait");
    };
    let parent = &rows.traits[index as usize];
    sizing(rows, header, work)?;
    if header.inputs.start != header.inputs.end || header.declared_inputs != 0 {
        return Err("associated own-family inputs require their complete wider profile");
    }
    if headers.scopes().complete_slots(header.scope, work)? != 1
        || headers.scopes().declared_slots(header.scope, work)?.start
            != headers.scopes().declared_slots(header.scope, work)?.end
        || parent.region_captures != 0
        || headers.scopes().complete_type_slots(parent.scope, work)? != 1
    {
        return Err("associated result generic/region family requires its wider explicit profile");
    }
    let Some(Type::Projection(reference)) = rows.types.get(header.projection.0 as usize) else {
        return Err("associated result canonical header lost its actual projection");
    };
    let projection = rows
        .projections
        .get(reference.0 as usize)
        .ok_or("associated canonical projection is unknown")?;
    if projection.declaration.artifact.0 != 0
        || projection.declaration.declaration != header.declaration
        || projection.subject != parent.self_type
    {
        return Err("associated canonical projection changed its actual member/Self identity");
    }

    Ok(())
}

pub(super) fn projection_contract(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    projection: &super::ProjectionRecipe,
    families: bool,
    work: &mut Work,
) -> Result<(), &'static str> {
    work.charge(3)?;
    if projection.declaration.artifact.0 != 0 {
        return Err("projection requires its actual authenticated local declaration");
    }
    let Some(HeaderRef::AssociatedType(index)) =
        headers.header(projection.declaration.declaration, work)?
    else {
        return Err("projection has no actual associated-type header");
    };
    let header = &rows.associated_types[index as usize];
    let application = rows
        .trait_applications
        .get(projection.trait_application.0 as usize)
        .ok_or("projection lost its actual trait application")?;
    if application.subject != projection.subject
        || application.declaration.artifact.0 != 0
        || application.declaration.declaration != header.parent
        || (!families && projection.arguments.start != projection.arguments.end)
    {
        return Err("projection changed its actual Self, parent trait or family arguments");
    }
    if families {
        let formals = headers
            .scopes()
            .declared_slots(header.scope, work)?
            .slice(&rows.slots)?;
        let arguments = projection.arguments.slice(&rows.arguments)?;
        work.charge(formals.len())?;
        if formals.len() != arguments.len()
            || formals.iter().zip(arguments).any(|(slot, argument)| {
                !matches!(
                    (slot.kind, argument),
                    (super::ParameterKind::Region, super::Argument::Region(_))
                        | (super::ParameterKind::Type { .. }, super::Argument::Type(_))
                )
            })
        {
            return Err("projection changed its actual Self, parent trait or family arguments");
        }
    }
    Ok(())
}

fn opaque_contract(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    opaque: &super::OpaqueFuture,
    trait_results: bool,
    work: &mut Work,
) -> Result<(), &'static str> {
    work.charge(3)?;
    let Some(HeaderRef::Function(index)) = headers.header(opaque.declaration, work)? else {
        return Err("opaque result has no actual function owner");
    };
    let function = &rows.functions[index as usize];
    if opaque.ordinal != 0
        || (opaque.trait_result && !trait_results)
        || opaque.future_trait != role(rows, LangItem::Future)?
        || opaque.output_member != role(rows, LangItem::FutureOutput)?
    {
        return Err("opaque result changed its actual ordinal, owner kind or Future roles");
    }
    let Some(HeaderRef::AssociatedType(member)) =
        headers.header(opaque.output_member.declaration, work)?
    else {
        return Err("opaque result lost its actual Output header");
    };
    if rows.associated_types[member as usize].parent != opaque.future_trait.declaration {
        return Err("opaque Output belongs to another actual trait");
    }
    if opaque.trait_result {
        let Some(parent) = function.parent else {
            return Err("trait async opaque result lacks its actual trait owner");
        };
        if function.body != FunctionBody::Absent
            || !matches!(headers.header(parent, work)?, Some(HeaderRef::Trait(_)))
        {
            return Err(
                "trait async opaque export requires a required trait method without a body",
            );
        }
    } else if !matches!(function.body, FunctionBody::CompilerOperation(_)) {
        return Err("authored async opaque export requires its real typed body/ownership profile");
    }
    let Some(Type::OpaqueFuture(canonical)) = rows.types.get(function.original.result.0 as usize)
    else {
        return Err("opaque owner lost its actual canonical constructor result");
    };
    let canonical = rows
        .opaque_futures
        .get(canonical.0 as usize)
        .ok_or("opaque canonical result row is unknown")?;
    if canonical.declaration != opaque.declaration
        || canonical.ordinal != opaque.ordinal
        || canonical.output != opaque.output
        || canonical.future_trait != opaque.future_trait
        || canonical.output_member != opaque.output_member
        || canonical.trait_result != opaque.trait_result
    {
        return Err(
            "opaque applications changed their one actual declaration/ordinal Output contract",
        );
    }
    opaque_captures(rows, headers, function, opaque, work)
}

fn opaque_captures(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    function: &super::FunctionHeader,
    opaque: &super::OpaqueFuture,
    work: &mut Work,
) -> Result<(), &'static str> {
    let captures = opaque.captures.slice(&rows.arguments)?;
    let slots = headers.scopes().complete_slots(function.scope, work)?;
    if captures.len() != slots as usize {
        return Err("opaque result omitted complete early/type/lifetime captures");
    }
    for argument in captures {
        match argument {
            Argument::Type(ty)
                if !matches!(rows.types.get(ty.0 as usize), Some(Type::Parameter(_))) =>
            {
                return Err("declared opaque capture is not its actual generic formal");
            }
            Argument::Region(region)
                if !match rows.regions.get(region.0 as usize) {
                    Some(Region::Bound { depth: 0, .. }) => true,
                    Some(Region::Parameter { .. }) => opaque.trait_result,
                    _ => false,
                } =>
            {
                return Err("declared opaque capture lost its exact callable lifetime frame");
            }
            Argument::Const(reference)
                if !opaque.trait_result
                    || !matches!(
                        rows.consts().get(reference.0 as usize),
                        Some(super::ConstTerm::Parameter(_))
                    ) =>
            {
                return Err("declared opaque const capture is not its actual const formal");
            }
            _ => {}
        }
    }
    Ok(())
}

fn constructor_signature(
    rows: &SourceRows,
    operation: &super::CompilerOperation,
    target: &super::ParameterSlot,
    signature: crate::support::crates::generics::Signature,
) -> Result<(), &'static str> {
    if !signature.unsafe_ || signature.variadic || signature.convention != Convention::Rust {
        return Err("compiler operation changed its actual async unsafe Rust constructor");
    }
    let Some(Type::OpaqueFuture(result)) = rows.types.get(signature.result.0 as usize) else {
        return Err("compiler operation lost its actual opaque result application");
    };
    let result = rows
        .opaque_futures
        .get(result.0 as usize)
        .ok_or("compiler operation result application is unknown")?;
    if result.declaration != operation.declaration || result.output != operation.output {
        return Err("compiler operation result belongs to another opaque owner/Output");
    }
    if rows
        .binders
        .get(signature.binder.0 as usize)
        .is_none_or(|binder| binder.count != 1 || binder.owner != operation.declaration)
    {
        return Err("compiler operation changed its actual signature binder domain");
    }
    let captures = result.captures.slice(&rows.arguments)?;
    let [
        Argument::Region(capture_region),
        Argument::Type(capture_type),
    ] = captures
    else {
        return Err("compiler operation changed its full capture ordering/kinds");
    };
    if rows.regions.get(capture_region.0 as usize)
        != Some(&Region::Bound {
            binder: signature.binder,
            depth: 0,
            slot: 0,
        })
        || !matches!(rows.types.get(capture_type.0 as usize),Some(Type::Parameter(slot)) if rows.slots.get(slot.0 as usize)==Some(target))
    {
        return Err("compiler operation changed its captured type or lifetime coordinates");
    }
    let parameters = signature.parameters.slice(&rows.roots)?;
    let [parameter] = parameters else {
        return Err("compiler operation changed its exact mutable target input count");
    };
    let Some(Type::Reference {
        mutable: true,
        region,
        referent,
    }) = rows.types.get(parameter.0 as usize)
    else {
        return Err("compiler operation requires its genuine mutable target borrow");
    };
    let Some(Type::Parameter(slot)) = rows.types.get(referent.0 as usize) else {
        return Err("compiler operation target is not its actual type formal");
    };
    if rows.slots.get(slot.0 as usize) != Some(target)
        || rows.regions.get(region.0 as usize)
            != Some(&Region::Bound {
                binder: signature.binder,
                depth: 0,
                slot: 0,
            })
    {
        return Err("compiler operation changed its formal target or borrow frame");
    }

    Ok(())
}

pub(super) fn sizing(
    rows: &SourceRows,
    header: &super::AssociatedTypeHeader,
    work: &mut Work,
) -> Result<(), &'static str> {
    let sized = role(rows, LangItem::Sized)?;
    let mut direct = false;
    for clause in header.bounds.slice(&rows.clauses)? {
        work.charge(1)?;
        if let super::Requirement::Trait {
            application,
            effect: Constness::Ordinary,
        } = clause.requirement
        {
            let application = rows
                .trait_applications
                .get(application.0 as usize)
                .ok_or("associated sizing bound lost its actual trait application")?;
            direct |= clause.binder.is_none()
                && application.subject == header.projection
                && application.declaration == sized
                && application.arguments.start == application.arguments.end;
        }
    }
    if header.sized != direct || header.copy || header.clone_ {
        return Err("associated builtin promises require their complete transitive bound profile");
    }
    Ok(())
}

fn opaque_declarations(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    declaration_policies: bool,
    work: &mut Work,
) -> Result<(), &'static str> {
    let declarations = &rows.cold().opaques.declarations;
    if declarations.is_empty() {
        return Ok(());
    }
    if !declaration_policies {
        return Err("source return-position opaque transport requires version fourteen");
    }
    let mut previous: Option<(u32, u32)> = None;
    for decl in declarations {
        work.charge(4)?;
        let expected_ordinal = match previous {
            Some((owner, ordinal)) if owner == decl.declaration.0 => ordinal
                .checked_add(1)
                .ok_or("source opaque declaration ordinal overflow")?,
            Some((owner, _)) if owner > decl.declaration.0 => {
                return Err("source opaque declarations are out of order");
            }
            _ => u32::from(
                rows.opaque_futures
                    .iter()
                    .any(|future| future.declaration == decl.declaration),
            ),
        };
        if decl.ordinal != expected_ordinal || (decl.flags.copy() && !decl.flags.clone_()) {
            return Err("source opaque declaration changed its ordinal or copy/clone contract");
        }
        previous = Some((decl.declaration.0, decl.ordinal));
        opaque_declaration_contract(rows, headers, decl, work)?;
    }
    Ok(())
}

fn opaque_declaration_contract(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    decl: &super::OpaqueDeclaration,
    work: &mut Work,
) -> Result<(), &'static str> {
    let Some(HeaderRef::Function(index)) = headers.header(decl.declaration, work)? else {
        return Err("source opaque declaration has no actual function owner");
    };
    let function = &rows.functions[index as usize];
    let is_trait_member = function.parent.is_some_and(|parent| {
        matches!(headers.header(parent, work), Ok(Some(HeaderRef::Trait(_))))
    });
    if decl.flags.trait_result() != is_trait_member || decl.bounds.start >= decl.bounds.end {
        return Err("source opaque declaration changed its trait ownership or omitted bounds");
    }
    let Some(&Type::OpaqueResult(app_ref)) = rows.types.get(decl.application.0 as usize) else {
        return Err("source opaque declaration lacks its canonical application recipe");
    };
    let app = rows
        .cold()
        .opaques
        .applications
        .get(app_ref.0 as usize)
        .ok_or("source opaque declaration application row is unknown")?;
    if app.declaration != decl.declaration || app.ordinal != decl.ordinal {
        return Err("source opaque declaration application belongs to another owner or ordinal");
    }
    opaque_declaration_captures(rows, headers, function, app.captures, work)
}

fn opaque_declaration_captures(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    function: &super::FunctionHeader,
    captures: super::super::Range,
    work: &mut Work,
) -> Result<(), &'static str> {
    let captures = captures.slice(&rows.arguments)?;
    let slots = headers.scopes().complete_slots(function.scope, work)?;
    if captures.len() > slots as usize {
        return Err("source opaque declaration captures exceed its complete scope");
    }
    let mut non_lifetime = false;
    let mut prev_region: Option<u32> = None;
    let mut seen_slots = Vec::with_capacity(captures.len());
    for argument in captures {
        work.charge(1)?;
        match *argument {
            Argument::Region(region) => {
                let Some(Region::Bound {
                    binder,
                    depth: 0,
                    slot,
                }) = rows.regions.get(region.0 as usize).copied()
                else {
                    return Err(
                        "source opaque declaration capture lost its original lifetime frame",
                    );
                };
                if non_lifetime
                    || binder != function.original.binder
                    || prev_region.is_some_and(|prior| prior >= slot)
                {
                    return Err("source opaque declaration lifetime captures are misordered");
                }
                prev_region = Some(slot);
            }
            Argument::Type(ty) => {
                non_lifetime = true;
                let Some(Type::Parameter(slot)) = rows.types.get(ty.0 as usize) else {
                    return Err("source opaque declaration capture is not a type formal");
                };
                if seen_slots.contains(&slot.0) {
                    return Err("source opaque declaration repeats a formal capture");
                }
                seen_slots.push(slot.0);
            }
            Argument::Const(reference) => {
                non_lifetime = true;
                let Some(super::ConstTerm::Parameter(slot)) =
                    rows.consts().get(reference.0 as usize)
                else {
                    return Err("source opaque declaration capture is not a const formal");
                };
                if seen_slots.contains(&slot.0) {
                    return Err("source opaque declaration repeats a formal capture");
                }
                seen_slots.push(slot.0);
            }
        }
    }
    Ok(())
}
