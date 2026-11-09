//! A reified fn item travels as one value node over its Function operand, and
//! every consumer re-proves the item's applied signature against the pointer
//! type. Forged operands, pointer types, signatures, conventions and binders
//! are rejected by name. Validation grows by a constant per reified site.
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

const PROVIDER: &str = "pa29/tests/spec/reified-items/provider.rs";
const LOST: &str = "source reification lost its function item or pointer type";
const DIFFERS: &str = "reified function pointer differs from its item's signature";
const DOMAIN: &str = "source Function input lost its exact zero-slot Rust formation domain";
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
/// One reification: its original, its value, its operand and its pointer.
#[derive(Clone, Copy)]
struct Site {
    original: usize,
    value: usize,
    operand: usize,
    signature: g::Signature,
}
fn scalar(rows: &s::SourceRows, ty: g::TypeRef) -> Option<Scalar> {
    match rows.types.get(ty.0 as usize)? {
        s::Type::Scalar(scalar) => Some(*scalar),
        _ => None,
    }
}
/// The one reification whose pointer has these scalar inputs and result, and
/// whose pointer and item have these safeties.
fn site(
    rows: &s::SourceRows,
    (inputs, result): (&[Scalar], Scalar),
    (pointer, item): (bool, bool),
) -> Result<Site> {
    let mut found = Vec::new();
    for (original, body) in rows.originals.iter().enumerate() {
        for (value, row) in body.values.iter().enumerate() {
            let s::ValueKind::Reify(operand) = row.kind else {
                continue;
            };
            let Some(&s::Type::Function(signature)) = rows.types.get(row.ty.0 as usize) else {
                return Err("reified pointer row".into());
            };
            let s::ValueKind::Function(application) = body.values[operand as usize].kind else {
                return Err("reified operand".into());
            };
            let target = rows.function_applications[application.0 as usize]
                .declaration
                .declaration;
            let unsafe_item = rows
                .functions
                .iter()
                .any(|header| header.declaration == target && header.callable.unsafe_);
            let parameters = signature.parameters.slice(&rows.roots)?;
            if parameters
                .iter()
                .map(|&ty| scalar(rows, ty))
                .eq(inputs.iter().copied().map(Some))
                && scalar(rows, signature.result) == Some(result)
                && (signature.unsafe_, unsafe_item) == (pointer, item)
            {
                found.push(Site {
                    original,
                    value,
                    operand: operand as usize,
                    signature,
                });
            }
        }
    }
    let [site] = *found.as_slice() else {
        return Err(format!("{} matching reifications", found.len()).into());
    };
    Ok(site)
}
/// Appends a fn pointer row and types the site's reification with it.
fn retyped(rows: &mut s::SourceRows, site: Site, signature: g::Signature) -> Result {
    let ty = g::TypeRef(u32::try_from(rows.types.len())?);
    rows.types.push(s::Type::Function(signature));
    rows.originals[site.original].values[site.value].ty = ty;
    Ok(())
}
/// One forged row and the diagnostic every consumer gives it.
fn forged(
    rows: &mut s::SourceRows,
    [take, pick, raw_ptr]: [Site; 3],
    mutation: usize,
) -> Result<&'static str> {
    let values = &mut rows.originals[take.original].values;
    Ok(match mutation {
        0 => {
            values[take.operand].kind = s::ValueKind::Unit;
            LOST
        }
        1 => {
            values[take.value].ty = values[take.operand].ty;
            LOST
        }
        2 => {
            let parameters = pick.signature.parameters;
            retyped(
                rows,
                take,
                g::Signature {
                    parameters,
                    ..take.signature
                },
            )?;
            DIFFERS
        }
        3 => {
            let result = pick.signature.result;
            retyped(
                rows,
                take,
                g::Signature {
                    result,
                    ..take.signature
                },
            )?;
            DIFFERS
        }
        4 => {
            let signature = g::Signature {
                unsafe_: false,
                ..raw_ptr.signature
            };
            retyped(rows, raw_ptr, signature)?;
            DIFFERS
        }
        5 => {
            let convention = g::Convention::C;
            retyped(
                rows,
                take,
                g::Signature {
                    convention,
                    ..take.signature
                },
            )?;
            DOMAIN
        }
        _ => {
            let binder = pick.signature.binder;
            retyped(
                rows,
                take,
                g::Signature {
                    binder,
                    ..take.signature
                },
            )?;
            DOMAIN
        }
    })
}
#[test]
fn reifications_restage_and_forged_rows_reject_by_name() -> Result {
    let source = crate::test_source::fixture(PROVIDER)?;
    let interface = exported_profile(&source, SourcePublication::ModuleBodies, "reified_items")?;
    let bytes = crates::encode(&interface)?;
    let decoded = crates::decode(&bytes)?;
    assert_eq!(crates::encode(&decoded)?, bytes);
    let (rows, _, _) = raw(&decoded)?;
    let sites = [
        site(&rows, (&[Scalar::U8], Scalar::I32), (false, false))?,
        site(&rows, (&[Scalar::U16], Scalar::U16), (false, false))?,
        site(&rows, (&[Scalar::U8], Scalar::I32), (true, true))?,
    ];
    // as_unsafe adds `unsafe` to its safe item, which the honest restage admits.
    site(&rows, (&[Scalar::U8], Scalar::I32), (true, false))?;
    stage(&decoded, rows.clone(), &mut g::Work::default())??;
    for mutation in 0..7 {
        let mut forgery = rows.clone();
        let expected = forged(&mut forgery, sites, mutation)?;
        let staged = stage(&decoded, forgery, &mut g::Work::default())?;
        assert_eq!(staged.err(), Some(expected), "mutation {mutation}");
    }
    Ok(())
}
/// The frozen provider and `count` zero-padded reified pairs.
fn family(provider: &str, count: usize) -> Result<String> {
    let mut text = format!("{provider}// Generated reification scaling pairs.\n");
    for index in 0..count {
        writeln!(
            text,
            "pub fn h{index:03}(_v: u8) -> i32 {{ 42 }}\npub fn t{index:03}() -> fn(u8) -> i32 {{ h{index:03} }}"
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
        "reified_pairs",
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
fn reified_sites_cost_constant_bytes_and_work() -> Result {
    let provider = crate::test_source::fixture(PROVIDER)?;
    let mut observed = Vec::with_capacity(COUNTS.len());
    for count in COUNTS {
        observed.push(measured(&provider, count)?);
    }
    eprintln!("reified pairs {COUNTS:?}: {observed:?}");
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
