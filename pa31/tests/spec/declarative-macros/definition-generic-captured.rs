#![feature(decl_macro)]
pub fn identity<T>(value: T) -> T {
    macro keep($ty:ident, $value:expr) {{ let result: $ty = $value; result }}
    keep!(T, value)
}
