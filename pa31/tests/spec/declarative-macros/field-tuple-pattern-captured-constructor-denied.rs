#![feature(decl_macro)]
mod provider {
    pub struct Record(u32);
    pub fn make() -> Record { Record(41) }
    pub macro read($record:expr, $constructor:path, $binding:ident) {{
        let $constructor($binding) = $record;
        $binding
    }}
}
#[unsafe(no_mangle)] pub extern "C" fn answer() -> u32 {
    provider::read!(provider::make(), provider::Record, selected)
}
