#![feature(const_trait_impl)]
#![allow(dead_code)]
fn nonconst()->u8{29}const fn constant()->u8{29}pub const trait K{fn mark()->u8;}const fn conditional<T:[const]K>()->u8{29}struct S;pub const trait Q:Sized{fn provided()->u8{nonconst()}}const impl Q for S{}#[unsafe(no_mangle)]pub extern "C" fn answer()->u8{<S as Q>::provided()}
