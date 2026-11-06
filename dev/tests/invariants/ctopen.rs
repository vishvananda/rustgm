//! Retained item identity never authenticates a forged constructor signature.
use super::*;
use crate::{body, sema};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn original(name: &str) -> Result<(sema::model::Declarations, Body, Candidate)> {
    let text =
        crate::test_source::fixture(&format!("pa31/tests/spec/source-const-cleanup/{name}.rs"))?;
    let report = crate::test_source::source_provider(&text, "constructor_opening")?;
    let facts = sema::declarations::analyze(&report)?;
    let facts = sema::bodies::analyze(&report, facts)?;
    let function = facts
        .functions
        .iter()
        .find(|f| {
            report
                .package
                .names
                .get(report.resolution.definitions[f.definition.0 as usize].name)
                == "probe"
        })
        .ok_or("probe")?;
    let body = body::templates::Prepared::default()
        .function(&report, &facts, function)?
        .body;
    let candidate = body
        .values
        .iter()
        .enumerate()
        .find_map(|(index, value)| {
            let Rvalue::Call {
                callee, signature, ..
            } = value.kind
            else {
                return None;
            };
            let callee = body.values.get(callee.0 as usize)?.ty;
            let Kind::FunctionItem { item, .. } = facts.types.kind(callee)? else {
                return None;
            };
            let variant = facts
                .members
                .constructor(crate::resolve::model::DefId(item.0))?;
            let variant = facts.members.variants.get(variant.0 as usize)?.ordinal;
            Some(Candidate {
                value: body::ir::ValueId(u32::try_from(index).ok()?),
                variant,
                callee,
                signature,
                result: value.ty,
            })
        })
        .ok_or("actual constructor call")?;
    Ok((facts, body, candidate))
}
fn authenticate(
    facts: &mut sema::model::Declarations,
    body: &Body,
    candidate: &Candidate,
) -> Result {
    let mut resolver = Resolver::new();
    resolver.constructors(
        &mut facts.types,
        &facts.aggregates,
        &Parts::new(64_000_000),
        body,
        std::slice::from_ref(candidate),
    )?;
    Ok(())
}
#[test]
fn actual_generic_and_lifetime_constructors_open_from_their_full_fields() -> Result {
    for name in ["prknow", "taggen", "taglif"] {
        let (mut facts, body, candidate) = original(name)?;
        authenticate(&mut facts, &body, &candidate).map_err(|e| format!("{name}: {e}"))?;
    }
    Ok(())
}
#[test]
fn same_real_item_with_forged_public_or_opened_signature_is_rejected() -> Result {
    for public in [true, false] {
        let (mut facts, mut body, mut candidate) = original("prknow")?;
        let Rvalue::Call { callee, .. } = body.values[candidate.value.0 as usize].kind else {
            return Err("call".into());
        };
        let Some(Kind::FunctionItem {
            item,
            arguments,
            signature,
        }) = facts.types.kind(candidate.callee)
        else {
            return Err("item".into());
        };
        let Some(Kind::Function {
            result,
            bound_regions,
            unsafe_,
            convention,
            ..
        }) = facts.types.kind(signature)
        else {
            return Err("signature".into());
        };
        let parameters = facts.types.intern_list(&[crate::types::TypeId::UNIT])?;
        let forged = facts.types.intern(Kind::Function {
            parameters,
            result,
            bound_regions,
            unsafe_,
            convention,
        })?;
        if public {
            let selected = facts.types.intern(Kind::FunctionItem {
                item,
                arguments,
                signature: forged,
            })?;
            body.values[callee.0 as usize].ty = selected;
            candidate.callee = selected;
        } else {
            let Rvalue::Call { signature, .. } = &mut body.values[candidate.value.0 as usize].kind
            else {
                return Err("call".into());
            };
            *signature = forged;
            candidate.signature = forged;
        }
        let error = authenticate(&mut facts, &body, &candidate)
            .err()
            .ok_or("forged constructor accepted")?;
        assert!(error.to_string().contains("signature"), "{error}");
    }
    Ok(())
}
