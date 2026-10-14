mod core { pub mod marker { pub trait Copy {} } }
fn f<T: core::marker::Copy>() {}
