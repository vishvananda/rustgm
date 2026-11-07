//! What macro metadata publishes for one local definition: its builtin
//! expander identity and the definition-site permissions a consumer re-applies.
//! An edition-panic contract is the `edition_panic` member of the allowance
//! list, exactly as the local frame derives it; `allow_internal_unsafe` is a
//! typed row flag.
use super::{Body, Definition};
use crate::{
    expand::{builtin::BuiltinKind, marks::Transparency},
    resolve::bindings::KnownFeature,
    support::crates::exported_macros::MacroKind,
};

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
    /// Rejects an edition-panic flag that its own list does not carry.
    pub(crate) fn exported_internal_unstable(&self) -> Result<Vec<String>, &'static str> {
        let mut features: Vec<String> = self
            .header
            .as_deref()
            .map(|header| header.features().map(str::to_owned).collect())
            .unwrap_or_default();
        features.sort_unstable();
        features.dedup();
        let listed = features
            .binary_search_by(|feature| {
                feature
                    .as_str()
                    .cmp(KnownFeature::EditionPanic.name().spelling())
            })
            .is_ok();
        if listed != self.edition_panic {
            return Err("exported macro edition-panic contract differs from its allowance list");
        }
        Ok(features)
    }
    /// Accept a scoped definition only with semiopaque transparency, which
    /// has `macro_rules!` hygiene; opaque and transparent ones are rejected.
    /// # Errors
    /// Names the rejected transparency, or a body that is not scoped.
    pub(crate) fn exported_semiopaque(&self) -> Result<(), &'static str> {
        match self.body {
            Body::Scoped {
                mode: Transparency::SemiOpaque,
                ..
            } => Ok(()),
            Body::Scoped {
                mode: Transparency::Transparent,
                ..
            } => Err("transparent macros 2.0 definitions are outside the macro metadata profile"),
            _ => Err("macros 2.0 definitions are outside the macro metadata profile"),
        }
    }
    /// Whether the checked header suppresses the unsafe-code lint inside this
    /// definition's own expansions.
    pub(crate) fn exported_internal_unsafe(&self) -> bool {
        self.header
            .as_deref()
            .is_some_and(|header| header.declaration().unsafe_lint)
    }
}
