#![feature(const_trait_impl)]
#![allow(dead_code)]
pub const trait Q:Sized{fn required()->u8;fn provided()->u8{Self::required()}}const fn via<T:Q>()->u8{T::required()}
