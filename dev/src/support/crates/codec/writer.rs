//! Every append checks the complete encoded payload bound.
use super::{CrateIdentity, MAX_BYTES};
#[derive(Default)]
pub(super) struct Writer {
    bytes: Vec<u8>,
}
impl Writer {
    pub(super) fn bytes(&mut self, bytes: &[u8]) -> Result<(), &'static str> {
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
    pub(super) fn identity(&mut self, identity: &CrateIdentity) -> Result<(), &'static str> {
        self.string(identity.name())?;
        self.string(identity.disambiguator())
    }
    pub(super) fn finish(self) -> Vec<u8> {
        self.bytes
    }
}
