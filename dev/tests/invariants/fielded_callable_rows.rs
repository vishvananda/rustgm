//! Under version thirteen a nominal's zero-slot fn field owns its closed
//! callable frame. Consumers form that frame in the nominal's own header
//! frame, a called constructor's payload in its own nominal's, and no call
//! signature may borrow it. Forged parents, counts, owners, signatures and
//! captured parameters are rejected by name, versions twelve and ten reject
//! the frame itself, and a fn field costs a constant over a `u8` twin.
use crate::{
    lower::instances::SourcePublication,
    module_packets::{Result, exported_profile, raw},
    sema::crates::Linkage,
    support::crates::{
        self,
        generics::{self as g, source as s},
        owner_catalog::DeclarationRef,
    },
};
use std::fmt::Write as _;

const PROVIDER: &str = "pa29/tests/spec/callable-fields/provider.rs";
const POINTERS: &str = "pa29/tests/spec/pointer-calls/provider.rs";
const NOMINALS: &str = "pa29/tests/spec/source-nominals/provider.rs";
const CLOSED: &str = "source shared call type requires an actual closed callable frame";
const DOMAIN: &str = "source Function input lost its exact zero-slot Rust formation domain";
const FOREIGN: &str = "source Call signature belongs to another actual original context";
const CAPTURED: &str = "source consuming type root captures another parameter owner";
/// The lines the older-version forgeries append to the nominals provider.
const KEEP: &str =
    "// Generated callable-field control.\npub fn keep(f: fn(u8) -> i32) -> fn(u8) -> i32 { f }\n";
const COUNTS: [usize; 3] = [8, 16, 32];
const TWINS: [usize; 3] = [1, 2, 4];
const ENTRY: &str = "#[unsafe(no_mangle)]\npub extern \"C\" fn entry() -> i32 { 42 }\n";
const HEADER: &str = "#![feature(no_core)]\n#![no_core]\nextern crate base;\n";

type Restage = fn(
    &crates::CrateIdentity,
    s::SourceRows,
    s::HeaderAttributeColumns,
    s::ModuleContractRows,
    s::AvailabilityRows,
    usize,
    &mut g::Work,
) -> std::result::Result<g::Payload, &'static str>;

