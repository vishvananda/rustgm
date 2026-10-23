#![feature(decl_macro)]
macro make($name:ident) {
    fn helper() -> u32 { 41 }
    pub fn $name() -> u32 { helper() }
}
make!(first);
make!(second);
fn helper() -> u32 { 43 }
#[unsafe(no_mangle)] pub extern "C" fn answer() -> u32 { if first() == 41 && second() == 41 { 82 } else { 0 } }
