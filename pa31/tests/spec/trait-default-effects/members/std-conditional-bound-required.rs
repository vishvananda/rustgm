#![feature(const_trait_impl)]
#![allow(dead_code)]
pub const trait Q:Sized{fn required()->u8;fn provided()->u8{Self::required()}}const impl Q for u8{fn required()->u8{29}}const fn via<T:[const]Q>()->u8{T::required()}const N:u8=via::<u8>();#[unsafe(no_mangle)]pub extern "C" fn answer()->u8{N}
