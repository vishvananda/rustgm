#![feature(const_closures, const_trait_impl, const_destruct)]
struct Owned(u8);
pub const fn compute()->u8 {
    let value=Owned(23);
    let call=const move || {let whole=value;whole.0};
    let _=call();
    value.0
}
