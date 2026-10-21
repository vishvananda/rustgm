//! Versioned little-endian metadata validates bounded tables before inference.
mod reader;
mod writer;
use super::{
    CrateIdentity, ExternalDef, ExternalKind, Function, Interface, MAX_BYTES, MAX_DEPENDENCIES,
    MAX_FUNCTIONS, Scalar, Signature, Symbol,
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
    out.bytes(&PROFILE)?;
    out.identity(interface.identity())?;
    out.u32(interface.dependencies().len())?;
    for dependency in interface.dependencies() {
        out.identity(dependency)?;
    }
    out.u32(interface.functions().len())?;
    for function in interface.functions() {
        encode_function(&mut out, function)?;
    }
    Ok(out.finish())
}
fn encode_function(out: &mut Writer, function: &Function) -> Result<(), &'static str> {
    out.u32(0)?; // Owner ordinal zero is this crate, never a local DefId.
    out.bytes(&[1, 1, 1])?; // Value namespace, Function kind, Export linkage.
    out.u8(u8::from(function.signature().unsafe_()))?;
    out.bytes(&[0, 0])?; // Rust ABI, nonvariadic. No const-body permission bit.
    out.u16(
        u16::try_from(function.definition().path().len())
            .map_err(|_| "metadata path exceeds 16 bits")?,
    )?;
    for name in function.definition().path() {
        out.string(name)?;
    }
    match function.symbol() {
        Symbol::Rust => out.u8(0)?,
        Symbol::Exact(name) => {
            out.u8(1)?;
            out.string(name)?;
        }
    }
    out.u16(
        u16::try_from(function.signature().parameters().len())
            .map_err(|_| "metadata signature exceeds 16 bits")?,
    )?;
    for &parameter in function.signature().parameters() {
        out.u8(parameter as u8)?;
    }
    out.u8(function.signature().result() as u8)
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
    if input.bytes(3)? != PROFILE {
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
    let count = input.count(MAX_FUNCTIONS, 21)?;
    let mut functions = Vec::with_capacity(count);
    for _ in 0..count {
        functions.push(decode_function(&mut input, &identity, dependencies.len())?);
    }
    if functions
        .windows(2)
        .any(|pair| pair[0].definition() >= pair[1].definition())
    {
        return Err("metadata definitions are duplicate or noncanonical");
    }
    if !input.done() {
        return Err("trailing metadata bytes");
    }
    Interface::new(identity, dependencies, functions)
}
fn decode_function(
    input: &mut Reader<'_>,
    owner: &CrateIdentity,
    dependencies: usize,
) -> Result<Function, &'static str> {
    let ordinal = input.u32()?;
    if ordinal > dependencies {
        return Err("unknown metadata owner ID");
    }
    if ordinal != 0 {
        return Err("foreign definition in metadata export table");
    }
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
    let count = usize::from(input.u16()?);
    if count == 0 || count > 64 || count > input.remaining() / 5 {
        return Err("invalid metadata path table length");
    }
    input.atoms(count)?;
    let mut path = Vec::with_capacity(count);
    for _ in 0..count {
        path.push(input.string(256)?);
    }
    let symbol = match input.u8()? {
        0 => Symbol::Rust,
        1 => Symbol::Exact(input.string(4096)?.into()),
        _ => return Err("unknown metadata symbol policy"),
    };
    let count = usize::from(input.u16()?);
    if count > 256 || count.saturating_add(1) > input.remaining() {
        return Err("invalid metadata parameter count");
    }
    input.atoms(count + 1)?;
    let mut parameters = Vec::with_capacity(count);
    for _ in 0..count {
        parameters.push(input.scalar()?);
    }
    let signature = Signature::new(parameters, input.scalar()?, flags == 1)?;
    Function::new(
        ExternalDef::new(owner.clone(), path, ExternalKind::Function)?,
        signature,
        symbol,
    )
}
