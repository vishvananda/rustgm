//! The first wire profile exports closed scalar Rust functions, with explicit symbols.
use super::{
    Constant, CrateIdentity, ExternalDef, ExternalKind, MAX_COMPONENTS, MAX_DEPENDENCIES,
    MAX_FUNCTIONS, MAX_NAME_BYTES,
};
use crate::support::linkage::Linkage;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Scalar {
    Unit = 0,
    Bool = 1,
    Char = 2,
    U8 = 3,
    U16 = 4,
    U32 = 5,
    U64 = 6,
    U128 = 7,
    Usize = 8,
    I8 = 9,
    I16 = 10,
    I32 = 11,
    I64 = 12,
    I128 = 13,
    Isize = 14,
    F32 = 15,
    F64 = 16,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Symbol {
    Rust,
    Exact(Arc<str>),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Signature {
    parameters: Box<[Scalar]>,
    result: Scalar,
    unsafe_: bool,
}
impl Signature {
    /// Signatures have Rust ABI; const/generic bodies are outside this wire profile.
    /// # Errors
    /// Rejects oversized signatures and non-scalar parameters.
    pub fn new(
        parameters: Vec<Scalar>,
        result: Scalar,
        unsafe_: bool,
    ) -> Result<Self, &'static str> {
        if parameters.len() > 256 || parameters.contains(&Scalar::Unit) {
            return Err("unsupported metadata scalar parameters");
        }
        Ok(Self {
            parameters: parameters.into_boxed_slice(),
            result,
            unsafe_,
        })
    }
    #[must_use]
    pub fn parameters(&self) -> &[Scalar] {
        &self.parameters
    }
    #[must_use]
    pub const fn result(&self) -> Scalar {
        self.result
    }
    #[must_use]
    pub const fn unsafe_(&self) -> bool {
        self.unsafe_
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Function {
    definition: ExternalDef,
    signature: Signature,
    symbol: Symbol,
}
impl Function {
    /// Preserve exact symbol bytes; only Rust names use external identity encoding.
    /// # Errors
    /// Rejects non-function identities and invalid exact names.
    pub fn new(
        definition: ExternalDef,
        signature: Signature,
        symbol: Symbol,
    ) -> Result<Self, &'static str> {
        if definition.kind() != ExternalKind::Function {
            return Err("metadata entry is not a function");
        }
        if let Symbol::Exact(name) = &symbol
            && (name.is_empty() || name.len() > 4096 || name.contains('\0'))
        {
            return Err("invalid exact metadata symbol");
        }
        Ok(Self {
            definition,
            signature,
            symbol,
        })
    }
    #[must_use]
    pub const fn definition(&self) -> &ExternalDef {
        &self.definition
    }
    #[must_use]
    pub const fn signature(&self) -> &Signature {
        &self.signature
    }
    #[must_use]
    pub const fn symbol(&self) -> &Symbol {
        &self.symbol
    }
    #[must_use]
    pub const fn linkage(&self) -> Linkage {
        Linkage::Export
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct Interface {
    identity: CrateIdentity,
    dependencies: Vec<CrateIdentity>,
    functions: Vec<Function>,
    constants: Vec<Constant>,
    atoms: usize,
    names: usize,
}
impl Interface {
    /// Publish a bounded canonical table before any resolver/inference allocation.
    /// # Errors
    /// Rejects duplicates, foreign definitions and profile storage bounds.
    pub fn new(
        identity: CrateIdentity,
        dependencies: Vec<CrateIdentity>,
        functions: Vec<Function>,
    ) -> Result<Self, &'static str> {
        Self::with_constants(identity, dependencies, functions, Vec::new())
    }
    /// Add only fully validated closed values; function-only interfaces retain profile zero.
    /// # Errors
    /// Rejects duplicate value names, foreign identities and bounded table exhaustion.
    pub fn with_constants(
        identity: CrateIdentity,
        mut dependencies: Vec<CrateIdentity>,
        mut functions: Vec<Function>,
        mut constants: Vec<Constant>,
    ) -> Result<Self, &'static str> {
        if dependencies.len() > MAX_DEPENDENCIES
            || functions.len().saturating_add(constants.len()) > MAX_FUNCTIONS
        {
            return Err("metadata table exceeds profile budget");
        }
        dependencies.sort_unstable();
        if dependencies
            .iter()
            .any(|dependency| dependency == &identity)
            || dependencies.windows(2).any(|pair| pair[0] == pair[1])
        {
            return Err("duplicate or self metadata dependency");
        }
        functions.sort_unstable_by(|left, right| left.definition.cmp(&right.definition));
        let mut components = 0usize;
        let mut bytes = identity
            .name()
            .len()
            .saturating_add(identity.disambiguator().len());
        for dependency in &dependencies {
            bytes = bytes
                .saturating_add(dependency.name().len())
                .saturating_add(dependency.disambiguator().len());
        }
        for function in &functions {
            if function.definition.owner() != &identity {
                return Err("foreign definition in metadata export table");
            }
            components = components
                .saturating_add(function.definition.path().len())
                .saturating_add(function.signature.parameters.len())
                .saturating_add(1);
            bytes = bytes.saturating_add(
                function
                    .definition
                    .path()
                    .iter()
                    .map(String::len)
                    .sum::<usize>(),
            );
            if let Symbol::Exact(name) = &function.symbol {
                bytes = bytes.saturating_add(name.len());
            }
            if components > MAX_COMPONENTS || bytes > MAX_NAME_BYTES {
                return Err("metadata contents exceed profile budget");
            }
        }
        if functions
            .windows(2)
            .any(|pair| pair[0].definition == pair[1].definition)
        {
            return Err("duplicate metadata definition");
        }
        constants.sort_unstable_by(|left, right| left.definition().cmp(right.definition()));
        let (components, bytes) =
            constant_budget(&identity, &functions, &constants, components, bytes)?;
        Ok(Self {
            identity,
            dependencies,
            functions,
            constants,
            atoms: components,
            names: bytes,
        })
    }
    pub(super) const fn budget(&self) -> (usize, usize) {
        (self.atoms, self.names)
    }
    #[must_use]
    pub const fn identity(&self) -> &CrateIdentity {
        &self.identity
    }
    #[must_use]
    pub fn dependencies(&self) -> &[CrateIdentity] {
        &self.dependencies
    }
    #[must_use]
    pub fn functions(&self) -> &[Function] {
        &self.functions
    }
    #[must_use]
    pub fn constants(&self) -> &[Constant] {
        &self.constants
    }
}

fn constant_budget(
    identity: &CrateIdentity,
    functions: &[Function],
    constants: &[Constant],
    mut atoms: usize,
    mut bytes: usize,
) -> Result<(usize, usize), &'static str> {
    if constants.is_empty() {
        return Ok((atoms, bytes));
    }
    // Both kinds occupy the value namespace. Check cross-kind collisions before
    // a resolver allocates local definitions; the old function-only path allocates nothing.
    let mut names: Vec<_> = functions
        .iter()
        .map(|item| item.definition().path())
        .collect();
    for constant in constants {
        if constant.definition().owner() != identity {
            return Err("foreign definition in metadata export table");
        }
        let path = constant.definition().path();
        names.push(path);
        atoms = atoms.saturating_add(path.len()).saturating_add(1);
        bytes = bytes.saturating_add(path.iter().map(String::len).sum::<usize>());
        if atoms > MAX_COMPONENTS || bytes > MAX_NAME_BYTES {
            return Err("metadata contents exceed profile budget");
        }
    }
    names.sort_unstable();
    if names.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err("duplicate metadata value name");
    }
    Ok((atoms, bytes))
}
