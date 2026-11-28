#![feature(no_core, pointer_metadata_fixture)]
#![allow(internal_features)]
#![no_core]
extern crate base;

struct Value(u8);
impl base::Read<u8> for Value {
    type Item = u8;
    fn read(&self, _: u8) -> u8 { self.0 }
}
struct Echo;
impl<'v> base::Visit<'v> for Echo {
    fn visit(&self, value: &'v u8) -> u8 { *value }
}
fn require_shared<T: ?base::Sized + base::Send + base::Sync>(_: &T) {}
fn read(reader: base::Reader<'_>) -> u8 {
    require_shared(reader.source);
    reader.source.read(0)
}
fn visit(visitor: base::Visitor<'_>) -> u8 {
    let local = 23;
    visitor.inner.visit(&local)
}
#[unsafe(no_mangle)]
pub extern "C" fn observed_field() -> u8 { read(base::Reader { source: &Value(41) }) }
#[unsafe(no_mangle)]
pub extern "C" fn observed_signature() -> u8 {
    base::read_through(base::Reader { source: &Value(7) }, 0)
}
#[unsafe(no_mangle)]
pub extern "C" fn observed_higher_ranked() -> u8 { visit(base::Visitor { inner: &Echo }) }
