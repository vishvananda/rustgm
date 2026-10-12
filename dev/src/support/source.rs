//! Original UTF-8 bytes and an optional normalized, append-only semantic spelling view.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FileId(pub u32);

/// Original bytes stay available for future physical source diagnostics.
pub struct SourceFile {
    original: String,
    normalized: Option<String>,
    physical_len: usize,
    line_starts: std::sync::OnceLock<Vec<u32>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Position {
    pub line: u32,
    pub column: u32,
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
            line_starts: std::sync::OnceLock::new(),
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
            line_starts: self.line_starts.clone(),
        }
    }

    /// Locate a normalized offset in the original physical source. Columns count
    /// Unicode scalar values; the UTF-8 BOM does not occupy a source column.
    /// # Errors
    /// Rejects generated, out-of-range and non-character-boundary offsets.
    pub fn position(&self, offset: u32) -> Result<Position, &'static str> {
        let source = &self.text()[..self.physical_len];
        let end = offset as usize;
        if end > source.len() || !source.is_char_boundary(end) {
            return Err("physical source offset is out of range or inside a character");
        }
        let starts = self.line_starts.get_or_init(|| {
            std::iter::once(0)
                .chain(
                    source
                        .bytes()
                        .enumerate()
                        .filter(|(_, byte)| *byte == b'\n')
                        .map(|(index, _)| u32::try_from(index + 1).unwrap_or(u32::MAX)),
                )
                .collect()
        });
        let line = starts.partition_point(|start| *start <= offset);
        let start = starts[line.saturating_sub(1)] as usize;
        let prefix = &source[start..end];
        let prefix = if start == 0 {
            prefix.strip_prefix('\u{feff}').unwrap_or(prefix)
        } else {
            prefix
        };
        Ok(Position {
            line: u32::try_from(line).map_err(|_| "too many source lines")?,
            column: u32::try_from(prefix.chars().count() + 1)
                .map_err(|_| "source column overflow")?,
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
