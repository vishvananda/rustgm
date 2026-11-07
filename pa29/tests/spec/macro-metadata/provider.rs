pub fn f() -> u8 { 40 }
#[macro_export]
macro_rules! call_f { () => { $crate::f() }; }
#[macro_export]
macro_rules! select { ($p:pat) => { match 2u8 { $p => 1u8, _ => 0u8 } }; }
macro_rules! private_only { () => { 5u8 }; }
pub fn private_use() -> u8 { private_only!() }
pub mod inner {
    pub fn g() -> u8 { 2 }
    #[macro_export]
    macro_rules! nested_export { ($($e:expr),*) => { $crate::inner::g() $(+ $e)* }; }
}
