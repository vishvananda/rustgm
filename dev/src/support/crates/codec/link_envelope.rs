//! Optional native dependencies wrap an unchanged, independently bounded interface.
use super::{Interface, MAGIC, Reader, SCHEMA, Writer};
use crate::support::linker_libraries::{Kind, Library, Plan};
pub(super) const TAG: u8 = 12;

pub(super) fn encode(interface: &Interface) -> Result<Vec<u8>, &'static str> {
    let plan = interface
        .native_links()
        .ok_or("missing native dependency plan")?;
    if plan
        .libraries()
        .iter()
        .any(|library| library.kind() == Kind::Static)
    {
        return Err("static native archive bundling in crate metadata is not implemented");
    }
    let inner = super::encode_core(interface)?;
    let mut output = Writer::default();
    output.bytes(MAGIC)?;
    output.u16(SCHEMA)?;
    output.bytes(&[0, 0, TAG, 1])?;
    output.u32(plan.libraries().len())?;
    for library in plan.libraries() {
        output.string(library.name())?;
    }
    output.u32(inner.len())?;
    output.bytes(&inner)?;
    Ok(output.finish())
}
pub(super) fn decode(input: &mut Reader<'_>) -> Result<Interface, &'static str> {
    if input.u8()? != 1 {
        return Err("unknown native dependency metadata version");
    }
    let count = input.u32()?;
    if count == 0 || count > 1024 {
        return Err("invalid native dependency count");
    }
    let mut plan = Plan::default();
    for _ in 0..count {
        let name = input.text(4096)?;
        plan.push(Library::new(name, Kind::Dylib)?)?;
    }
    let length = input.u32()?;
    let inner = input.bytes(length)?;
    if !input.done() || inner.get(12) == Some(&TAG) {
        return Err("trailing or nested native dependency metadata");
    }
    super::decode(inner)?.with_native_links(plan)
}
