//! Actual supported CFG operations are preserved; unsupported tables reject intact.
mod lexical_rows;
mod value_rows;
use super::{Body, Context, Declarations, Owner, Stage, g, ir, locations, s};
use locations::Sites;

pub(super) fn emit(
    stage: &mut Stage,
    facts: &mut Declarations,
    body: &Body,
    context: Context,
    variables: Vec<s::OwnedRegion>,
    sites: &mut Sites,
) -> Result<s::Original, &'static str> {
    validate(facts, body)?;
    let reference = context
        .body()
        .ok_or("source CFG has no actual original body identity")?;
    let mut raw = s::Original {
        body: reference,
        declaration: context.owner(),
        result: stage.ty(facts, body.result, context)?,
        sites: 0,
        context: call_context(facts, body)?,
        variables,
        locals: Vec::new(),
        parameter_bindings: Vec::new(),
        projections: Vec::new(),
        values: Vec::new(),
        statements: Vec::new(),
        blocks: Vec::new(),
        scopes: Vec::new(),
        exits: Vec::new(),
        temporaries: Vec::new(),
        owned: None,
        arguments: Vec::new(),
        openings: Vec::new(),
        calls: Vec::new(),
    };
    locals(stage, facts, body, context, &mut raw, sites)?;
    for projection in &body.projections {
        stage.charge(1)?;
        stage.prepare_type_arena(facts, projection.ty)?;
        let ir::ProjectionKind::Dereference { region, mutable } = projection.kind else {
            return Err(
                "source projection transport supports retained reference dereferences only",
            );
        };
        stage.reserve(4 * size_of::<s::Projection>())?;
        raw.projections.push(s::Projection {
            base: place(projection.base)?,
            ty: stage.ty(facts, projection.ty, context)?,
            region: stage.region(region, context)?,
            mutable,
        });
    }
    value_rows::emit(stage, facts, body, context, &mut raw, sites)?;
    for statement in &body.statements {
        stage.reserve(4 * size_of::<s::Statement>())?;
        raw.statements.push(s::Statement {
            site: sites.body(stage, statement.source)?,
            action: action(statement.action)?,
        });
    }
    for block in &body.blocks {
        stage.reserve(4 * size_of::<g::Block>())?;
        let terminator = match block.terminator {
            ir::Terminator::Jump(block) => g::Terminator::Jump(block.0),
            ir::Terminator::Return(value) => g::Terminator::Return(value.0),
            ir::Terminator::Unreachable => g::Terminator::Unreachable,
            _ => return Err("source CFG terminator requires a wider transport profile"),
        };
        raw.blocks.push(g::Block {
            statements: g::Range {
                start: block.start,
                end: block.end,
            },
            site: sites.body(stage, block.source)?,
            terminator,
        });
    }
    lexical_rows::emit(stage, body, &mut raw, sites)?;
    Ok(raw)
}

fn locals(
    stage: &mut Stage,
    facts: &mut Declarations,
    body: &Body,
    context: Context,
    raw: &mut s::Original,
    sites: &mut Sites,
) -> Result<(), &'static str> {
    for (index, local) in body.locals.iter().enumerate() {
        stage.charge(1)?;
        stage.prepare_type_arena(facts, local.ty)?;
        let definition = local
            .definition
            .map(|definition| stage.declaration(definition))
            .transpose()?;
        if let Some(declaration) = definition {
            stage.local_declaration(
                declaration,
                context,
                local.scope.0,
                u32::try_from(index).map_err(|_| "source local ordinal overflow")?,
            )?;
            if let ir::LocalKind::Parameter(input) = local.kind {
                stage.reserve(4 * size_of::<g::ParameterBinding>())?;
                raw.parameter_bindings.push(g::ParameterBinding {
                    declaration,
                    input,
                    local: u32::try_from(index).map_err(|_| "source formal local overflow")?,
                });
            }
        }
        let kind = match local.kind {
            ir::LocalKind::Parameter(input) => g::LocalKind::Parameter(input),
            ir::LocalKind::Binding => g::LocalKind::Binding,
            ir::LocalKind::Merge => g::LocalKind::Merge,
            ir::LocalKind::Guard(_) => {
                return Err("source guard locals require a wider transport profile");
            }
        };
        stage.reserve(4 * size_of::<g::Local>())?;
        raw.locals.push(g::Local {
            scope: local.scope.0,
            storage_parent: local.storage_parent.map(|parent| parent.0),
            ty: stage.ty(facts, local.ty, context)?,
            kind,
            definition,
            mutable: local.mutable,
            site: sites.body(stage, local.source)?,
        });
    }
    Ok(())
}

