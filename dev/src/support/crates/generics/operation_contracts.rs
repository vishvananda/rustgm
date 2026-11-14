//! Deferred recipes retain canonical contracts without publishing executable code.
use super::{
    Argument, CompilerOperationKind, FunctionBody, HeaderIndex, HeaderRef, ParameterKind, Profile,
    Region, SourceRows, Type,
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
    // Version thirteen carries declared results and their projections; only
    // version eleven carries opaque futures and symbolic operations.
    let results = profile.compiler_operations() || profile.body_nodes();
    if !profile.body_nodes() && !rows.associated_definitions().is_empty() {
        return Err(
            "associated result definitions require their explicit version-thirteen profile",
        );
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
        projection_contract(rows, headers, projection, work)?;
    }
    validate_operations(rows, headers, work)
}
pub(super) fn validate_operations(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    work: &mut Work,
) -> Result<(), &'static str> {
    for opaque in &rows.opaque_futures {
        opaque_contract(rows, headers, opaque, work)?;
    }
    for operation in &rows.compiler_operations {
        operation_contract(rows, headers, operation, work)?;
    }
    Ok(())
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
        || projection.arguments.start != projection.arguments.end
    {
        return Err("projection changed its actual Self, parent trait or family arguments");
    }

    Ok(())
}

fn opaque_contract(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    opaque: &super::OpaqueFuture,
    work: &mut Work,
) -> Result<(), &'static str> {
    work.charge(3)?;
    let Some(HeaderRef::Function(index)) = headers.header(opaque.declaration, work)? else {
        return Err("opaque result has no actual function owner");
    };
    let function = &rows.functions[index as usize];
    if opaque.ordinal != 0
        || opaque.trait_result
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
    if !matches!(function.body, FunctionBody::CompilerOperation(_)) {
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
    {
        return Err(
            "opaque applications changed their one actual declaration/ordinal Output contract",
        );
    }
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
                if !matches!(
                    rows.regions.get(region.0 as usize),
                    Some(Region::Bound { depth: 0, .. })
                ) =>
            {
                return Err("declared opaque capture lost its exact callable lifetime frame");
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
