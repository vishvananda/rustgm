#![feature(const_closures, const_trait_impl, const_destruct)]
pub struct Owned(u8);
const fn once<F:[const]FnOnce()->Owned>(call:F)->Owned {call()}
const fn compute(value:u8)->u8 {
    let owned=Owned(value);
    once(const move || owned).0
}
pub const ANSWER:u8=compute(23);
