//! Original UTF-8 bytes and an optional normalized, append-only semantic spelling view.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FileId(pub u32);

/// Original bytes stay available for future physical source diagnostics.
pub struct SourceFile {
    original: String,
    normalized: Option<String>,
    physical_len: usize,
}

impl SourceFile {
    /// Validate the offset of a new generated spelling image before committing it.
    /// A separating newline keeps physical and generated spans disjoint.
    ///
    /// # Errors
    /// Rejects semantic images exceeding the compact offset limit.
    pub fn append_offset(&self, text: &str) -> Result<u32, &'static str> {
        let start = self.text().len().checked_add(1);
        start
            .and_then(|n| n.checked_add(text.len()))
            .and_then(|n| u32::try_from(n).ok())
            .ok_or("semantic source exceeds the 32-bit offset limit")?;
        start
            .and_then(|n| u32::try_from(n).ok())
            .ok_or("invalid semantic source offset")
    }

    /// Append demanded expansion spelling. Existing offsets and original bytes
    /// remain stable; consumers use typed tokens and never reparse this view.
    ///
    /// # Errors
    /// Rejects semantic images exceeding the compact offset limit.
    pub fn append(&mut self, text: &str) -> Result<u32, &'static str> {
        let start = self.append_offset(text)?;
        let view = self.normalized.get_or_insert_with(|| self.original.clone());
        view.push('\n');
        view.push_str(text);
        Ok(start)
    }
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
        let physical_len = normalized.as_ref().map_or(original.len(), String::len);
        Ok(Self {
            original,
            normalized,
            physical_len,
        })
    }

    /// Copy a demanded module instance's original view without normalizing again
    /// or retaining any spelling appended by a previous expansion instance.
    #[must_use]
    pub fn physical_copy(&self) -> Self {
        Self {
            original: self.original.clone(),
            normalized: (self.original.len() != self.physical_len)
                .then(|| self.text()[..self.physical_len].to_owned()),
            physical_len: self.physical_len,
        }
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
