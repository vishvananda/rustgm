#![feature(no_core,rustc_attrs,staged_api)]
#![no_core]
#![stable(feature="root",since="1.0.0")]
#![allow(internal_features,dead_code)]
extern crate minicore;
#[stable(feature="root",since="1.0.0")]#[rustc_const_stable(feature="client_stable",since="1.0.0")]#[rustc_allow_const_fn_unstable(door)]pub const fn check()->u8{minicore::target()}
#[stable(feature="root",since="1.0.0")]#[unsafe(no_mangle)]pub extern "C" fn consumer_observed()->u8{check()}

fn main(){}
