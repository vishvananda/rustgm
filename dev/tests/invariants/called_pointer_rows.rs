//! A call through a closed Rust fn pointer travels as one code-30 value node
//! with its lexical safety and no request row. Every consumer re-proves its
//! signature against the callee's pointer type and its safety with a direct
//! call's matrix. Forged callees, requests, signatures, safeties, operand
//! orders and constructor pointer types are rejected by name. Validation grows
//! by a constant per call site.
use crate::{
    lower::instances::SourcePublication,
    module_packets::{Result, exported_profile, raw},
    sema::crates::Linkage,
    support::crates::{
        self, Scalar,
        generics::{self as g, source as s},
    },
};
use std::fmt::Write as _;

const PROVIDER: &str = "pa29/tests/spec/pointer-calls/provider.rs";
const LOST: &str = "source pointer Call lost its fn-pointer callee";
const DIFFERS: &str = "source pointer Call signature differs from its callee's pointer type";
const CLAIMED: &str = "source safe Call cannot claim an unsafe obligation";
const REIFIED: &str = "reified function pointer differs from its item's signature";
/// The frozen provider's pointer calls, from `pointer-call-outcomes.json`.
const CALLS: usize = 11;
const COUNTS: [usize; 3] = [8, 16, 32];
const CLIENT: &str = "#![feature(no_core)]\n#![no_core]\nextern crate base;\n#[unsafe(no_mangle)]\npub extern \"C\" fn entry() -> i32 { 42 }\n";

