//! Explicit crate disambiguators and canonical paths survive every local remapping.
use std::sync::Arc;
use unicode_normalization::UnicodeNormalization;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CrateIdentity {
    name: Arc<str>,
    disambiguator: Arc<str>,
}
impl CrateIdentity {
    /// The scalar profile uses bounded ASCII crate names and explicit stable IDs.
    /// # Errors
    /// Rejects empty, oversized or noncanonical identity components.
    pub fn new(name: &str, disambiguator: &str) -> Result<Self, &'static str> {
        let first = name.as_bytes().first().copied();
        if name.is_empty()
            || name.len() > 64
            || name == "_"
            || !first.is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
            || !name
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        {
            return Err("invalid metadata crate name");
        }
        if disambiguator.is_empty()
            || disambiguator.len() > 64
            || !disambiguator
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        {
            return Err("invalid metadata crate disambiguator");
        }
        Ok(Self {
            name: Arc::from(name),
            disambiguator: Arc::from(disambiguator),
        })
    }
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    #[must_use]
    pub fn disambiguator(&self) -> &str {
        &self.disambiguator
    }
    /// ASCII symbol prefix; exact foreign spelling never enters this encoder.
    #[must_use]
    pub fn symbol_prefix(&self) -> String {
        let mut out = String::from("rgc$");
        component(&mut out, &self.name);
        component(&mut out, &self.disambiguator);
        out
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Namespace {
    Type,
    Value,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExternalKind {
    Module,
    Function,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExternalDef {
    owner: CrateIdentity,
    path: Arc<[String]>,
    kind: ExternalKind,
}
impl ExternalDef {
    /// Build a source-independent identity from its canonical semantic path.
    /// # Errors
    /// Rejects malformed or unbounded paths before a local definition is allocated.
    pub fn new(
        owner: CrateIdentity,
        path: Vec<String>,
        kind: ExternalKind,
    ) -> Result<Self, &'static str> {
        if path.len() > 64 || (path.is_empty() && kind == ExternalKind::Function) {
            return Err("invalid metadata definition path length");
        }
        for name in &path {
            validate_component(name)?;
        }
        Ok(Self {
            owner,
            path: path.into(),
            kind,
        })
    }
    #[must_use]
    pub const fn owner(&self) -> &CrateIdentity {
        &self.owner
    }
    #[must_use]
    pub fn path(&self) -> &[String] {
        &self.path
    }
    #[must_use]
    pub const fn kind(&self) -> ExternalKind {
        self.kind
    }
    #[must_use]
    pub const fn namespace(&self) -> Namespace {
        match self.kind {
            ExternalKind::Module => Namespace::Type,
            ExternalKind::Function => Namespace::Value,
        }
    }
    #[must_use]
    pub fn rust_symbol(&self) -> String {
        let mut out = self.owner.symbol_prefix();
        out.push(match self.kind {
            ExternalKind::Module => 'm',
            ExternalKind::Function => 'f',
        });
        out.push('$');
        for name in self.path.iter() {
            component(&mut out, name);
        }
        out
    }
}
pub(super) fn validate_component(name: &str) -> Result<(), &'static str> {
    let mut characters = name.chars();
    if name == "_"
        || name.len() > 256
        || !characters
            .next()
            .is_some_and(|value| value == '_' || unicode_ident::is_xid_start(value))
        || !characters.all(unicode_ident::is_xid_continue)
        || !name.chars().eq(name.nfc())
    {
        return Err("invalid canonical metadata path component");
    }
    Ok(())
}
fn component(out: &mut String, value: &str) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    out.push_str(&value.len().to_string());
    out.push('_');
    for byte in value.bytes() {
        out.push(char::from(HEX[usize::from(byte >> 4)]));
        out.push(char::from(HEX[usize::from(byte & 15)]));
    }
    out.push('$');
}
