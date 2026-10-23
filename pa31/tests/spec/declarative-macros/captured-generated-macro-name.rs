#![feature(decl_macro)]
pub macro create($name:ident) {
    pub fn helper() -> u32 { 17 }
    pub macro $name() { helper() }
}
create!(generated);
pub fn helper() -> u32 { 99 }
#[unsafe(no_mangle)] pub extern "C" fn answer() -> u32 { generated!() }
