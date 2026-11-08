//! Full declared formals and actual body absence survive canonical header staging.
use super::{Declarations, Error, Header, Headers, Parameter, error, mapped_source, s};
use crate::{
    resolve::body_origin::DeclarationOrigin,
    sema::declaration_inputs::Input,
    support::crates::owner_catalog::DeclarationRef,
    types::{arguments::Argument, parameters::environment::Environment},
};
use std::sync::Arc;

pub(in crate::sema::crates::typed_headers) struct Callable {
    pub header: Header,
    pub regions: crate::sema::lifetimes::FunctionRegions,
    pub receiver: Option<(crate::types::TypeId, u32)>,
}

pub(super) fn parameters(
    context: &mapped_source::Context<'_>,
    graph: &mapped_source::Graph,
    scope: &s::ParameterScope,
    budget: &mut Headers,
) -> Result<Vec<Parameter>, Error> {
    let rows = context.descriptor.rows();
    let slots = scope.declared.slice(&rows.slots).map_err(error)?;
    budget.reserve(slots.len().saturating_mul(size_of::<Parameter>()))?;
    let mut parameters = Vec::with_capacity(slots.len());
    for (offset, slot) in slots.iter().enumerate() {
        budget.charge(1)?;
        if slot.owner != scope.owner {
            return Err(error(
                "source generic binding belongs to another actual owner",
            ));
        }
        let Some(binding) = slot.definition else {
            if slot.kind != s::ParameterKind::Region {
                return Err(error(
                    "source type formal omitted its actual resolver definition",
                ));
            }
            continue;
        };
        let binding = context.definition(binding, budget)?;
        let ordinal = scope
            .declared
            .start
            .checked_add(
                u32::try_from(offset)
                    .map_err(|_| error("source generic binding ordinal overflow"))?,
            )
            .ok_or_else(|| error("source generic slot reference overflow"))?;
        let argument = graph.slot(s::SlotRef(ordinal))?;
        // Version thirteen: a nominal's own lifetime is its leading template
        // argument; callable early lifetimes keep their wider profile.
        if !matches!(argument, Argument::Type(_))
            && !mapped_source::slot_bindings::nominal_lifetime(&rows.declarations, slot)
        {
            return Err(error(
                "source named early lifetime requires its wider callable profile",
            ));
        }
        // Validation admitted only the owning trait's Self as a default.
        let default = slot
            .default
            .map(|ty| graph.ty(ty).map(Argument::Type))
            .transpose()?;
        parameters.push(Parameter {
            binding,
            anonymous: false,
            argument,
            default,
        });
    }
    budget.observed.vector(&parameters);
    Ok(parameters)
}

pub(super) fn header(
    context: &mapped_source::Context<'_>,
    graph: &mapped_source::Graph,
    facts: &mut Declarations,
    owner: DeclarationRef,
    inputs: &Arc<[Input]>,
    environment: &Arc<Environment>,
    budget: &mut Headers,
) -> Result<Option<Callable>, Error> {
    budget.charge(1)?;
    let rows = context.descriptor.rows();
    let Some(s::HeaderRef::Function(index)) = rows.header(owner).map_err(error)? else {
        return Ok(None);
    };
    let raw = &rows.functions[index as usize];
    let constant = raw.constness == crate::support::constness::Constness::Const;
    // Version thirteen: a validated callable role's required call method is
    // conditional; it imports as a declaration with no const eligibility.
    let conditional = raw.constness == crate::support::constness::Constness::Conditional
        && context.descriptor.profile().body_nodes()
        && raw.parent.is_some()
        && raw.body == s::FunctionBody::Absent;
    if raw.regions.captures != 0 && !super::scheme_ports::inherent_captures(context.descriptor, raw)
        || raw.regions.early.start != raw.regions.early.end
        || (raw.constness != crate::support::constness::Constness::Ordinary
            && !conditional
            && !(constant
                && context.descriptor.profile().unconditional_callables()
                && (raw.parent.is_none()
                    || super::scheme_ports::inherent_captures(context.descriptor, raw))))
    {
        return Err(error(
            "source callable captures/effects require their wider header profile",
        ));
    }
    let definition = context.definition(owner, budget)?;
    let declaration = context.catalog.declaration(definition).map_err(error)?;
    let signature = graph.signature(
        context,
        facts,
        owner,
        s::BinderKind::OriginalFunction,
        raw.original,
        budget,
    )?;
    let callable = graph.signature(
        context,
        facts,
        owner,
        s::BinderKind::PublicFunction,
        raw.callable,
        budget,
    )?;
    let body = match raw.body {
        s::FunctionBody::CompilerOperation(reference) => {
            let operation = rows
                .compiler_operations
                .get(reference.0 as usize)
                .ok_or_else(|| error("compiler operation lost its actual body recipe"))?;
            let id = context
                .imported
                .generic_body(context.descriptor.identity(), operation.body)
                .map_err(error)?;
            Some(context.catalog.origin(id).map_err(error)?)
        }
        s::FunctionBody::Absent => None,
        s::FunctionBody::Original(reference) => {
            let id = context
                .imported
                .generic_body(context.descriptor.identity(), reference)
                .map_err(error)?;
            Some(context.catalog.origin(id).map_err(error)?)
        }
    };
    if !graph
        .function(context.catalog, definition, body, raw.regions.total)
        .map_err(error)?
    {
        return Err(error(
            "source function changed its actual provider inventory",
        ));
    }
    let declared = raw.declared_inputs as usize;
    if declared > inputs.len() {
        return Err(error(
            "source declaration input prefix exceeds its actual full assumptions",
        ));
    }
    let regions = super::scheme_ports::prepare(context, graph, raw, budget)?;
    let receiver = super::scheme_ports::receiver(graph, facts, raw, signature, budget)?;
    let header = Header {
        declaration: DeclarationOrigin::Imported(declaration),
        body,
        signature,
        callable,
        environment: Some(Arc::clone(environment)),
        inputs: Arc::clone(inputs),
        declared,
        universals: raw.regions.total,
        constant,
    };
    Ok(Some(Callable {
        header,
        regions,
        receiver,
    }))
}
