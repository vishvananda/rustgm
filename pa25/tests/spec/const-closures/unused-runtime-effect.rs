#![feature(const_closures, const_trait_impl, const_destruct)]
fn runtime()->u8 {23}
pub const fn make(){let _=const || runtime();}
