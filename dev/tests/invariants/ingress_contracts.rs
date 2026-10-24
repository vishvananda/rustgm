//! Actual checked provider rows drive this neutral complete-input frontier.
use super::*;
use crate::support::crates::generics::{Work, source as s};
use crate::support::crates::owner_catalog::DeclarationRef;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn actual() -> Result<s::SourceRows> {
    super::super::super::tests::actual()
}
fn named(rows: &s::SourceRows, name: &str) -> Result<DeclarationRef> {
    rows.functions
        .iter()
        .find(|header| {
            rows.declarations[header.declaration.0 as usize]
                .name
                .as_deref()
                == Some(name)
        })
        .map(|header| header.declaration)
        .ok_or_else(|| "actual named source function".into())
}
fn body(rows: &s::SourceRows, name: &str) -> Result<usize> {
    let owner = named(rows, name)?;
    rows.originals
        .iter()
        .position(|original| original.declaration == owner)
        .ok_or_else(|| "actual provided source body".into())
}
#[test]
fn complete_originals_derive_requests_without_erasing_default_or_unused_formals() -> Result {
    let rows = actual()?;
    let mut work = Work::default();
    let headers = s::HeaderIndex::new(&rows, 0, &mut work)?;
    let domains = headers.domains(&mut work)?;
    let plan = domains.original_inputs(&mut work)?;
    for original in &rows.originals {
        let inputs = plan.required(original.body, &mut work)?;
        assert!(!inputs.is_empty());
        assert!(std::ptr::eq(
            plan.inputs(original.body, &mut work)?,
            std::ptr::from_ref(&rows.body_inputs[original.body.0 as usize])
        ));
        assert_eq!(
            inputs
                .iter()
                .filter(|input| matches!(input.kind(), RequiredKind::Formal { .. }))
                .count(),
            rows.body_inputs[original.body.0 as usize].parameters.len()
        );
        assert_eq!(
            inputs
                .iter()
                .filter(|input| matches!(input.kind(), RequiredKind::Call { .. }))
                .count(),
            original.calls.len()
        );
        if original.declaration == named(&rows, "clone_from")? {
            assert!(inputs.iter().any(|input| matches!(input.kind(), RequiredKind::DefiningCleanup { member, .. } if member == original.declaration)));
        }
    }
    assert!(std::ptr::eq(plan.rows(), std::ptr::from_ref(&rows)));
    assert!(plan.retained_bytes() <= plan.peak_bytes());
    assert!(plan.peak_bytes() <= crate::support::crates::MAX_BYTES);
    println!(
        "source-original-inputs {:?} retained={} peak={} plan={} request={}",
        plan.work(),
        plan.retained_bytes(),
        plan.peak_bytes(),
        size_of::<OriginalInputPlan<'_>>(),
        size_of::<RequiredInput>()
    );
    Ok(())
}
fn corrupt(rows: &mut s::SourceRows, case: usize) -> Result {
    let from = body(rows, "clone_from")?;
    match case {
        0 => {
            let signature = rows.originals[from].calls[0].signature;
            let scalar = rows
                .types
                .iter()
                .position(|ty| *ty == s::Type::Scalar(crate::support::crates::Scalar::U8))
                .ok_or("actual scalar result")?;
            let s::Type::Function(signature) = &mut rows.types[signature.0 as usize] else {
                return Err("actual Call signature".into());
            };
            signature.result = g::TypeRef(u32::try_from(scalar)?);
        }
        1 => {
            let owner = rows.originals[from].declaration;
            let late = rows
                .functions
                .iter()
                .find(|header| header.declaration == owner)
                .ok_or("actual original header")?
                .regions
                .late;
            assert_eq!(late.end - late.start, 2);
            rows.region_slots
                .swap(late.start as usize, late.start as usize + 1);
        }
        2 => {
            let obligations = &mut rows.body_inputs[from].obligations;
            let index = obligations
                .iter()
                .position(|input| matches!(input.kind, s::InputKind::Coercion { .. }))
                .ok_or("actual Constrain input")?;
            obligations.remove(index);
        }
        3 => {
            let id = body(rows, "identity")?;
            rows.body_inputs[id]
                .builtins
                .as_mut()
                .and_then(|input| input.source_copy.as_mut())
                .ok_or("actual Copy query carrier")?
                .goals
                .clear();
        }
        _ => {
            let ordinary = body(rows, "clone")?;
            let ty = g::TypeRef(u32::try_from(rows.types.len())?);
            rows.types
                .push(s::Type::Scalar(crate::support::crates::Scalar::U16));
            let value = rows.originals[ordinary]
                .values
                .iter_mut()
                .find(|value| matches!(value.kind, s::ValueKind::Read(_)))
                .ok_or("actual unused ordinary Read")?;
            value.ty = ty;
        }
    }
    Ok(())
}
#[test]
fn actual_signature_relocation_omission_and_unused_body_opponents_reject() -> Result {
    let original = actual()?;
    for case in 0..5 {
        let mut rows = original.clone();
        corrupt(&mut rows, case)?;
        let mut work = Work::default();
        let headers = s::HeaderIndex::new(&rows, 0, &mut work)?;
        let domains = headers.domains(&mut work)?;
        let before = work.steps;
        let Err(error) = domains.original_inputs(&mut work) else {
            return Err(format!("source input opponent {case} returned a plan").into());
        };
        assert!(work.steps > before);
        println!(
            "source-input-opponent={case} attempted={} retained={} peak={} published=false error={error}",
            work.steps - before,
            domains.retained_bytes(),
            domains.peak_bytes()
        );
    }
    Ok(())
}
#[test]
fn original_input_budget_failure_and_foreign_query_return_no_partial_stage() -> Result {
    let rows = actual()?;
    let mut work = Work::default();
    let headers = s::HeaderIndex::new(&rows, 0, &mut work)?;
    let domains = headers.domains(&mut work)?;
    let plan = domains.original_inputs(&mut work)?;
    let foreign =
        crate::support::crates::owner_catalog::BodyRef(u32::try_from(rows.originals.len())?);
    let before = work.steps;
    assert!(plan.required(foreign, &mut work).is_err());
    assert!(plan.inputs(foreign, &mut work).is_err());
    assert!(work.steps >= before + 2);
    work.steps = crate::support::crates::MAX_COMPONENTS;
    assert!(domains.original_inputs(&mut work).is_err());
    assert!(work.steps > crate::support::crates::MAX_COMPONENTS);
    Ok(())
}

