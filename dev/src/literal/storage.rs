//! Move decoded literals across source-loading steps without owning each token.
use super::value::{Float, FloatType, Integer, Value};
use std::borrow::Cow;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bytes {
    Source { start: u32, end: u32 },
    Decoded { start: u32, end: u32 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoredValue {
    Integer(Integer),
    Float {
        decimal: Bytes,
        suffix: Option<FloatType>,
    },
    Character(char),
    Byte(u8),
    String(Bytes),
    ByteString(Bytes),
    CString(Bytes),
    Boolean(bool),
}

impl StoredValue {
    /// Rebase borrowed ranges after appending their source image. Decoded bytes
    /// already belong to the destination pool and keep their stored identity.
    ///
    /// # Errors
    /// Rejects source ranges overflowing compact offsets.
    pub fn offset_source(mut self, offset: u32) -> Result<Self, &'static str> {
        let bytes = match &mut self {
            Self::Float { decimal, .. } => Some(decimal),
            Self::String(bytes) | Self::ByteString(bytes) | Self::CString(bytes) => Some(bytes),
            _ => None,
        };
        if let Some(Bytes::Source { start, end }) = bytes {
            *start = start
                .checked_add(offset)
                .ok_or("literal offset exceeds 32 bits")?;
            *end = end
                .checked_add(offset)
                .ok_or("literal offset exceeds 32 bits")?;
        }
        Ok(self)
    }
}

/// One append-only buffer for escaped/normalized literal payloads in a source.
/// Plain literal contents remain ranges into that source's immutable image.
#[derive(Default, Debug)]
pub struct Storage {
    decoded: Vec<u8>,
}

impl Storage {
    pub(crate) fn copy_prefix(&self, size: usize) -> Result<Self, &'static str> {
        self.decoded
            .get(..size)
            .map(|bytes| Self {
                decoded: bytes.to_vec(),
            })
            .ok_or("invalid literal-pool prefix")
    }
    pub(crate) fn truncate(&mut self, size: usize) {
        self.decoded.truncate(size);
    }
    /// # Errors
    /// Rejects borrowed data outside the source or a decoded pool over 4 GiB.
    pub fn store(&mut self, source: &str, value: &Value<'_>) -> Result<StoredValue, &'static str> {
        Ok(match value {
            Value::Integer(value) => StoredValue::Integer(*value),
            Value::Character(value) => StoredValue::Character(*value),
            Value::Byte(value) => StoredValue::Byte(*value),
            Value::Boolean(value) => StoredValue::Boolean(*value),
            Value::Float(value) => StoredValue::Float {
                decimal: match &value.decimal {
                    Cow::Borrowed(text) => Self::borrowed(source, text.as_bytes())?,
                    Cow::Owned(text) => self.owned(text.as_bytes())?,
                },
                suffix: value.suffix,
            },
            Value::String(bytes) => StoredValue::String(self.bytes(source, bytes)?),
            Value::ByteString(bytes) => StoredValue::ByteString(self.bytes(source, bytes)?),
            Value::CString(bytes) => StoredValue::CString(self.bytes(source, bytes)?),
        })
    }

    /// Copy decoded bytes once and rebase source ranges when installing cached syntax.
    pub(crate) fn import(
        &mut self,
        source: &str,
        storage: &Self,
        mut value: StoredValue,
        base: u32,
    ) -> Result<StoredValue, &'static str> {
        let bytes = match &mut value {
            StoredValue::Float { decimal, .. } => Some(decimal),
            StoredValue::String(bytes)
            | StoredValue::ByteString(bytes)
            | StoredValue::CString(bytes) => Some(bytes),
            _ => None,
        };
        if let Some(bytes) = bytes {
            let data = storage.get(source, *bytes)?;
            if matches!(bytes, Bytes::Decoded { .. }) {
                *bytes = self.owned(data)?;
            }
        }
        value.offset_source(base)
    }

    #[expect(
        clippy::ptr_arg,
        reason = "Cow ownership distinguishes source ranges from decoded pool bytes"
    )]
    fn bytes(&mut self, source: &str, value: &Cow<'_, [u8]>) -> Result<Bytes, &'static str> {
        match value {
            Cow::Borrowed(bytes) => Self::borrowed(source, bytes),
            Cow::Owned(bytes) => self.owned(bytes),
        }
    }

    fn borrowed(source: &str, bytes: &[u8]) -> Result<Bytes, &'static str> {
        if bytes.is_empty() {
            return Ok(Bytes::Source { start: 0, end: 0 });
        }
        let start = bytes
            .as_ptr()
            .addr()
            .checked_sub(source.as_ptr().addr())
            .ok_or("literal does not borrow this source")?;
        let end = start
            .checked_add(bytes.len())
            .filter(|end| *end <= source.len())
            .ok_or("literal does not borrow this source")?;
        Ok(Bytes::Source {
            start: u32::try_from(start).map_err(|_| "literal offset exceeds 32 bits")?,
            end: u32::try_from(end).map_err(|_| "literal offset exceeds 32 bits")?,
        })
    }

    fn owned(&mut self, bytes: &[u8]) -> Result<Bytes, &'static str> {
        let start = u32::try_from(self.decoded.len())
            .map_err(|_| "decoded literal pool exceeds 32 bits")?;
        let end = self
            .decoded
            .len()
            .checked_add(bytes.len())
            .and_then(|end| u32::try_from(end).ok())
            .ok_or("decoded literal pool exceeds 32 bits")?;
        self.decoded.extend_from_slice(bytes);
        Ok(Bytes::Decoded { start, end })
    }

    /// Borrow an existing decoded value; this never interprets source again.
    ///
    /// # Errors
    /// Rejects invalid manually supplied storage ranges or decimal UTF-8.
    pub fn view<'a>(
        &'a self,
        source: &'a str,
        value: StoredValue,
    ) -> Result<Value<'a>, &'static str> {
        Ok(match value {
            StoredValue::Integer(value) => Value::Integer(value),
            StoredValue::Character(value) => Value::Character(value),
            StoredValue::Byte(value) => Value::Byte(value),
            StoredValue::Boolean(value) => Value::Boolean(value),
            StoredValue::Float { decimal, suffix } => Value::Float(Float {
                decimal: Cow::Borrowed(
                    std::str::from_utf8(self.get(source, decimal)?)
                        .map_err(|_| "invalid stored decimal UTF-8")?,
                ),
                suffix,
            }),
            StoredValue::String(bytes) => Value::String(Cow::Borrowed(self.get(source, bytes)?)),
            StoredValue::ByteString(bytes) => {
                Value::ByteString(Cow::Borrowed(self.get(source, bytes)?))
            }
            StoredValue::CString(bytes) => Value::CString(Cow::Borrowed(self.get(source, bytes)?)),
        })
    }

    fn get<'a>(&'a self, source: &'a str, range: Bytes) -> Result<&'a [u8], &'static str> {
        let (bytes, start, end) = match range {
            Bytes::Source { start, end } => (source.as_bytes(), start, end),
            Bytes::Decoded { start, end } => (self.decoded.as_slice(), start, end),
        };
        bytes
            .get(start as usize..end as usize)
            .ok_or("invalid stored literal range")
    }

    #[must_use]
    pub fn decoded_bytes(&self) -> usize {
        self.decoded.len()
    }
}
