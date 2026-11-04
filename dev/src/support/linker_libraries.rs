//! Ordered native dependencies are separate from foreign symbol names and `LowIR`.
use std::io::{self, Write};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Dylib,
    Static,
}
impl Kind {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Dylib => "dylib",
            Self::Static => "static",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Library {
    name: Box<str>,
    kind: Kind,
}
impl Library {
    /// # Errors
    /// Rejects empty, NUL-containing or excessive names before retaining bytes.
    pub fn new(name: &str, kind: Kind) -> Result<Self, &'static str> {
        if name.is_empty() || name.contains('\0') || name.len() > 4096 {
            return Err("invalid native library name");
        }
        Ok(Self {
            name: name.into(),
            kind,
        })
    }
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    #[must_use]
    pub const fn kind(&self) -> Kind {
        self.kind
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Plan {
    libraries: Vec<Library>,
    name_bytes: usize,
}
impl Plan {
    /// # Errors
    /// Bound both record count and aggregate name storage before publication.
    pub fn push(&mut self, library: Library) -> Result<(), &'static str> {
        let bytes = self.name_bytes.saturating_add(library.name.len());
        if self.libraries.len() >= 1024 || bytes > 65_536 {
            return Err("native library plan exceeds profile budget");
        }
        self.libraries
            .try_reserve(1)
            .map_err(|_| "native library allocation failed")?;
        self.libraries.push(library);
        self.name_bytes = bytes;
        Ok(())
    }
    #[must_use]
    pub fn libraries(&self) -> &[Library] {
        &self.libraries
    }
    #[must_use]
    pub(crate) fn allocation_bytes(&self) -> usize {
        size_of::<Self>()
            .saturating_add(
                self.libraries
                    .capacity()
                    .saturating_mul(size_of::<Library>()),
            )
            .saturating_add(self.name_bytes)
    }
    /// Write the external backend manifest. Compiler phases consume typed facts.
    /// # Errors
    /// Reports output errors without invoking an external tool.
    pub fn write(&self, output: &mut impl Write) -> io::Result<()> {
        write!(
            output,
            "{{\"schema\":1,\"target\":\"x86_64-linux\",\"libraries\":["
        )?;
        for (index, library) in self.libraries.iter().enumerate() {
            if index != 0 {
                output.write_all(b",")?;
            }
            write!(output, "{{\"kind\":\"{}\",\"name\":", library.kind.name())?;
            output.write_all(b"\"")?;
            for ch in library.name.chars() {
                match ch {
                    '"' => output.write_all(b"\\\"")?,
                    '\\' => output.write_all(b"\\\\")?,
                    ch if ch <= '\u{1f}' => write!(output, "\\u{:04x}", u32::from(ch))?,
                    ch => write!(output, "{ch}")?,
                }
            }
            output.write_all(b"\"}")?;
        }
        output.write_all(b"]}\n")
    }
}