/// Restages `rows` in place of the interface's own with one version's rules.
fn staged(
    interface: &crates::Interface,
    rows: s::SourceRows,
    version: Restage,
    work: &mut g::Work,
) -> Result<std::result::Result<g::Payload, &'static str>> {
    let (_, attributes, modules) = raw(interface)?;
    let availability = interface
        .source_descriptor()
        .ok_or("actual descriptor")?
        .availability()?
        .clone();
    let owner = interface.identity();
    Ok(version(
        owner,
        rows,
        attributes,
        modules,
        availability,
        0,
        work,
    ))
}
fn single<T: Copy>(found: &[T], what: &str) -> Result<T> {
    match found {
        [one] => Ok(*one),
        _ => Err(format!("{} {what}", found.len()).into()),
    }
}
/// The one declaration among `owners` named `name`.
fn named(
    rows: &s::SourceRows,
    owners: impl Iterator<Item = DeclarationRef>,
    name: &str,
) -> Result<DeclarationRef> {
    let found: Vec<DeclarationRef> = owners
        .filter(|owner| {
            rows.declarations
                .get(owner.0 as usize)
                .and_then(|row| row.name.as_deref())
                == Some(name)
        })
        .collect();
    single(&found, name)
}
fn nominal(rows: &s::SourceRows, name: &str) -> Result<DeclarationRef> {
    named(
        rows,
        rows.nominals.iter().map(|header| header.declaration),
        name,
    )
}
fn function(rows: &s::SourceRows, name: &str) -> Result<DeclarationRef> {
    named(
        rows,
        rows.functions.iter().map(|header| header.declaration),
        name,
    )
}
/// Every closed callable frame `owner` owns.
fn frames(rows: &s::SourceRows, owner: DeclarationRef) -> Vec<usize> {
    rows.binders
        .iter()
        .enumerate()
        .filter(|(_, frame)| frame.owner == owner && frame.kind == s::BinderKind::CallableType)
        .map(|(index, _)| index)
        .collect()
}
/// The provider's rows that the forgeries retarget.
#[derive(Clone, Copy)]
struct Chosen {
    slot: DeclarationRef,
    slot_frame: usize,
    table: DeclarationRef,
    tr: DeclarationRef,
    via: DeclarationRef,
    held: DeclarationRef,
    store_frame: usize,
    contra_frame: usize,
    inv: DeclarationRef,
    apply_field: usize,
}
fn chosen(rows: &s::SourceRows) -> Result<Chosen> {
    let slot = nominal(rows, "Slot")?;
    let apply = nominal(rows, "Apply")?;
    let fields: Vec<usize> = (0..rows.fields.len())
        .filter(|&index| rows.fields[index].owner == apply)
        .collect();
    let chosen = Chosen {
        slot,
        slot_frame: single(&frames(rows, slot), "Slot frames")?,
        table: nominal(rows, "Table")?,
        tr: named(
            rows,
            rows.traits.iter().map(|header| header.declaration),
            "Tr",
        )?,
        via: function(rows, "via")?,
        held: function(rows, "held")?,
        store_frame: parameter_frame(rows, function(rows, "store")?)?,
        contra_frame: single(&frames(rows, nominal(rows, "Contra")?), "Contra frames")?,
        inv: nominal(rows, "Inv")?,
        apply_field: single(&fields, "Apply fields")?,
    };
    // A parent must precede its child.
    assert!(chosen.slot_frame > 0);
    Ok(chosen)
}
/// The frame of `owner`'s one fn-pointer public parameter.
fn parameter_frame(rows: &s::SourceRows, owner: DeclarationRef) -> Result<usize> {
    let header = rows
        .functions
        .iter()
        .find(|header| header.declaration == owner)
        .ok_or("function header")?;
    let found: Vec<usize> = header
        .callable
        .parameters
        .slice(&rows.roots)?
        .iter()
        .filter_map(|ty| match rows.types.get(ty.0 as usize) {
            Some(&s::Type::Function(signature)) => Some(signature.binder.0 as usize),
            _ => None,
        })
        .collect();
    single(&found, "fn-pointer parameters")
}
fn signature_of(kind: s::ValueKind) -> Option<g::TypeRef> {
    match kind {
        s::ValueKind::Call { signature, .. } | s::ValueKind::PointerCall { signature, .. } => {
            Some(signature)
        }
        _ => None,
    }
}
/// The original of `owner` and its one direct or pointer call value.
fn call_site(rows: &s::SourceRows, owner: DeclarationRef, pointer: bool) -> Result<(usize, usize)> {
    let original = rows
        .originals
        .iter()
        .position(|body| body.declaration == owner)
        .ok_or("call original")?;
    let found: Vec<usize> = (0..rows.originals[original].values.len())
        .filter(|&value| match rows.originals[original].values[value].kind {
            s::ValueKind::PointerCall { .. } => pointer,
            s::ValueKind::Call { .. } => !pointer,
            _ => false,
        })
        .collect();
    Ok((original, single(&found, "call values")?))
}
fn pushed(rows: &mut s::SourceRows, signature: g::Signature) -> Result<g::TypeRef> {
    let ty = g::TypeRef(u32::try_from(rows.types.len())?);
    rows.types.push(s::Type::Function(signature));
    Ok(ty)
}
fn function_row(rows: &s::SourceRows, ty: g::TypeRef) -> Result<g::Signature> {
    match rows.types.get(ty.0 as usize) {
        Some(&s::Type::Function(signature)) => Ok(signature),
        _ => Err("fn pointer row".into()),
    }
}
/// Types the call at `site`, and any request for it, with `ty`.
fn resigned(rows: &mut s::SourceRows, (original, at): (usize, usize), ty: g::TypeRef) -> Result {
    let body = &mut rows.originals[original];
    match &mut body.values[at].kind {
        s::ValueKind::Call { signature, .. } | s::ValueKind::PointerCall { signature, .. } => {
            *signature = ty;
        }
        _ => return Err("call value".into()),
    }
    let value = u32::try_from(at)?;
    for request in body
        .calls
        .iter_mut()
        .filter(|request| request.value == value)
    {
        request.signature = ty;
    }
    Ok(())
}
/// The call at `site` typed by a copy of its own signature bound in `frame`.
fn rebound(rows: &mut s::SourceRows, site: (usize, usize), frame: usize) -> Result {
    let honest = signature_of(rows.originals[site.0].values[site.1].kind).ok_or("call value")?;
    let signature = g::Signature {
        binder: g::BinderRef(u32::try_from(frame)?),
        ..function_row(rows, honest)?
    };
    let ty = pushed(rows, signature)?;
    resigned(rows, site, ty)
}
/// Retypes `Apply`'s field as `fn(T) -> T` over `Contra`'s `T`, still in
/// `Apply`'s frame.
fn captured(rows: &mut s::SourceRows, chosen: Chosen) -> Result {
    let contra = rows
        .types
        .iter()
        .find_map(|row| match row {
            &s::Type::Function(signature) if signature.binder.0 as usize == chosen.contra_frame => {
                Some(signature)
            }
            _ => None,
        })
        .ok_or("Contra fn row")?;
    let [parameter] = *contra.parameters.slice(&rows.roots)? else {
        return Err("Contra fn parameters".into());
    };
    let signature = g::Signature {
        parameters: contra.parameters,
        result: parameter,
        ..function_row(rows, rows.fields[chosen.apply_field].ty)?
    };
    let ty = pushed(rows, signature)?;
    rows.fields[chosen.apply_field].ty = ty;
    Ok(())
}
/// One forged row and the diagnostic every consumer gives it.
fn forged(rows: &mut s::SourceRows, chosen: Chosen, mutation: usize) -> Result<&'static str> {
    let slot = chosen.slot_frame;
    Ok(match mutation {
        0 => {
            rows.binders[slot].parent = Some(g::BinderRef(0));
            CLOSED
        }
        1 => {
            rows.binders[slot].count = 1;
            CLOSED
        }
        2 => {
            rows.binders[slot].owner = chosen.tr;
            CLOSED
        }
        3 => {
            rows.binders[slot].owner = chosen.table;
            DOMAIN
        }
        4 => {
            rows.binders[slot].owner = chosen.via;
            DOMAIN
        }
        5 => {
            let site = call_site(rows, chosen.via, true)?;
            rebound(rows, site, slot)?;
            FOREIGN
        }
        6 => {
            let site = call_site(rows, chosen.held, false)?;
            rebound(rows, site, slot)?;
            FOREIGN
        }
        7 => {
            let site = call_site(rows, chosen.via, true)?;
            let ty = rows.fields[chosen.apply_field].ty;
            resigned(rows, site, ty)?;
            CAPTURED
        }
        // A function's parameter frame owned by a nominal.
        8 => {
            rows.binders[chosen.store_frame].owner = chosen.slot;
            DOMAIN
        }
        // A constructor's payload frame owned by another nominal.
        9 => {
            rows.binders[chosen.contra_frame].owner = chosen.inv;
            DOMAIN
        }
        _ => {
            captured(rows, chosen)?;
            CAPTURED
        }
    })
}
#[test]
fn fn_fields_restage_and_forged_frames_reject_by_name() -> Result {
    let source = crate::test_source::fixture(PROVIDER)?;
    let interface = exported_profile(&source, SourcePublication::ModuleBodies, "fn_fields")?;
    let bytes = crates::encode(&interface)?;
    let decoded = crates::decode(&bytes)?;
    assert_eq!(crates::encode(&decoded)?, bytes);
    let (rows, _, _) = raw(&decoded)?;
    let chosen = chosen(&rows)?;
    let bodies: Restage = g::Payload::from_source_module_bodies;
    staged(&decoded, rows.clone(), bodies, &mut g::Work::default())??;
    for mutation in 0..11 {
        let mut forgery = rows.clone();
        let expected = forged(&mut forgery, chosen, mutation)?;
        let result = staged(&decoded, forgery, bodies, &mut g::Work::default())?;
        assert_eq!(result.err(), Some(expected), "mutation {mutation}");
    }
    Ok(())
}
/// A function's own closed frame retargeted to a nominal: versions ten and
/// twelve reject the frame, and thirteen admits it only to fail its
/// formation in the function's header.
#[test]
fn older_versions_reject_nominal_owned_frames_by_name() -> Result {
    let source = format!("{}{KEEP}", crate::test_source::fixture(NOMINALS)?);
    let versions: [(SourcePublication, Restage, &str); 3] = [
        (
            SourcePublication::ModuleNominals,
            g::Payload::from_source_module_nominals,
            CLOSED,
        ),
        (
            SourcePublication::ModuleBranches,
            g::Payload::from_source_module_branches,
            CLOSED,
        ),
        (
            SourcePublication::ModuleBodies,
            g::Payload::from_source_module_bodies,
            DOMAIN,
        ),
    ];
    for (profile, version, expected) in versions {
        let interface = exported_profile(&source, profile, "field_older")?;
        let (mut rows, _, _) = raw(&interface)?;
        staged(&interface, rows.clone(), version, &mut g::Work::default())??;
        let packet = nominal(&rows, "Packet")?;
        let owned = frames(&rows, function(&rows, "keep")?);
        assert!(!owned.is_empty(), "{profile:?}");
        for frame in owned {
            rows.binders[frame].owner = packet;
        }
        let result = staged(&interface, rows, version, &mut g::Work::default())?;
        assert_eq!(result.err(), Some(expected), "{profile:?}");
    }
    Ok(())
}
/// Signed observations, so that an excess may be compared in either direction.
fn signed<const N: usize>(row: [usize; N]) -> Result<[i64; N]> {
    let mut out = [0; N];
    for (slot, value) in out.iter_mut().zip(row) {
        *slot = i64::try_from(value)?;
    }
    Ok(out)
}
/// Over three doubling counts, the second step is exactly twice the first
/// for `exact` metrics and at most twice it plus a constant otherwise.
fn affine<const N: usize>(label: &str, observed: [[i64; N]; 3], exact: [bool; N]) {
    let [small, middle, large] = observed;
    for (((a, b), c), exact) in small.into_iter().zip(middle).zip(large).zip(exact) {
        let (first, second) = (b - a, c - b);
        if exact {
            assert!(first > 0, "{label}: {observed:?}");
            assert_eq!(second, 2 * first, "{label}: {observed:?}");
        } else {
            assert!(second <= 2 * first + 64, "{label}: {observed:?}");
        }
    }
}
/// The callable-field provider and `count` generated structs, each with a
/// function over a reference to it that reads its field with `read`.
fn family(provider: &str, count: usize, (field, read): (&str, &str)) -> Result<String> {
    let mut text = format!("{provider}// Generated callable-field scaling items.\n");
    for index in 0..count {
        writeln!(text, "pub struct F{index:03} {{ pub f: {field} }}")?;
        writeln!(text, "pub fn v{index:03}(s: &F{index:03}) {read}")?;
    }
    Ok(text)
}
/// The packet's bytes, its restage's work, and the client's installed
/// original-header work, retained bytes, observed bytes and visits.
fn mapped(provider: &str, disambiguator: &str, client: &str) -> Result<[usize; 6]> {
    let interface = exported_profile(provider, SourcePublication::ModuleBodies, disambiguator)?;
    let bytes = crates::encode(&interface)?;
    let decoded = crates::decode(&bytes)?;
    let (rows, _, _) = raw(&decoded)?;
    let mut work = g::Work::default();
    staged(
        &decoded,
        rows,
        g::Payload::from_source_module_bodies,
        &mut work,
    )??;
    let (_, analysis, _) = crate::source_packets::client_analysis(decoded, client)?;
    let (steps, retained, observed, visits) = analysis
        .facts
        .crates
        .as_deref()
        .and_then(Linkage::original_work)
        .ok_or("installed header budget")?;
    Ok([bytes.len(), work.steps, steps, retained, observed, visits])
}
/// The fn family's packet bytes grow exactly affinely and its restage by a
/// constant per struct. An empty client's header work over it exceeds that
/// over `u8` twins, whose functions read the field, by a constant per struct
/// (outcomes amendment 2).
#[test]
fn fn_fields_cost_constant_bytes_and_work() -> Result {
    let provider = crate::test_source::fixture(PROVIDER)?;
    let client = format!("{HEADER}{ENTRY}");
    let mut observed = [[0; 2]; 3];
    let mut excess = [[0; 1]; 3];
    let mut plain = [[0; 3]; 3];
    for (((slot, extra), twin), count) in observed
        .iter_mut()
        .zip(&mut excess)
        .zip(&mut plain)
        .zip(COUNTS)
    {
        let callable = ("fn(u8) -> i32", "-> i32 { (s.f)(7) }");
        let [bytes, restage, header, ..] = signed(mapped(
            &family(&provider, count, callable)?,
            "field_scaling",
            &client,
        )?)?;
        let plain_family = family(&provider, count, ("u8", "-> u8 { s.f }"))?;
        let [twin_bytes, twin_restage, twin_header, ..] =
            signed(mapped(&plain_family, "field_scaling", &client)?)?;
        *slot = [bytes, restage];
        *extra = [header - twin_header];
        *twin = [twin_bytes, twin_restage, twin_header];
    }
    eprintln!("fn fields {COUNTS:?}: {observed:?}; client excess {excess:?}; u8 twins {plain:?}");
    affine("fn fields", observed, [true, false]);
    affine("fn-field client excess", excess, [false]);
    Ok(())
}
/// The pointer-call provider and `count` structs, each with one `field`.
fn twins(provider: &str, field: &str, count: usize) -> Result<String> {
    let mut text = format!("{provider}// Generated callable-field twins.\n");
    for index in 0..count {
        writeln!(text, "pub struct F{index:03} {{ pub f: {field} }}")?;
    }
    Ok(text)
}
/// A client that copies each twin's field out of a borrow.
fn reader(count: usize) -> Result<String> {
    let mut text = String::from(HEADER);
    for index in 0..count {
        writeln!(
            text,
            "pub fn r{index:03}(s: &base::F{index:03}) {{ let _x = s.f; }}"
        )?;
    }
    text.push_str(ENTRY);
    Ok(text)
}
/// Mapping a nominal's fn field must not demand its own owner again: the
/// client's header counters for `count` fn fields exceed those for `count`
/// `u8` twins by a constant per field, and no mapping is blocked.
#[test]
fn fn_fields_map_at_a_constant_over_u8_twins() -> Result {
    let provider = crate::test_source::fixture(POINTERS)?;
    let mut extra = [[0; 4]; 3];
    let mut plain = [[0; 6]; 3];
    for ((slot, twin), count) in extra.iter_mut().zip(&mut plain).zip(TWINS) {
        let client = reader(count)?;
        let fielded = twins(&provider, "fn(u8) -> i32", count)?;
        let [.., steps, retained, observed, visits] =
            signed(mapped(&fielded, "field_twins", &client)?)?;
        *twin = mapped(&twins(&provider, "u8", count)?, "field_twins", &client)?;
        let [.., base_steps, base_retained, base_observed, base_visits] = signed(*twin)?;
        *slot = [
            steps - base_steps,
            retained - base_retained,
            observed - base_observed,
            visits - base_visits,
        ];
    }
    eprintln!("fn-field excess over u8 twins {TWINS:?}: {extra:?}; u8 twins: {plain:?}");
    affine("fn-field excess", extra, [false; 4]);
    Ok(())
}
