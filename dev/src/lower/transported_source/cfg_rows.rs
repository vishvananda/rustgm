//! Actual supported CFG operations are preserved; unsupported tables reject intact.
mod instruction_rows;
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
    let runtime_panic = if stage.declaration_policies()
        && !body.const_context
        && let Owner::Function { definition, .. } = body.owner
        && facts.is_const_function(definition)
    {
        crate::sema::compiler_items::runtime_format_entry(facts, definition)?
    } else {
        false
    };
    validate(stage, facts, body, runtime_panic)?;
    if stage.body_nodes() {
        instruction_rows::preflight(stage, facts, body)?;
    }
    let reference = context
        .body()
        .ok_or("source CFG has no actual original body identity")?;
    let call_context = if runtime_panic {
        s::CallContext::Runtime
    } else {
        call_context(facts, body, stage.declaration_policies())?
    };
    let mut raw = s::Original {
        body: reference,
        declaration: context.owner(),
        result: stage.ty(facts, body.result, context)?,
        sites: 0,
        context: call_context,
        precise_live_drops: call_context != s::CallContext::Runtime && stage.precise_live_drops(),
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
        guards: None,
        texts: None,
        assembly: None,
    };
    locals(stage, facts, body, context, &mut raw, sites)?;
    projections(stage, facts, body, context, &mut raw)?;
    value_rows::emit(stage, facts, body, context, &mut raw, sites)?;
    if stage.body_nodes() {
        instruction_rows::emit(stage, facts, (body, context), &mut raw, sites)?;
    }
    for statement in &body.statements {
        stage.reserve(4 * size_of::<s::Statement>())?;
        raw.statements.push(s::Statement {
            site: sites.body(stage, statement.source)?,
            action: action(stage, statement.action)?,
        });
    }
    guards(stage, body, &mut raw)?;
    for block in &body.blocks {
        stage.reserve(4 * size_of::<g::Block>())?;
        let terminator = terminator(
            block.terminator,
            stage.branch_transport(),
            stage.body_nodes(),
        )?;
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

fn projections(
    stage: &mut Stage,
    facts: &mut Declarations,
    body: &Body,
    context: Context,
    raw: &mut s::Original,
) -> Result<(), &'static str> {
    for projection in &body.projections {
        stage.charge(1)?;
        stage.prepare_type_arena(facts, projection.ty)?;
        let kind = match projection.kind {
            ir::ProjectionKind::FutureField(_) => {
                return Err(
                    "generated future frame fields require authenticated authored-state transport",
                );
            }
            ir::ProjectionKind::Dereference { region, mutable } => s::ProjectionKind::Dereference {
                region: stage.region(region, context)?,
                mutable,
            },
            ir::ProjectionKind::Field(crate::sema::members::Selection::Nominal(field))
                if stage.nominal_transport() =>
            {
                let identity = facts
                    .members
                    .identity(field)
                    .ok_or("source projection lost its actual nominal field identity")?;
                let enumerated = facts
                    .aggregates
                    .get(crate::types::ItemId(identity.owner.0))
                    .is_some_and(|shape| shape.kind == crate::types::aggregate::Kind::Enum);
                if enumerated && stage.body_nodes() {
                    // A downcast field names its variant; the consumer re-proves
                    // that every use is dominated by that constructor's test.
                    let variant = facts
                        .members
                        .variant(field)
                        .ok_or("source downcast field lost its actual variant")?;
                    s::ProjectionKind::Field {
                        owner: stage.variant(variant)?,
                        ordinal: identity.field,
                    }
                } else if identity.variant != 0 || enumerated {
                    return Err("source nominal field requires its wider variant profile");
                } else {
                    s::ProjectionKind::Field {
                        owner: stage.declaration(identity.owner)?,
                        ordinal: identity.field,
                    }
                }
            }
            _ => return Err("source projection transport requires its wider explicit contract"),
        };
        stage.reserve(4 * size_of::<s::Projection>())?;
        raw.projections.push(s::Projection {
            base: match projection.base {
                ir::Place::Temporary(value) if stage.conversion_roles() => {
                    s::Place::Temporary(value.0)
                }
                base => place(stage, base)?,
            },
            ty: stage.ty(facts, projection.ty, context)?,
            kind,
        });
    }
    Ok(())
}
fn terminator(
    terminator: ir::Terminator,
    branches: bool,
    guards: bool,
) -> Result<g::Terminator, &'static str> {
    Ok(match terminator {
        ir::Terminator::Jump(block) => g::Terminator::Jump(block.0),
        ir::Terminator::Return(value) => g::Terminator::Return(value.0),
        ir::Terminator::Unreachable => g::Terminator::Unreachable,
        ir::Terminator::Branch { condition, yes, no } if branches => g::Terminator::Branch {
            condition: condition.0,
            yes: yes.0,
            no: no.0,
        },
        ir::Terminator::Panic if branches => g::Terminator::Panic,
        ir::Terminator::Branch { .. } | ir::Terminator::Panic => {
            return Err("source Branch/Panic CFG exits require the version-twelve profile");
        }
        // Imaginary ownership edges belong to pattern decisions; version
        // thirteen retains them with the guard tables they protect.
        ir::Terminator::FalseEdge { real, imaginary } if guards => g::Terminator::FalseEdge {
            real: real.0,
            imaginary: imaginary.0,
        },
        ir::Terminator::FalseEdge { .. } => {
            return Err("source imaginary CFG edges require a wider transport profile");
        }
    })
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
        // A guard view previews the binding its arm later declares; that
        // binding's own local carries the declaration.
        let definition = local
            .definition
            .filter(|_| !matches!(local.kind, ir::LocalKind::Guard(_)))
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
            ir::LocalKind::Variadic => {
                return Err("source variadic locals require a wider transport profile");
            }
            ir::LocalKind::Guard(guard) if stage.body_nodes() => g::LocalKind::Guard(guard.0),
            ir::LocalKind::Guard(_) => {
                return Err("source guard locals require a wider transport profile");
            }
        };
        let ty = stage.ty(facts, local.ty, context)?;
        if value_rows::requested_formation(stage, facts, local.ty) {
            sites.function_formation(stage, body.source.root_body_site(), local.ty, ty)?;
        }
        stage.reserve(4 * size_of::<g::Local>())?;
        raw.locals.push(g::Local {
            scope: local.scope.0,
            storage_parent: local.storage_parent.map(|parent| parent.0),
            ty,
            kind,
            definition,
            mutable: local.mutable,
            site: sites.body(stage, local.source)?,
        });
    }
    Ok(())
}

