#![feature(decl_macro)]
mod selected { pub fn value() -> u32 { 23 } }
pub macro choose($path:ident) {{
    mod selected { pub fn value() -> u32 { 29 } }
    $path::value()
}}
#[unsafe(no_mangle)] pub extern "C" fn answer() -> u32 { choose!(selected) }
