//! Crate roots that share a source session with their provider.
use crate::support::edition::Edition;
use std::path::{Path, PathBuf};

/// A guest crate as the driver names it: its crate name, root source and
/// optional edition. An omitted edition inherits the host's.
#[derive(Clone, Debug)]
pub struct Guest {
    name: String,
    root: PathBuf,
    edition: Option<Edition>,
}
impl Guest {
    /// # Errors
    /// Rejects a name that is not a plain crate identifier.
    pub fn new(name: &str, root: PathBuf) -> Result<Self, &'static str> {
        let mut characters = name.chars();
        let plain = characters
            .next()
            .is_some_and(|first| first == '_' || first.is_ascii_alphabetic())
            && characters.all(|next| next == '_' || next.is_ascii_alphanumeric());
        if !plain || matches!(name, "_" | "self" | "Self" | "super" | "crate") {
            return Err("a guest crate name must be a plain identifier");
        }
        Ok(Self {
            name: name.to_owned(),
            root,
            edition: None,
        })
    }
    /// Select this crate's source edition without changing the host's inputs.
    #[must_use]
    pub const fn with_edition(mut self, edition: Edition) -> Self {
        self.edition = Some(edition);
        self
    }
    #[must_use]
    pub const fn edition(&self) -> Option<Edition> {
        self.edition
    }
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }
}
