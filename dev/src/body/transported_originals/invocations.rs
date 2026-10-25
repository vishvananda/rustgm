//! Reconstruct every call debt under the caller's actual original Environment.
use super::ImportedOriginals;
use crate::{
    body::{
        ir::{Body, Rvalue},
        regions::{Kind, Obligation},
    },
    sema::model::Declarations,
    support::crates::generics as g,
    types::{Kind as Type, parameters::predicates::Predicate},
};
pub(super) fn append(
    facts: &Declarations,
    payload: &g::Payload,
    function: &g::Function,
    body: &Body,
    obligations: &mut Vec<Obligation>,
    predicates: &mut Vec<(crate::body::ir::ProofSite, Predicate)>,
    work: &mut ImportedOriginals,
) -> Result<(), &'static str> {
    let rows = payload.legacy_rows()?;
    let Some(_) = &rows.calls else {
        return Ok(());
    };
    let original = &rows.originals[function.body.0 as usize];
    let linkage = facts
        .crates
        .as_deref()
        .ok_or("call goals have no imported linkage")?;
    let catalog = linkage
        .origin_catalog()
        .ok_or("call goals have no current owner catalog")?;
    let crate::body::ir::BodyOrigin::Imported(origin) = body.source else {
        return Err("call original has no imported body owner");
    };
    for (index, value) in original.values.iter().enumerate() {
        work.charge(1)?;
        let g::ValueKind::Call {
            signature: _,
            callee,
            arguments,
            site,
        } = value.kind
        else {
            continue;
        };
        let Some(g::Type::FunctionItem(application)) = rows
            .types
            .get(original.values[callee as usize].ty.0 as usize)
        else {
            return Err("call debt lost its exact function item");
        };
        let selected = linkage.original_application(payload.owners().owner(), *application)?;
        let Rvalue::Call { signature, .. } = body.values[index].kind else {
            return Err("imported call operation changed kind");
        };
        if body.values[callee as usize].ty != selected.item || signature != selected.signature {
            return Err("imported call changed its selected item or exact signature");
        }
        let Some(Type::Function {
            bound_regions: 0,
            parameters,
            result,
            unsafe_: false,
            ..
        }) = facts.types.kind(signature)
        else {
            return Err("call debt requires its actual safe callable");
        };
        if body.const_context && !facts.is_const_function(selected.target) {
            return Err("constant original calls a nonconstant helper");
        }
        let inputs = facts
            .types
            .list(parameters)
            .ok_or("call input list disappeared")?;
        let actuals = arguments.slice(&original.arguments)?;
        if inputs.len() != actuals.len() {
            return Err("call debt has incomplete arguments");
        }
        let source = body.source.proof_site(catalog.site(origin.id(), site)?)?;
        work.charge(
            selected
                .arguments
                .len()
                .saturating_add(selected.predicates.len())
                .saturating_add(actuals.len())
                .saturating_add(1),
        )?;
        for &ty in &selected.arguments {
            obligations.push(Obligation {
                source,
                kind: Kind::WellFormed(ty),
            });
        }
        predicates.extend(selected.predicates.iter().copied().map(|p| (source, p)));
        for (&actual, &expected) in actuals.iter().zip(inputs) {
            obligations.push(coercion(source, body.values[actual as usize].ty, expected));
        }
        obligations.push(coercion(source, result, body.values[index].ty));
    }
    Ok(())
}
fn coercion(
    source: crate::body::ir::ProofSite,
    actual: crate::types::TypeId,
    expected: crate::types::TypeId,
) -> Obligation {
    Obligation {
        source,
        kind: Kind::Coercion {
            actual,
            expected,
            unsize: false,
            reference: None,
        },
    }
}
