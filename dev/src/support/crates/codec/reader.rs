//! Untrusted lengths are checked before allocation or integer-to-index conversion.
use super::{CrateIdentity, Scalar};
use crate::support::crates::{MAX_COMPONENTS, MAX_NAME_BYTES};
pub(super) struct Reader<'a> {
    bytes: &'a [u8],
    cursor: usize,
    names: usize,
    atoms: usize,
}
impl<'a> Reader<'a> {
    pub(super) const fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            cursor: 0,
            names: 0,
            atoms: 0,
        }
    }
    pub(super) fn bytes(&mut self, count: usize) -> Result<&'a [u8], &'static str> {
        let end = self
            .cursor
            .checked_add(count)
            .ok_or("metadata length overflow")?;
        let value = self
            .bytes
            .get(self.cursor..end)
            .ok_or("truncated metadata artifact")?;
        self.cursor = end;
        Ok(value)
    }
    pub(super) fn u8(&mut self) -> Result<u8, &'static str> {
        self.bytes(1)?
            .first()
            .copied()
            .ok_or("truncated metadata byte")
    }
    pub(super) fn u16(&mut self) -> Result<u16, &'static str> {
        Ok(u16::from_le_bytes(
            self.bytes(2)?
                .try_into()
                .map_err(|_| "truncated metadata u16")?,
        ))
    }
    pub(super) fn u32(&mut self) -> Result<usize, &'static str> {
        let value = u32::from_le_bytes(
            self.bytes(4)?
                .try_into()
                .map_err(|_| "truncated metadata u32")?,
        );
        usize::try_from(value).map_err(|_| "metadata count exceeds host index")
    }
    pub(super) fn u128(&mut self) -> Result<u128, &'static str> {
        Ok(u128::from_le_bytes(
            self.bytes(16)?
                .try_into()
                .map_err(|_| "truncated metadata u128")?,
        ))
    }
    pub(super) fn owner(&mut self, dependencies: usize) -> Result<(), &'static str> {
        let ordinal = self.u32()?;
        if ordinal > dependencies {
            return Err("unknown metadata owner ID");
        }
        if ordinal != 0 {
            return Err("foreign definition in metadata export table");
        }
        Ok(())
    }
    pub(super) fn path(&mut self) -> Result<Vec<String>, &'static str> {
        let path = self.module_path()?;
        if path.is_empty() {
            return Err("invalid metadata path table length");
        }
        Ok(path)
    }
    pub(super) fn module_path(&mut self) -> Result<Vec<String>, &'static str> {
        let count = usize::from(self.u16()?);
        if count > 64 || count > self.remaining() / 5 {
            return Err("invalid metadata path table length");
        }
        self.atoms(count)?;
        let mut path = Vec::with_capacity(count);
        for _ in 0..count {
            path.push(self.string(256)?);
        }
        Ok(path)
    }
    pub(super) fn count(
        &mut self,
        limit: usize,
        minimum_bytes: usize,
    ) -> Result<usize, &'static str> {
        let count = self.u32()?;
        if count > limit {
            return Err("metadata table exceeds profile budget");
        }
        if count > self.remaining() / minimum_bytes {
            return Err("truncated metadata table");
        }
        Ok(count)
    }
    pub(super) fn string(&mut self, limit: usize) -> Result<String, &'static str> {
        let length = self.u32()?;
        if length > limit {
            return Err("metadata string exceeds profile budget");
        }
        self.names = self
            .names
            .checked_add(length)
            .filter(|bytes| *bytes <= MAX_NAME_BYTES)
            .ok_or("metadata names exceed profile budget")?;
        let value =
            std::str::from_utf8(self.bytes(length)?).map_err(|_| "metadata name is not UTF-8")?;
        Ok(value.to_owned())
    }
    pub(super) fn identity(&mut self) -> Result<CrateIdentity, &'static str> {
        CrateIdentity::new(&self.string(64)?, &self.string(64)?)
    }
    pub(super) fn atoms(&mut self, count: usize) -> Result<(), &'static str> {
        self.atoms = self
            .atoms
            .checked_add(count)
            .filter(|atoms| *atoms <= MAX_COMPONENTS)
            .ok_or("metadata components exceed profile budget")?;
        Ok(())
    }
    pub(super) fn scalar(&mut self) -> Result<Scalar, &'static str> {
        Ok(match self.u8()? {
            0 => Scalar::Unit,
            1 => Scalar::Bool,
            2 => Scalar::Char,
            3 => Scalar::U8,
            4 => Scalar::U16,
            5 => Scalar::U32,
            6 => Scalar::U64,
            7 => Scalar::U128,
            8 => Scalar::Usize,
            9 => Scalar::I8,
            10 => Scalar::I16,
            11 => Scalar::I32,
            12 => Scalar::I64,
            13 => Scalar::I128,
            14 => Scalar::Isize,
            15 => Scalar::F32,
            16 => Scalar::F64,
            _ => return Err("unknown metadata scalar type"),
        })
    }
    pub(super) const fn done(&self) -> bool {
        self.cursor == self.bytes.len()
    }
    pub(super) const fn remaining(&self) -> usize {
        self.bytes.len() - self.cursor
    }
}
