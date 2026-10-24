#![feature(decl_macro)]
pub fn identity<T>(value: T) -> T {
    macro keep($value:expr) {{ let result: T = $value; result }}
    keep!(value)
}