fn call_context(facts: &Declarations, body: &Body) -> Result<s::CallContext, &'static str> {
    let Owner::Function { definition, .. } = body.owner else {
        return Err("source call context has no function owner");
    };
    let source = body
        .source
        .source()
        .ok_or("source defining context has no actual semantic body")?;
    Ok(match facts.function_context(definition, source)? {
        crate::sema::function_context::FunctionContext::Runtime => s::CallContext::Runtime,
        crate::sema::function_context::FunctionContext::ConstFunction
        | crate::sema::function_context::FunctionContext::ConstTraitDefault { .. } => {
            s::CallContext::ConstDefinition
        }
    })
}
fn validate(facts: &Declarations, body: &Body) -> Result<(), &'static str> {
    let Owner::Function { definition, .. } = body.owner else {
        return Err("source CFG has no actual function owner");
    };
    if body.const_context != facts.is_const_function(definition) {
        return Err("source original changed its actual executable const context");
    }
    if !body.guards.is_empty() || !body.guard_places.is_empty() {
        return Err("source guard facts require a wider transport profile");
    }
    if !body.scalars.is_empty() {
        return Err("source scalar pool requires a wider transport profile");
    }
    if body.closure_calls.is_some() {
        return Err("source selected closure entries require a wider transport profile");
    }
    if body.resolved_callables.is_some() {
        return Err("source resolved callable identities require a wider transport profile");
    }
    // An empty completed cache has no roots to omit. Imported originals still
    // require fresh ordinary promotion analysis; no provider proof is retained.
    if body
        .promotions
        .as_ref()
        .is_some_and(|facts| !facts.is_empty())
    {
        return Err("source promoted or deferred roots require a wider transport profile");
    }
    if body
        .static_types
        .as_ref()
        .is_some_and(|values| !values.is_empty())
    {
        return Err("source static type facts require a wider transport profile");
    }
    if body
        .mutable_statics
        .as_ref()
        .is_some_and(|values| !values.is_empty())
    {
        return Err("source mutable static permissions require a wider transport profile");
    }
    Ok(())
}
pub(super) fn place(place: ir::Place) -> Result<s::Place, &'static str> {
    match place {
        ir::Place::Local(local) => Ok(s::Place::Local(local.0)),
        ir::Place::Projection(projection) => Ok(s::Place::Projection(projection.0)),
        _ => Err("source place transport supports locals and reference projections only"),
    }
}
fn action(action: ir::Action) -> Result<s::Action, &'static str> {
    Ok(match action {
        ir::Action::StorageLive(local) => s::Action::StorageLive(local.0),
        ir::Action::StorageDead(local) => s::Action::StorageDead(local.0),
        ir::Action::StorageEnd { first, last } => s::Action::StorageEnd {
            first: first.0,
            last: last.0,
        },
        ir::Action::Project(projection) => s::Action::Project(projection.0),
        ir::Action::Evaluate(value) => s::Action::Evaluate(value.0),
        ir::Action::Store {
            place: target,
            value,
        } => s::Action::Store {
            place: place(target)?,
            value: value.0,
        },
        ir::Action::CheckInitialized(target) => s::Action::CheckInitialized(place(target)?),
        ir::Action::Mention(target) => s::Action::Mention(place(target)?),
        _ => return Err("source statement requires a wider transport profile"),
    })
}
