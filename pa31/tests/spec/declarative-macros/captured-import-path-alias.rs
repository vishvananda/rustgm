#![feature(decl_macro)]
mod selected { pub fn value() -> u32 { 31 } }
pub macro choose($path:ident) {{
    mod selected { pub fn value() -> u32 { 37 } }
    use $path::value as chosen;
    chosen()
}}
#[unsafe(no_mangle)] pub extern "C" fn answer() -> u32 { choose!(selected) }