#[test]
fn shared_recipe_depth_and_query_width_do_not_repeat_complete_chain_comparisons() -> Result {
    let rows = actual()?;
    let mut work = Work::default();
    let headers = s::HeaderIndex::new(&rows, 0, &mut work)?;
    let domains = headers.domains(&mut work)?;
    let scalar = rows
        .types
        .iter()
        .position(|ty| *ty == s::Type::Scalar(crate::support::crates::Scalar::U8))
        .ok_or("actual closed scalar root")?;
    let original = &rows.originals[body(&rows, "clone_from")?];
    let slot = original
        .variables
        .iter()
        .position(|region| matches!(region.allocation, s::RegionAllocation::Inference { .. }))
        .ok_or("actual body region allocation")?;
    let mut observed = [[0_usize; 3]; 3];
    for (d, depth) in [8, 32, 128].into_iter().enumerate() {
        for (q, queries) in [8, 32, 128].into_iter().enumerate() {
            observed[d][q] = shared_case(
                &domains,
                g::TypeRef(u32::try_from(scalar)?),
                nodes::Region::Variable {
                    body: original.body,
                    slot: u32::try_from(slot)?,
                },
                depth,
                queries,
                &mut work,
            )?;
        }
    }
    let slope = observed[0][2] - observed[0][0];
    for row in observed {
        assert_eq!(row[2] - row[0], slope);
    }
    assert_eq!(
        observed[1][2] - observed[0][2],
        observed[1][0] - observed[0][0]
    );
    assert_eq!(
        observed[2][2] - observed[1][2],
        observed[2][0] - observed[1][0]
    );
    Ok(())
}

fn shared_case(
    domains: &DomainIndex<'_, '_>,
    scalar: g::TypeRef,
    region: nodes::Region,
    depth: usize,
    queries: usize,
    work: &mut Work,
) -> Result<usize> {
    let before = work.steps;
    let mut arena = nodes::Arena::new(domains, 0, work)?;
    let key = uses::key(domains, scalar, nodes::Rewrite::Original, None, work)?;
    let leaf = arena.root(domains, key, work)?;
    let mut expected = leaf;
    let mut actual = leaf;
    for _ in 0..depth {
        expected = arena.intern(
            nodes::Node::Reference {
                mutable: false,
                region: nodes::Region::Static,
                referent: expected,
            },
            work,
        )?;
        actual = arena.intern(
            nodes::Node::Reference {
                mutable: false,
                region,
                referent: actual,
            },
            work,
        )?;
    }
    let nodes::Region::Variable { body, .. } = region else {
        return Err("actual shared query body".into());
    };
    for _ in 0..queries {
        // Structural request sharing only; no regional proof or permission.
        arena.same(
            domains,
            expected,
            actual,
            matching::Opening::Relation { body },
            work,
        )?;
    }
    assert_eq!(arena.nodes.len(), 1 + 2 * depth);
    assert_eq!(arena.matched.len(), depth);
    let bytes = arena.retained_bytes(domains, 0);
    assert!(bytes <= arena.reserved);
    assert!(arena.reserved <= arena.peak);
    let steps = work.steps - before;
    println!(
        "source-input-shared-dag depth={depth} query-width={queries} attempted={steps} nodes={} pairs={} hits={} retained={bytes} peak={}",
        arena.nodes.len(),
        arena.matched.len(),
        arena.hits,
        arena.peak
    );
    Ok(steps)
}