/// Version thirteen retains each guard's tested places and view range as
/// recorded; every consumer re-validates them against its rebuilt body.
fn guards(stage: &mut Stage, body: &Body, raw: &mut s::Original) -> Result<(), &'static str> {
    if body.guards.is_empty() && body.guard_places.is_empty() {
        return Ok(());
    }
    stage.charge(body.guards.len().saturating_add(body.guard_places.len()))?;
    stage.reserve(
        size_of::<s::Guards>()
            .saturating_add(4 * size_of::<s::Guard>() * body.guards.len())
            .saturating_add(4 * size_of::<s::Place>() * body.guard_places.len()),
    )?;
    let rows = body
        .guards
        .iter()
        .map(|guard| s::Guard {
            tested: g::Range {
                start: guard.tested_start,
                end: guard.tested_end,
            },
            views: g::Range {
                start: guard.views_start,
                end: guard.views_end,
            },
        })
        .collect();
    let places = body
        .guard_places
        .iter()
        .map(|&tested| place(stage, tested))
        .collect::<Result<_, _>>()?;
    raw.guards = Some(Box::new(s::Guards { rows, places }));
    Ok(())
}
fn call_context(
    facts: &Declarations,
    body: &Body,
    declaration_policies: bool,
) -> Result<s::CallContext, &'static str> {
    if matches!(
        body.owner,
        Owner::Initializer {
            kind: super::InitializerKind::Const,
            ..
        }
    ) {
        return Ok(s::CallContext::ClosedConst);
    }
    let Owner::Function { definition, .. } = body.owner else {
        return Err("source call context has no function owner");
    };
    let source = body
        .source
        .source()
        .ok_or("source defining context has no actual semantic body")?;
    Ok(match facts.function_context(definition, source)? {
        crate::sema::function_context::FunctionContext::Runtime => s::CallContext::Runtime,
        crate::sema::function_context::FunctionContext::Comptime if declaration_policies => {
            s::CallContext::ConstDefinition
        }
        crate::sema::function_context::FunctionContext::Comptime => {
            return Err("always-comptime source bodies require a wider transport profile");
        }
        crate::sema::function_context::FunctionContext::ConstFunction
        | crate::sema::function_context::FunctionContext::ConstTraitDefault { .. } => {
            s::CallContext::ConstDefinition
        }
    })
}
fn validate(
    stage: &mut Stage,
    facts: &Declarations,
    body: &Body,
    runtime_panic: bool,
) -> Result<(), &'static str> {
    let scalar_values = stage.feature_predicates();
    let guards = stage.body_nodes();
    let const_context = match body.owner {
        Owner::Function { definition, .. } => facts.is_const_function(definition) && !runtime_panic,
        Owner::Initializer {
            kind: super::InitializerKind::Const,
            ..
        } if stage.declaration_policies() => true,
        _ => return Err("source CFG has no actual function or constant owner"),
    };
    if body.const_context != const_context {
        return Err("source original changed its actual executable const context");
    }
    if !guards && (!body.guards.is_empty() || !body.guard_places.is_empty()) {
        return Err("source guard facts require a wider transport profile");
    }
    if !scalar_values && !body.scalars.is_empty() {
        return Err("source scalar pool requires a wider transport profile");
    }
    if body.closure_calls.is_some() {
        return Err("source selected closure entries require a wider transport profile");
    }
    if body.resolved_callables.is_some()
        && !(stage.declaration_policies() && forwarded_location(facts, body)?)
    {
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
    if let Some(values) = body
        .static_types
        .as_ref()
        .filter(|values| !values.is_empty())
    {
        if let Some(registry) = &facts.foreign
            && values.keys().any(|&definition| {
                registry
                    .global(definition)
                    .is_some_and(|global| global.weak.is_some())
            })
        {
            return Err("selected source body needs weak foreign static storage transport");
        }
        if !stage.declaration_policies() {
            return Err("source static type facts require a wider transport profile");
        }
        for (&definition, &ty) in values.as_ref() {
            stage.static_declaration(definition)?;
            if facts
                .definitions
                .get(definition.0 as usize)
                .copied()
                .flatten()
                != Some(ty)
            {
                return Err("source static place changed its allocation type");
            }
        }
    }
    if body
        .static_permissions
        .as_ref()
        .is_some_and(|values| !values.is_empty())
    {
        return Err("source mutable static permissions require a wider transport profile");
    }
    Ok(())
}

