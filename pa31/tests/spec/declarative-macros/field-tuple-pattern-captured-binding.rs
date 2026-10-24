#![feature(decl_macro)]
mod provider {
    pub struct Record(u32);
    pub fn make() -> Record { Record(41) }
    pub macro read($record:expr, $binding:ident) {{
        let Record($binding) = $record;
        $binding
    }}
}
#[unsafe(no_mangle)] pub extern "C" fn answer() -> u32 {
    provider::read!(provider::make(), selected)
}
