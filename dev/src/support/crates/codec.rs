//! Versioned little-endian metadata validates bounded tables before inference.
mod nominals;
mod reader;
mod scalars;
mod signatures;
mod writer;
use super::{
    CrateIdentity, ExternalDef, ExternalKind, Function, Interface, MAX_BYTES, MAX_DEPENDENCIES,
    MAX_FUNCTIONS, Scalar, Signature, Symbol, TypeGraph,
};
use reader::Reader;
use writer::Writer;
pub const SCHEMA: u16 = 1;
const MAGIC: &[u8; 8] = b"RGMETA\r\n";
// Schema 1 permanently assigns these IDs; another target/language/profile must
// receive a new ID (or schema), even when its first implementation is similar.
const TARGET_LINUX_X86_64: u8 = 0;
const LANGUAGE_RUST_1_98_EDITION_2024: u8 = 0;
const PROFILE_SCALAR_RUST_FUNCTIONS: u8 = 0;
const PROFILE_SCALAR_RUST_FUNCTIONS_AND_VALUES: u8 = 1;
const PROFILE_BORROWED_RUST_FUNCTIONS_AND_VALUES: u8 = 2;
const PROFILE_NAMED_RECORDS: u8 = 3;
const PROFILE: [u8; 3] = [
    TARGET_LINUX_X86_64,
    LANGUAGE_RUST_1_98_EDITION_2024,
    PROFILE_SCALAR_RUST_FUNCTIONS,
];

