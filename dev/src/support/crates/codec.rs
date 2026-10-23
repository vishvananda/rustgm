//! Versioned little-endian metadata validates bounded tables before inference.
mod nominals;
mod originals;
mod profiles;
mod public_names;
mod reader;
mod scalars;
mod signatures;
mod stored_globals;
mod writer;
use super::{
    Abi, CrateIdentity, ExternalDef, ExternalKind, Function, Interface, MAX_BYTES,
    MAX_DEPENDENCIES, MAX_FUNCTIONS, Scalar, Signature, Symbol, TypeGraph,
};
use profiles::{Payload, Profile};
use reader::Reader;
use writer::Writer;
pub const SCHEMA: u16 = 1;
const MAGIC: &[u8; 8] = b"RGMETA\r\n";
// Schema 1 permanently assigns these IDs; another target/language/profile must
// receive a new ID (or schema), even when its first implementation is similar.
const TARGET_LINUX_X86_64: u8 = 0;
const LANGUAGE_RUST_1_98_EDITION_2024: u8 = 0;
const PROFILE: [u8; 3] = [TARGET_LINUX_X86_64, LANGUAGE_RUST_1_98_EDITION_2024, 0];

/// Encode the scalar Rust 1.98/2024 Linux x86-64 interface in canonical order.
/// # Errors
/// Rejects metadata that exceeds the bounded artifact size.
pub fn encode(interface: &Interface) -> Result<Vec<u8>, &'static str> {
    let mut out = Writer::default();
    out.bytes(MAGIC)?;
    out.u16(SCHEMA)?;
    out.bytes(&PROFILE[..2])?;
    let shared = interface.nominals();
    let stored = interface.storage();
    let profile = Profile::for_interface(interface);
    profile.write(&mut out)?;
    out.identity(interface.identity())?;
    out.u32(interface.dependencies().len())?;
    for dependency in interface.dependencies() {
        out.identity(dependency)?;
    }
    if let Some(records) = shared {
        nominals::encode(&mut out, records, profile)?;
    }
    out.u32(interface.functions().len())?;
    for function in interface.functions() {
        encode_function(&mut out, function, profile)?;
    }
    if shared.is_some() || profile.typed() || !interface.constants().is_empty() {
        scalars::encode(&mut out, interface.constants())?;
    }
    if let Some(storage) = stored {
        stored_globals::encode(&mut out, storage)?;
    }
    if let Some(payload) = interface.generic_payload() {
        originals::encode(&mut out, payload)?;
    }
    if let Some(names) = interface.public_names() {
        public_names::encode(
            &mut out,
            names,
            interface.identity(),
            interface.dependencies(),
        )?;
    }
    Ok(out.finish())
}
fn encode_function(
    out: &mut Writer,
    function: &Function,
    profile: Profile,
) -> Result<(), &'static str> {
    out.u32(0)?; // Owner ordinal zero is this crate, never a local DefId.
    out.bytes(&[1, 1, 1])?; // Value namespace, Function kind, Export linkage.
    out.u8(u8::from(function.signature().unsafe_()))?;
    out.bytes(&[
        u8::from(function.signature().abi() == Abi::C),
        u8::from(function.signature().variadic()),
    ])?; // Calling convention grants no const-body permission.
    out.path(function.definition().path())?;
    match function.symbol() {
        Symbol::Rust => out.u8(0)?,
        Symbol::Exact(name) => {
            out.u8(1)?;
            out.string(name)?;
        }
    }
    if profile.shared {
        out.u16(function.signature().bound_regions())?;
        signatures::encode_roots(out, function.signature())
    } else {
        signatures::encode(out, function.signature(), profile)
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
    let profile = decode_profile(&mut input)?;
    let stored = profile.payload == Payload::Storage;
    let identity = input.identity()?;
    let dependencies = decode_dependencies(&mut input)?;
    let nominal = if profile.shared {
        Some(nominals::decode(
            &mut input,
            &identity,
            dependencies.len(),
            profile,
        )?)
    } else {
        None
    };
    let count = input.count(MAX_FUNCTIONS, 21)?;
    let mut functions = Vec::with_capacity(count);
    for _ in 0..count {
        functions.push(decode_function(
            &mut input,
            &identity,
            dependencies.len(),
            profile,
            nominal.as_ref().map(super::Nominals::types),
        )?);
    }
    if functions
        .windows(2)
        .any(|pair| pair[0].definition() >= pair[1].definition())
    {
        return Err("metadata definitions are duplicate or noncanonical");
    }
    if profile.payload == Payload::Borrowed
        && functions
            .iter()
            .all(|function| function.signature().is_scalar())
    {
        return Err("borrowed metadata profile requires a typed signature");
    }
    if profile.payload == Payload::Foreign
        && functions
            .iter()
            .all(|function| function.signature().abi() == Abi::Rust)
    {
        return Err("foreign metadata profile requires a foreign calling convention");
    }
    let constants = if profile.payload == Payload::Scalar {
        Vec::new()
    } else {
        scalars::decode(
            &mut input,
            &identity,
            functions.len() + nominal.as_ref().map_or(0, |n| n.records().len()),
            profile.typed() || nominal.is_some(),
        )?
    };
    let storage = if stored {
        Some(stored_globals::decode(&mut input, &identity, profile)?)
    } else {
        None
    };
    if profile.generic() {
        let interface = decoded_base(
            identity,
            dependencies,
            functions,
            constants,
            nominal,
            storage,
        )?;
        return generic_tail(&mut input, interface, profile);
    }
    let public = if profile.public_names {
        Some(public_names::decode(&mut input, &identity, &dependencies)?)
    } else {
        None
    };
    if !input.done() {
        return Err("trailing metadata bytes");
    }
    let interface = decoded_base(
        identity,
        dependencies,
        functions,
        constants,
        nominal,
        storage,
    )?;
    profile.validate(&interface)?;
    match public {
        Some(names) => interface.with_public_names(names),
        None => Ok(interface),
    }
}
fn decoded_base(
    identity: CrateIdentity,
    dependencies: Vec<CrateIdentity>,
    functions: Vec<Function>,
    constants: Vec<super::Constant>,
    nominal: Option<super::Nominals>,
    storage: Option<super::Storage>,
) -> Result<Interface, &'static str> {
    match (nominal, storage) {
        (Some(records), Some(storage)) => Interface::with_storage(
            identity,
            dependencies,
            functions,
            constants,
            records,
            storage,
        ),
        (Some(records), None) => {
            Interface::with_nominals(identity, dependencies, functions, constants, records)
        }
        (None, None) => Interface::with_constants(identity, dependencies, functions, constants),
        (None, Some(_)) => Err("static metadata requires a shared type graph"),
    }
}
fn generic_tail(
    input: &mut Reader<'_>,
    interface: Interface,
    profile: Profile,
) -> Result<Interface, &'static str> {
    let payload = originals::decode(input, &interface)?;
    let interface = interface.with_generic_payload(payload)?;
    let public = if profile.public_names {
        Some(public_names::decode(
            input,
            interface.identity(),
            interface.dependencies(),
        )?)
    } else {
        None
    };
    if !input.done() {
        return Err("trailing metadata bytes");
    }
    profile.validate(&interface)?;
    match public {
        Some(names) => interface.with_public_names(names),
        None => Ok(interface),
    }
}
fn decode_profile(input: &mut Reader<'_>) -> Result<Profile, &'static str> {
    if input.bytes(MAGIC.len())? != MAGIC || input.u16()? != SCHEMA {
        return Err("metadata magic or schema mismatch");
    }
    if input.bytes(2)? != &PROFILE[..2] {
        return Err("metadata target, language or profile mismatch");
    }
    Profile::read(input)
}
fn decode_dependencies(input: &mut Reader<'_>) -> Result<Vec<CrateIdentity>, &'static str> {
    let count = input.count(MAX_DEPENDENCIES, 10)?;
    let mut dependencies = Vec::with_capacity(count);
    for _ in 0..count {
        dependencies.push(input.identity()?);
    }
    if dependencies.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err("metadata dependencies are not canonical");
    }
    Ok(dependencies)
}
fn decode_function(
    input: &mut Reader<'_>,
    owner: &CrateIdentity,
    dependencies: usize,
    profile: Profile,
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
    let (abi, variadic) = match input.bytes(2)? {
        [0, 0] => (Abi::Rust, false),
        [1, 0] if profile.foreign() => (Abi::C, false),
        [1, 1] if profile.foreign() && profile.variadic() => (Abi::C, true),
        _ => return Err("unsupported metadata calling convention"),
    };
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
        signatures::decode(input, profile, flags == 1)?
    };
    Function::new(
        ExternalDef::new(owner.clone(), path, ExternalKind::Function)?,
        signature.with_convention(abi, variadic)?,
        symbol,
    )
}
