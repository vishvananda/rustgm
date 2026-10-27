//! The first wire profile exports closed scalar Rust functions, with explicit symbols.
mod payload;
use super::{
    Constant, CrateIdentity, ExternalDef, ExternalKind, MAX_COMPONENTS, MAX_DEPENDENCIES,
    MAX_FUNCTIONS, MAX_NAME_BYTES, Nominals, PublicNames, Record, Signature, Storage, TypeNode,
};
use crate::support::linkage::Linkage;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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
impl Symbol {
    pub(super) fn validate(&self) -> Result<(), &'static str> {
        if let Self::Exact(name) = self
            && (name.is_empty() || name.len() > 4096 || name.contains('\0'))
        {
            return Err("invalid exact metadata symbol");
        }
        Ok(())
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
        symbol.validate()?;
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
#[derive(Debug)]
pub struct Interface {
    identity: CrateIdentity,
    dependencies: Vec<CrateIdentity>,
    functions: Vec<Function>,
    constants: Vec<Constant>,
    nominals: Option<Nominals>,
    storage: Option<Storage>,
    public_names: Option<Box<PublicNames>>,
    generic_payload: Option<Box<super::generics::Payload>>,
    atoms: usize,
    names: usize,
}
impl PartialEq for Interface {
    fn eq(&self, other: &Self) -> bool {
        // Actual validation/sorting and storage counters are paid separately;
        // they do not change metadata when construction order/capacity differs.
        self.identity == other.identity
            && self.dependencies == other.dependencies
            && self.functions == other.functions
            && self.constants == other.constants
            && self.nominals == other.nominals
            && self.storage == other.storage
            && self.public_names == other.public_names
            && self.generic_payload == other.generic_payload
    }
}
impl Eq for Interface {}
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
        dependencies: Vec<CrateIdentity>,
        functions: Vec<Function>,
        constants: Vec<Constant>,
    ) -> Result<Self, &'static str> {
        if functions.iter().any(|function| {
            function
                .signature
                .types()
                .nodes()
                .iter()
                .any(|node| matches!(node, TypeNode::Record(_)))
        }) {
            return Err("nominal metadata requires an owned record table");
        }
        for function in &functions {
            function.signature.types().canonical(
                function
                    .signature
                    .parameters()
                    .iter()
                    .copied()
                    .chain(std::iter::once(function.signature.result())),
            )?;
        }
        Self::build(identity, dependencies, functions, constants, None, None)
    }
    /// Publish shared nominal storage only after all roots and identities validate.
    /// # Errors
    /// Rejects invalid record ownership, roots, ordering and combined quotas.
    pub fn with_nominals(
        identity: CrateIdentity,
        dependencies: Vec<CrateIdentity>,
        functions: Vec<Function>,
        constants: Vec<Constant>,
        nominals: Nominals,
    ) -> Result<Self, &'static str> {
        if nominals.records().is_empty() {
            return Err("nominal metadata requires a nonempty record table");
        }
        Self::build(
            identity,
            dependencies,
            functions,
            constants,
            Some(nominals),
            None,
        )
    }
    /// Publish actual storage roots in the shared artifact graph.
    /// # Errors
    /// Rejects invalid initializers, identities, reachability and combined quotas.
    pub fn with_storage(
        identity: CrateIdentity,
        dependencies: Vec<CrateIdentity>,
        functions: Vec<Function>,
        constants: Vec<Constant>,
        nominals: Nominals,
        storage: Storage,
    ) -> Result<Self, &'static str> {
        Self::build(
            identity,
            dependencies,
            functions,
            constants,
            Some(nominals),
            Some(storage),
        )
    }
    fn build(
        identity: CrateIdentity,
        mut dependencies: Vec<CrateIdentity>,
        mut functions: Vec<Function>,
        mut constants: Vec<Constant>,
        nominals: Option<Nominals>,
        storage: Option<Storage>,
    ) -> Result<Self, &'static str> {
        if dependencies.len() > MAX_DEPENDENCIES
            || functions
                .len()
                .saturating_add(constants.len())
                .saturating_add(nominals.as_ref().map_or(0, |n| n.records().len()))
                .saturating_add(storage.as_ref().map_or(0, |s| s.statics().len()))
                > MAX_FUNCTIONS
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
                .saturating_add(if nominals.is_some() {
                    function.signature.root_atoms()
                } else {
                    function.signature.atoms()
                });
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
        let (components, bytes) = match &nominals {
            Some(records) => records.validate(
                &identity,
                &functions,
                &constants,
                storage.as_ref(),
                (components, bytes),
            )?,
            None => (components, bytes),
        };
        let (components, bytes) = if let Some(storage) = &storage {
            let records = nominals
                .as_ref()
                .ok_or("static metadata requires a shared type graph")?;
            let (atoms, names) = storage.validate(&identity, &functions, &constants, records)?;
            let components = components.saturating_add(atoms);
            let bytes = bytes.saturating_add(names);
            if components > MAX_COMPONENTS || bytes > MAX_NAME_BYTES {
                return Err("metadata contents exceed profile budget");
            }
            (components, bytes)
        } else {
            (components, bytes)
        };
        Ok(Self {
            identity,
            dependencies,
            functions,
            constants,
            nominals,
            storage,
            public_names: None,
            generic_payload: None,
            atoms: components,
            names: bytes,
        })
    }
    #[must_use]
    pub const fn nominals(&self) -> Option<&Nominals> {
        self.nominals.as_ref()
    }
    #[must_use]
    pub const fn storage(&self) -> Option<&Storage> {
        self.storage.as_ref()
    }
    /// Publish exact public bindings after owned declarations have been validated.
    /// # Errors
    /// Rejects duplicate publication, invalid targets and combined artifact quotas.
    pub fn with_public_names(mut self, public_names: PublicNames) -> Result<Self, &'static str> {
        if self.public_names.is_some() {
            return Err("public metadata names are already published");
        }
        self.check_generic_names_storage(&public_names, true)?;
        public_names.validate(&self)?;
        let (atoms, names) = public_names.budget();
        self.atoms = self.atoms.saturating_add(atoms);
        self.names = self.names.saturating_add(names);
        if self.atoms > MAX_COMPONENTS || self.names > MAX_NAME_BYTES {
            return Err("metadata contents exceed profile budget");
        }
        self.public_names = Some(Box::new(public_names));
        Ok(self)
    }
    #[must_use]
    pub fn public_names(&self) -> Option<&PublicNames> {
        self.public_names.as_deref()
    }
    #[must_use]
    pub fn records(&self) -> &[Record] {
        self.nominals.as_ref().map_or(&[], Nominals::records)
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
