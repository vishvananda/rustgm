//! Every append checks the complete encoded payload bound.
use super::{CrateIdentity, MAX_BYTES};
#[derive(Default)]
pub(super) struct Writer {
    bytes: Vec<u8>,
    source: Option<(usize, usize)>,
}
impl Writer {
    pub(super) const fn source(base: usize) -> Self {
        Self {
            bytes: Vec::new(),
            source: Some((base, 0)),
        }
    }
    pub(super) fn bytes(&mut self, bytes: &[u8]) -> Result<(), &'static str> {
        if let Some((base, steps)) = &mut self.source {
            *steps = steps.saturating_add(1);
            let capacity = self
                .bytes
                .len()
                .saturating_add(bytes.len())
                .max(8)
                .saturating_mul(4);
            if *steps > crate::support::crates::MAX_COMPONENTS
                || base.saturating_add(capacity) > MAX_BYTES
            {
                return Err("source encoded output exceeds combined work/storage budget");
            }
        }
        if self
            .bytes
            .len()
            .checked_add(bytes.len())
            .is_none_or(|length| length > MAX_BYTES)
        {
            return Err("metadata artifact exceeds 16 MB budget");
        }
        self.bytes.extend_from_slice(bytes);
        Ok(())
    }
    pub(super) fn u8(&mut self, value: u8) -> Result<(), &'static str> {
        self.bytes(&[value])
    }
    pub(super) fn u16(&mut self, value: u16) -> Result<(), &'static str> {
        self.bytes(&value.to_le_bytes())
    }
    pub(super) fn u32(&mut self, value: usize) -> Result<(), &'static str> {
        let value = u32::try_from(value).map_err(|_| "metadata count exceeds 32 bits")?;
        self.bytes(&value.to_le_bytes())
    }
    pub(super) fn string(&mut self, value: &str) -> Result<(), &'static str> {
        self.u32(value.len())?;
        self.bytes(value.as_bytes())
    }
    pub(super) fn path(&mut self, path: &[String]) -> Result<(), &'static str> {
        self.u16(u16::try_from(path.len()).map_err(|_| "metadata path exceeds 16 bits")?)?;
        for name in path {
            self.string(name)?;
        }
        Ok(())
    }
    pub(super) fn identity(&mut self, identity: &CrateIdentity) -> Result<(), &'static str> {
        self.string(identity.name())?;
        self.string(identity.disambiguator())
    }
    pub(super) fn finish(self) -> Vec<u8> {
        self.bytes
    }
}