/// Encode the scalar Rust 1.98/2024 Linux x86-64 interface in canonical order.
/// # Errors
/// Rejects metadata that exceeds the bounded artifact size.
pub fn encode(interface: &Interface) -> Result<Vec<u8>, &'static str> {
    let mut out = Writer::default();
    out.bytes(MAGIC)?;
    out.u16(SCHEMA)?;
    out.bytes(&PROFILE[..2])?;
    let shared = interface.nominals();
    let typed = shared.is_none()
        && interface
            .functions()
            .iter()
            .any(|function| !function.signature().is_scalar());
    out.u8(if shared.is_some() {
        PROFILE_NAMED_RECORDS
    } else if typed {
        PROFILE_BORROWED_RUST_FUNCTIONS_AND_VALUES
    } else if interface.constants().is_empty() {
        PROFILE_SCALAR_RUST_FUNCTIONS
    } else {
        PROFILE_SCALAR_RUST_FUNCTIONS_AND_VALUES
    })?;
    out.identity(interface.identity())?;
    out.u32(interface.dependencies().len())?;
    for dependency in interface.dependencies() {
        out.identity(dependency)?;
    }
    if let Some(records) = shared {
        nominals::encode(&mut out, records)?;
    }
    out.u32(interface.functions().len())?;
    for function in interface.functions() {
        encode_function(&mut out, function, typed, shared.is_some())?;
    }
    if shared.is_some() || typed || !interface.constants().is_empty() {
        scalars::encode(&mut out, interface.constants())?;
    }
    Ok(out.finish())
}
fn encode_function(
    out: &mut Writer,
    function: &Function,
    typed: bool,
    shared: bool,
) -> Result<(), &'static str> {
    out.u32(0)?; // Owner ordinal zero is this crate, never a local DefId.
    out.bytes(&[1, 1, 1])?; // Value namespace, Function kind, Export linkage.
    out.u8(u8::from(function.signature().unsafe_()))?;
    out.bytes(&[0, 0])?; // Rust ABI, nonvariadic. No const-body permission bit.
    out.path(function.definition().path())?;
    match function.symbol() {
        Symbol::Rust => out.u8(0)?,
        Symbol::Exact(name) => {
            out.u8(1)?;
            out.string(name)?;
        }
    }
    if shared {
        out.u16(function.signature().bound_regions())?;
        signatures::encode_roots(out, function.signature())
    } else {
        signatures::encode(out, function.signature(), typed)
    }
}
/// Decode and validate all identities, counts and profile rules before inference.
/// # Errors
/// Rejects malformed/truncated data, foreign IDs, duplicates and profile mismatches.
pub fn decode(bytes: &[u8]) -> Result<Interface, &'static str> {
    if bytes.len() > MAX_BYTES {
        return Err("metadata artifact exceeds 16 MB budget");
    }
    let mut input = Reader::new(bytes);
    if input.bytes(MAGIC.len())? != MAGIC || input.u16()? != SCHEMA {
        return Err("metadata magic or schema mismatch");
    }
    let profile = input.bytes(3)?;
    if profile[..2] != PROFILE[..2] || profile[2] > PROFILE_NAMED_RECORDS {
        return Err("metadata target, language or profile mismatch");
    }
    let identity = input.identity()?;
    let count = input.count(MAX_DEPENDENCIES, 10)?;
    let mut dependencies = Vec::with_capacity(count);
    for _ in 0..count {
        dependencies.push(input.identity()?);
    }
    if dependencies.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err("metadata dependencies are not canonical");
    }
    let nominal = if profile[2] == PROFILE_NAMED_RECORDS {
        Some(nominals::decode(&mut input, &identity, dependencies.len())?)
    } else {
        None
    };
    let count = input.count(MAX_FUNCTIONS, 21)?;
    let mut functions = Vec::with_capacity(count);
    let typed = profile[2] == PROFILE_BORROWED_RUST_FUNCTIONS_AND_VALUES;
    for _ in 0..count {
        functions.push(decode_function(
            &mut input,
            &identity,
            dependencies.len(),
            typed,
            nominal.as_ref().map(super::Nominals::types),
        )?);
    }
    if functions
        .windows(2)
        .any(|pair| pair[0].definition() >= pair[1].definition())
    {
        return Err("metadata definitions are duplicate or noncanonical");
    }
    if typed
        && functions
            .iter()
            .all(|function| function.signature().is_scalar())
    {
        return Err("borrowed metadata profile requires a typed signature");
    }
    let constants = if profile[2] == PROFILE_SCALAR_RUST_FUNCTIONS {
        Vec::new()
    } else {
        scalars::decode(
            &mut input,
            &identity,
            functions.len() + nominal.as_ref().map_or(0, |n| n.records().len()),
            typed || nominal.is_some(),
        )?
    };
    if !input.done() {
        return Err("trailing metadata bytes");
    }
    match nominal {
        Some(records) => {
            Interface::with_nominals(identity, dependencies, functions, constants, records)
        }
        None => Interface::with_constants(identity, dependencies, functions, constants),
    }
}
fn decode_function(
    input: &mut Reader<'_>,
    owner: &CrateIdentity,
    dependencies: usize,
    typed: bool,
    shared: Option<&TypeGraph>,
) -> Result<Function, &'static str> {
    input.owner(dependencies)?;
    if input.bytes(3)? != [1, 1, 1] {
        return Err("unsupported metadata definition identity or linkage");
    }
    let flags = input.u8()?;
    if flags & 2 != 0 {
        return Err("metadata const bodies are not supported by scalar profile");
    }
    if flags > 1 {
        return Err("unknown metadata function flags");
    }
    if input.bytes(2)? != [0, 0] {
        return Err("unsupported metadata calling convention");
    }
    let path = input.path()?;
    let symbol = match input.u8()? {
        0 => Symbol::Rust,
        1 => Symbol::Exact(input.string(4096)?.into()),
        _ => return Err("unknown metadata symbol policy"),
    };
    let signature = if let Some(graph) = shared {
        let bound_regions = input.u16()?;
        input.atoms(usize::from(bound_regions))?;
        let (parameters, result) = signatures::decode_roots(input)?;
        Signature::with_shared_types(graph.clone(), bound_regions, parameters, result, flags == 1)?
    } else {
        signatures::decode(input, typed, flags == 1)?
    };
    Function::new(
        ExternalDef::new(owner.clone(), path, ExternalKind::Function)?,
        signature,
        symbol,
    )
}
