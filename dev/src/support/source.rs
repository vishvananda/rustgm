//! One owned UTF-8 file, with a normalized view only when CRLF requires it.

/// Original bytes stay available for future physical source diagnostics.
pub struct SourceFile {
    original: String,
    normalized: Option<String>,
}

impl SourceFile {
    /// Read a source image without the C++ translation-phase transformations.
    ///
    /// # Errors
    /// Rejects invalid UTF-8 and images too large for compact source offsets.
    pub fn new(bytes: Vec<u8>) -> Result<Self, String> {
        if u32::try_from(bytes.len()).is_err() {
            return Err("source exceeds the 32-bit offset limit".into());
        }
        let original = String::from_utf8(bytes).map_err(|error| {
            format!("invalid UTF-8 at byte {}", error.utf8_error().valid_up_to())
        })?;
        let normalized = original
            .contains("\r\n")
            .then(|| original.replace("\r\n", "\n"));
        Ok(Self {
            original,
            normalized,
        })
    }

    #[must_use]
    pub fn text(&self) -> &str {
        self.normalized.as_deref().unwrap_or(&self.original)
    }

    #[must_use]
    pub fn original(&self) -> &str {
        &self.original
    }
}