fn stage(
    interface: &crates::Interface,
    rows: s::SourceRows,
    work: &mut g::Work,
) -> Result<std::result::Result<g::Payload, &'static str>> {
    let descriptor = interface.source_descriptor().ok_or("actual descriptor")?;
    let (_, attributes, modules) = raw(interface)?;
    Ok(g::Payload::from_source_module_bodies(
        interface.identity(),
        rows,
        attributes,
        modules,
        descriptor.availability()?.clone(),
        0,
        work,
    ))
}
/// One pointer call: its original, its value and its operands.
#[derive(Clone, Copy)]
struct Site {
    original: usize,
    value: usize,
    signature: g::TypeRef,
    safety: s::CallSafety,
}
/// Every pointer call, in original and value order.
fn sites(rows: &s::SourceRows) -> Vec<Site> {
    let mut found = Vec::new();
    for (original, body) in rows.originals.iter().enumerate() {
        for (value, row) in body.values.iter().enumerate() {
            if let s::ValueKind::PointerCall {
                signature, safety, ..
            } = row.kind
            {
                found.push(Site {
                    original,
                    value,
                    signature,
                    safety,
                });
            }
        }
    }
    found
}
/// Whether the original's defining function is declared `unsafe`.
fn unsafe_caller(rows: &s::SourceRows, site: Site) -> bool {
    let declaration = rows.originals[site.original].declaration;
    rows.functions
        .iter()
        .any(|header| header.declaration == declaration && header.callable.unsafe_)
}
/// `reified_call`, the one original that reifies an item and calls the
/// pointer, and `block_call`, the one unsafe pointer called in an unsafe
/// block of a safe function.
fn chosen(rows: &s::SourceRows) -> Result<[Site; 2]> {
    let all = sites(rows);
    let reified: Vec<Site> = all
        .iter()
        .copied()
        .filter(|site| {
            rows.originals[site.original]
                .values
                .iter()
                .any(|row| matches!(row.kind, s::ValueKind::Reify(_)))
        })
        .collect();
    let block: Vec<Site> = all
        .iter()
        .copied()
        .filter(|&site| {
            site.safety == s::CallSafety::ExplicitUnsafeBlock && !unsafe_caller(rows, site)
        })
        .collect();
    match (reified.as_slice(), block.as_slice()) {
        ([reified], [block]) => Ok([*reified, *block]),
        _ => Err(format!("{} reified and {} block calls", reified.len(), block.len()).into()),
    }
}
/// The signature row of a call site.
fn signature(rows: &s::SourceRows, site: Site) -> Result<g::Signature> {
    match rows.types.get(site.signature.0 as usize) {
        Some(&s::Type::Function(signature)) => Ok(signature),
        _ => Err("pointer call signature row".into()),
    }
}
fn kind(rows: &mut s::SourceRows, site: Site) -> &mut s::ValueKind {
    &mut rows.originals[site.original].values[site.value].kind
}
/// Appends a fn pointer row and makes it the site's call signature.
fn resigned(rows: &mut s::SourceRows, site: Site, signature: g::Signature) -> Result {
    let ty = g::TypeRef(u32::try_from(rows.types.len())?);
    rows.types.push(s::Type::Function(signature));
    let s::ValueKind::PointerCall { signature, .. } = kind(rows, site) else {
        return Err("pointer call".into());
    };
    *signature = ty;
    Ok(())
}
fn safety(rows: &mut s::SourceRows, site: Site, safety: s::CallSafety) -> Result {
    let s::ValueKind::PointerCall { safety: slot, .. } = kind(rows, site) else {
        return Err("pointer call".into());
    };
    *slot = safety;
    Ok(())
}
fn callee(rows: &mut s::SourceRows, site: Site, value: u32) -> Result {
    let s::ValueKind::PointerCall { callee, .. } = kind(rows, site) else {
        return Err("pointer call".into());
    };
    *callee = value;
    Ok(())
}
fn scalar(rows: &s::SourceRows, ty: g::TypeRef, scalar: Scalar) -> bool {
    rows.types.get(ty.0 as usize) == Some(&s::Type::Scalar(scalar))
}
/// The `fn(u16) -> u16` pointer `pick` reifies.
fn wider(rows: &s::SourceRows) -> Result<g::Signature> {
    rows.types
        .iter()
        .find_map(|row| match row {
            &s::Type::Function(signature)
                if signature.parameters.slice(&rows.roots).is_ok_and(
                    |inputs| matches!(inputs, [input] if scalar(rows, *input, Scalar::U16)),
                ) =>
            {
                Some(signature)
            }
            _ => None,
        })
        .ok_or_else(|| "u16 pointer row".into())
}
/// An i32 type row, `take`'s result.
fn integer(rows: &s::SourceRows) -> Result<g::TypeRef> {
    let index = rows
        .types
        .iter()
        .position(|row| *row == s::Type::Scalar(Scalar::I32))
        .ok_or("i32 row")?;
    Ok(g::TypeRef(u32::try_from(index)?))
}
/// `ctor`'s reification, the one whose pointer takes a u8 and returns a
/// struct with fields: its original, its value and its pointer.
fn constructor(rows: &s::SourceRows) -> Result<(usize, usize, g::Signature)> {
    let structs: Vec<&s::Type> = rows
        .nominals
        .iter()
        .filter(|header| header.fields.end > header.fields.start)
        .filter_map(|header| rows.types.get(header.self_type.0 as usize))
        .collect();
    let mut found = Vec::new();
    for (original, body) in rows.originals.iter().enumerate() {
        for (value, row) in body.values.iter().enumerate() {
            let (s::ValueKind::Reify(_), Some(&s::Type::Function(signature))) =
                (row.kind, rows.types.get(row.ty.0 as usize))
            else {
                continue;
            };
            let input = matches!(
                signature.parameters.slice(&rows.roots)?,
                [input] if scalar(rows, *input, Scalar::U8)
            );
            let result = rows.types.get(signature.result.0 as usize);
            if input && result.is_some_and(|result| structs.contains(&result)) {
                found.push((original, value, signature));
            }
        }
    }
    let [site] = *found.as_slice() else {
        return Err(format!("{} struct constructor reifications", found.len()).into());
    };
    Ok(site)
}
/// Appends a fn pointer row and types `ctor`'s reification with it. A
/// consistent forger also records the new row's formation input, at the site
/// of the body's input for the honest pointer.
fn retyped(
    rows: &mut s::SourceRows,
    formed: bool,
    forge: impl FnOnce(&mut g::Signature),
) -> Result {
    let (original, value, mut signature) = constructor(rows)?;
    let honest = rows.originals[original].values[value].ty;
    forge(&mut signature);
    let ty = g::TypeRef(u32::try_from(rows.types.len())?);
    rows.types.push(s::Type::Function(signature));
    rows.originals[original].values[value].ty = ty;
    if formed {
        let body = rows.originals[original].body;
        let shape = rows
            .types
            .get(honest.0 as usize)
            .ok_or("honest pointer row")?;
        let inputs = rows
            .body_inputs
            .iter_mut()
            .find(|inputs| inputs.body == body)
            .ok_or("ctor body inputs")?;
        let site = inputs
            .obligations
            .iter()
            .find_map(|obligation| match obligation.kind {
                s::InputKind::WellFormed(root)
                    if rows.types.get(root.0 as usize) == Some(shape) =>
                {
                    Some(obligation.site)
                }
                _ => None,
            })
            .ok_or("ctor pointer formation input")?;
        inputs.obligations.push(s::Obligation {
            site,
            kind: s::InputKind::WellFormed(ty),
        });
    }
    Ok(())
}
/// A direct request for `reified_call`'s pointer call, as a direct call's.
fn requested(rows: &mut s::SourceRows, reified: Site) -> Result {
    let value = u32::try_from(reified.value)?;
    let body = &mut rows.originals[reified.original];
    let target = body
        .values
        .iter()
        .find_map(|row| match row.kind {
            s::ValueKind::Function(application) => Some(application),
            _ => None,
        })
        .ok_or("reified application")?;
    let site = body
        .statements
        .iter()
        .find(|statement| statement.action == s::Action::Evaluate(value))
        .ok_or("pointer call evaluation")?
        .site;
    let effect = body.context.request();
    body.calls.push(s::CallRequest {
        value,
        site,
        target,
        method: None,
        signature: reified.signature,
        opening: None,
        effect,
        safety: s::CallSafety::Safe,
    });
    Ok(())
}
/// One forged row and the diagnostic every consumer gives it.
fn forged(
    rows: &mut s::SourceRows,
    [reified, block]: [Site; 2],
    mutation: usize,
) -> Result<&'static str> {
    Ok(match mutation {
        0 => {
            let item = rows.originals[reified.original]
                .values
                .iter()
                .position(|row| matches!(row.kind, s::ValueKind::Function(_)))
                .ok_or("reified item")?;
            callee(rows, reified, u32::try_from(item)?)?;
            LOST
        }
        1 => {
            requested(rows, reified)?;
            "source request does not belong to a Call value"
        }
        2 => {
            let parameters = wider(rows)?.parameters;
            let signature = g::Signature {
                parameters,
                ..signature(rows, reified)?
            };
            resigned(rows, reified, signature)?;
            DIFFERS
        }
        3 => {
            let signature = g::Signature {
                unsafe_: false,
                ..signature(rows, block)?
            };
            resigned(rows, block, signature)?;
            DIFFERS
        }
        4 => {
            safety(rows, block, s::CallSafety::Safe)?;
            "source unsafe Call omitted its actual lexical context"
        }
        5 => {
            safety(rows, block, s::CallSafety::UnsafeFunction)?;
            "source unsafe-function Call changed its actual caller declaration"
        }
        6 => {
            safety(rows, reified, s::CallSafety::ExplicitUnsafeBlock)?;
            CLAIMED
        }
        7 => {
            safety(rows, reified, s::CallSafety::UnsafeFunction)?;
            CLAIMED
        }
        8 => {
            callee(rows, reified, u32::try_from(reified.value)?)?;
            "source value recipe is cyclic, forward or unknown"
        }
        9 => {
            let parameters = wider(rows)?.parameters;
            retyped(rows, false, |signature| signature.parameters = parameters)?;
            "source operation omitted a required nontrivial formation input"
        }
        10 => {
            let result = integer(rows)?;
            retyped(rows, false, |signature| signature.result = result)?;
            REIFIED
        }
        _ => {
            let parameters = wider(rows)?.parameters;
            retyped(rows, true, |signature| signature.parameters = parameters)?;
            REIFIED
        }
    })
}
#[test]
fn pointer_calls_restage_and_forged_rows_reject_by_name() -> Result {
    let source = crate::test_source::fixture(PROVIDER)?;
    let interface = exported_profile(&source, SourcePublication::ModuleBodies, "pointer_calls")?;
    let bytes = crates::encode(&interface)?;
    let decoded = crates::decode(&bytes)?;
    assert_eq!(crates::encode(&decoded)?, bytes);
    let (rows, _, _) = raw(&decoded)?;
    assert_eq!(sites(&rows).len(), CALLS);
    let chosen = chosen(&rows)?;
    stage(&decoded, rows.clone(), &mut g::Work::default())??;
    for mutation in 0..12 {
        let mut forgery = rows.clone();
        let expected = forged(&mut forgery, chosen, mutation)?;
        let staged = stage(&decoded, forgery, &mut g::Work::default())?;
        assert_eq!(staged.err(), Some(expected), "mutation {mutation}");
    }
    Ok(())
}
/// The frozen provider and `count` zero-padded pointer-call functions.
fn family(provider: &str, count: usize) -> Result<String> {
    let mut text = format!("{provider}// Generated pointer-call scaling functions.\n");
    for index in 0..count {
        writeln!(
            text,
            "pub fn c{index:03}(f: fn(u8) -> i32) -> i32 {{ f(7) }}"
        )?;
    }
    Ok(text)
}
/// The packet's bytes, the restage's work and an empty client's installed
/// header-budget work.
fn measured(provider: &str, count: usize) -> Result<[usize; 3]> {
    let interface = exported_profile(
        &family(provider, count)?,
        SourcePublication::ModuleBodies,
        "pointer_scaling",
    )?;
    let bytes = crates::encode(&interface)?;
    let decoded = crates::decode(&bytes)?;
    let (rows, _, _) = raw(&decoded)?;
    let mut work = g::Work::default();
    stage(&decoded, rows, &mut work)??;
    let (_, analysis, _) = crate::source_packets::client_analysis(decoded, CLIENT)?;
    let (client, _, _, _) = analysis
        .facts
        .crates
        .as_deref()
        .and_then(Linkage::original_work)
        .ok_or("installed header budget")?;
    Ok([bytes.len(), work.steps, client])
}
#[test]
fn pointer_call_sites_cost_constant_bytes_and_work() -> Result {
    let provider = crate::test_source::fixture(PROVIDER)?;
    let mut observed = Vec::with_capacity(COUNTS.len());
    for count in COUNTS {
        observed.push(measured(&provider, count)?);
    }
    eprintln!("pointer calls {COUNTS:?}: {observed:?}");
    let [small, middle, large] = <[[usize; 3]; 3]>::try_from(observed).map_err(|_| "counts")?;
    for (metric, label) in [(0, "packet bytes"), (1, "restage work"), (2, "client work")] {
        let first = middle[metric] - small[metric];
        let second = large[metric] - middle[metric];
        assert!(first > 0, "{label} did not grow: {small:?} {middle:?}");
        if metric == 0 {
            assert_eq!(second, 2 * first, "{label}: {small:?} {middle:?} {large:?}");
        } else {
            assert!(
                second <= 2 * first + 64,
                "{label}: {small:?} {middle:?} {large:?}"
            );
        }
    }
    Ok(())
}
