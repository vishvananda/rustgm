//! What macro metadata publishes for one local definition: its builtin
//! expander identity and the definition-site permissions a consumer re-applies.
//! Every other checked definition-site contract stays rejected by name.
use super::{Body, Definition};
use crate::{expand::builtin::BuiltinKind, support::crates::exported_macros::MacroKind};

/// A `#[rustc_builtin_macro]` definition as macro metadata spells it.
pub struct ExportedBuiltin {
    /// The registry spelling of the selected expander.
    pub identity: &'static str,
    pub kind: MacroKind,
    /// Declared derive helper attributes, sorted and unique.
    pub helpers: Vec<String>,
}

const fn wire_kind(kind: BuiltinKind) -> MacroKind {
    match kind {
        BuiltinKind::Bang => MacroKind::Bang,
        BuiltinKind::Attribute => MacroKind::Attribute,
        BuiltinKind::Derive => MacroKind::Derive,
    }
}

impl Definition {
    /// The builtin expander of a builtin definition, or `None` for any other body.
    /// # Errors
    /// Rejects a builtin whose checked header claims a different role.
    pub(crate) fn exported_builtin(&self) -> Result<Option<ExportedBuiltin>, &'static str> {
        let Body::Builtin(role) = self.body else {
            return Ok(None);
        };
        let header = self.header.as_deref();
        if header.is_some_and(|header| header.declaration().builtin != Some(role)) {
            return Err("exported builtin macro changed its claimed role");
        }
        let mut helpers = Vec::new();
        if let Some(header) = header.filter(|_| role.kind() == BuiltinKind::Derive) {
            helpers = header
                .derive_helpers(role)
                .map_err(|_| "exported builtin derive helpers are unavailable")?
                .iter()
                .map(|helper| helper.name().to_owned())
                .collect();
            helpers.sort_unstable();
            helpers.dedup();
        }
        Ok(Some(ExportedBuiltin {
            identity: role.name(),
            kind: wire_kind(role.kind()),
            helpers,
        }))
    }
    /// The definition's `allow_internal_unstable` features, sorted and unique.
    /// # Errors
    /// Edition-panic contracts and `allow_internal_unsafe` are not transported.
    pub(crate) fn exported_internal_unstable(&self) -> Result<Vec<String>, &'static str> {
        if self.edition_panic {
            return Err("edition-panic macro contracts are outside the macro metadata profile");
        }
        let Some(header) = self.header.as_deref() else {
            return Ok(Vec::new());
        };
        if header.declaration().unsafe_lint {
            return Err("allow_internal_unsafe macros are outside the macro metadata profile");
        }
        let mut features: Vec<String> = header.features().map(str::to_owned).collect();
        features.sort_unstable();
        features.dedup();
        Ok(features)
    }
}