fn forwarded_location(facts: &Declarations, body: &Body) -> Result<bool, &'static str> {
    use crate::support::bindings::SourceIntrinsic;
    let (Owner::Function { definition, .. }, Some(site)) = (body.owner, body.lone_caller_site())
    else {
        return Ok(false);
    };
    if !facts.recorded_caller_tracking(definition)? || body.caller_site(site.value)? != Some(site) {
        return Ok(false);
    }
    let Some(ir::Rvalue::Call { callee, .. }) = body
        .values
        .get(site.value.0 as usize)
        .map(|value| value.kind)
    else {
        return Ok(false);
    };
    let Some(callee) = body.values.get(callee.0 as usize) else {
        return Ok(false);
    };
    let ir::Rvalue::Function(target) = callee.kind else {
        return Ok(false);
    };
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(false);
    };
    Ok(callee.ty == site.callable
        && catalog.intrinsic_kind(target)? == Some(SourceIntrinsic::CallerLocation))
}
pub(super) fn place(stage: &mut Stage, place: ir::Place) -> Result<s::Place, &'static str> {
    match place {
        ir::Place::Local(local) => Ok(s::Place::Local(local.0)),
        ir::Place::Projection(projection) => Ok(s::Place::Projection(projection.0)),
        ir::Place::Static(definition) if stage.declaration_policies() => {
            Ok(s::Place::Static(stage.static_declaration(definition)?))
        }
        _ => Err("source place transport supports locals and reference projections only"),
    }
}
fn action(stage: &mut Stage, action: ir::Action) -> Result<s::Action, &'static str> {
    let guards = stage.body_nodes();
    Ok(match action {
        ir::Action::FutureTake { .. } => {
            return Err(
                "generated future ownership transfers require authenticated authored-state transport",
            );
        }
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
            place: place(stage, target)?,
            value: value.0,
        },
        ir::Action::CheckInitialized(target) => s::Action::CheckInitialized(place(stage, target)?),
        ir::Action::Mention(target) => s::Action::Mention(place(stage, target)?),
        ir::Action::GuardStart(guard) if guards => s::Action::GuardStart(guard.0),
        ir::Action::GuardUse(guard) if guards => s::Action::GuardUse(guard.0),
        ir::Action::BindView {
            view,
            place: target,
        } if guards => s::Action::BindView {
            view: view.0,
            place: place(stage, target)?,
        },
        ir::Action::Select { projection, anchor } if guards => s::Action::Select {
            projection: projection.0,
            anchor: place(stage, anchor)?,
        },
        _ => return Err("source statement requires a wider transport profile"),
    })
}
