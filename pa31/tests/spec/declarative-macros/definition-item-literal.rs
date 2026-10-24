#![feature(decl_macro)]
pub fn identity(value: u8) -> u8 {
    type T = u8;
    macro keep($value:expr) {{ let result: T = $value; result }}
    keep!(value)
}
