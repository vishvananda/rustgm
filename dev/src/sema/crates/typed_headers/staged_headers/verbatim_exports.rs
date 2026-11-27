//! Actual catalog coordinates select exact imports without granting body readiness.
use super::{Declarations, Error, Headers, error, mapped_source, template_ports::Symbol};
use crate::{
    sema::foreign::{PreparedFunctions, Registry},
    syntax::item::Safety,
    types::Kind,
};
pub(super) fn prepare(
    context: &mapped_source::Context<'_>,
    facts: &mut Declarations,
    functions: &[super::template_ports::Callable],
    budget: &mut Headers,
) -> Result<Option<PreparedFunctions>, Error> {
    let rows = context.descriptor.rows();
    if rows.exact_callables.is_empty() {
        return Ok(None);
    }
    budget.charge(functions.len())?;
    budget.reserve(
        size_of::<crate::sema::foreign::Registry>()
            + rows.exact_callables.len().saturating_mul(size_of::<(
                crate::resolve::model::DefId,
                crate::types::TypeId,
                Safety,
                &str,
            )>()),
    )?;
    let mut inputs = Vec::with_capacity(rows.exact_callables.len());
    for function in functions {
        if !matches!(function.symbol, Symbol::Exact) {
            continue;
        }
        budget.charge(12 + rows.exact_callables.len().checked_ilog2().unwrap_or(0) as usize)?;
        let id = context
            .catalog
            .declaration(function.definition)
            .map_err(error)?;
        let (owner, declaration, _) = context.catalog.declaration_coordinate(id).map_err(error)?;
        let row = rows
            .exact_callable(declaration)
            .ok_or_else(|| error("exact import lost its actual completed definition row"))?;
        if let Some(reference) = row.body {
            let body = context.catalog.body_at(owner, reference).map_err(error)?;
            context
                .catalog
                .validate_function_origin(
                    function.definition,
                    context.catalog.origin(body).map_err(error)?,
                )
                .map_err(error)?;
        }
        if owner != context.descriptor.identity() {
            return Err(error("exact source import selected a foreign artifact"));
        }
        let Some(Kind::FunctionItem {
            item, signature, ..
        }) = facts.types.kind(function.ty)
        else {
            return Err(error("exact source import lost its actual callable"));
        };
        if item.0 != function.definition.0 {
            return Err(error(
                "exact source import changed its actual function item",
            ));
        }
        let safety = if row.body.is_some() {
            Safety::Default
        } else {
            match facts.types.kind(signature) {
                Some(Kind::Function { unsafe_: true, .. }) => Safety::Unsafe,
                Some(Kind::Function { unsafe_: false, .. }) => Safety::Safe,
                _ => {
                    return Err(error(
                        "source runtime import lost its actual callable safety",
                    ));
                }
            }
        };
        inputs.push((function.definition, signature, safety, row.symbol.as_ref()));
    }
    if inputs.len() != rows.exact_callables.len() {
        return Err(error(
            "exact import omitted its complete actual function inventory",
        ));
    }
    super::super::predicate_remap::translation::feature_names::paid(budget, |pay| {
        facts
            .foreign
            .get_or_insert_default()
            .prepare_source_functions(&inputs, pay)
    })
    .map(Some)
}

/// Keep the authenticated destination borrowed through the infallible commit.
pub(super) fn destination(
    plan: Option<PreparedFunctions>,
    registry: Option<&mut Registry>,
) -> Result<Option<(PreparedFunctions, &mut Registry)>, Error> {
    let Some(plan) = plan else {
        return Ok(None);
    };
    let registry = registry.ok_or_else(|| error("prepared exact registry disappeared"))?;
    plan.require(registry).map_err(error)?;
    Ok(Some((plan, registry)))
}
