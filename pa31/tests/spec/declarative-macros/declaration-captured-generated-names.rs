#![feature(decl_macro)]
macro_rules! declare{($name:ident)=>{pub macro $name(){not_resolved()}};}declare!(first);declare!(second);
#[unsafe(no_mangle)] pub extern "C" fn answer()->u32{7u32}
