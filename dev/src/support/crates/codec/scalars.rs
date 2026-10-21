//! Profile one adds closed scalar values after the unchanged function table.
use super::{Reader, Writer};
use crate::support::crates::{
    Constant, CrateIdentity, ExternalDef, ExternalKind, MAX_FUNCTIONS, ScalarValue,
};

pub(super) fn encode(out: &mut Writer, constants: &[Constant]) -> Result<(), &'static str> {
    out.u32(constants.len())?;
    for constant in constants {
        out.u32(0)?; // Export owner, never a local declaration identity.
        out.bytes(&[1, 2])?; // Value namespace, Constant kind; no runtime linkage.
        out.u8(constant.value().ty() as u8)?;
        out.path(constant.definition().path())?;
        out.bytes(&constant.value().bits().to_le_bytes())?;
    }
    Ok(())
}
pub(super) fn decode(
    input: &mut Reader<'_>,
    owner: &CrateIdentity,
    functions: usize,
) -> Result<Vec<Constant>, &'static str> {
    let count = input.count(MAX_FUNCTIONS.saturating_sub(functions), 30)?;
    if count == 0 {
        return Err("empty scalar value profile is not canonical");
    }
    let mut constants = Vec::with_capacity(count);
    for _ in 0..count {
        input.owner(0)?;
        if input.bytes(2)? != [1, 2] {
            return Err("unsupported metadata constant identity");
        }
        let ty = input.scalar()?;
        input.atoms(1)?;
        let definition = ExternalDef::new(owner.clone(), input.path()?, ExternalKind::Constant)?;
        constants.push(Constant::new(
            definition,
            ScalarValue::new(ty, input.u128()?)?,
        )?);
    }
    if constants
        .windows(2)
        .any(|pair| pair[0].definition() >= pair[1].definition())
    {
        return Err("metadata constants are duplicate or noncanonical");
    }
    Ok(constants)
}
