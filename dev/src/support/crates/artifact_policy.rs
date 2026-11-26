//! Output policy is an explicit caller input, separate from source metadata.

/// The existing `LowIR` driver's two output selections, not rustc file formats.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputKind {
    Binary,
    Library,
}

/// An inspection checks declarations without requesting an output artifact.
/// Source compilation retains an omitted selection separately from an override.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ArtifactContext {
    #[default]
    Inspection,
    /// The adopted ordinary executable profile admits no source artifact selector.
    Ordinary,
    /// Retain authenticated headers for an ordinary crate's typed artifact.
    BootstrapLibrary,
    Source {
        selected: Option<OutputKind>,
    },
}
