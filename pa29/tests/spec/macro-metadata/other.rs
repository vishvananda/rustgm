pub fn f() -> u8 { 9 }
#[macro_export]
macro_rules! call_f { () => { $crate::f() }; }
