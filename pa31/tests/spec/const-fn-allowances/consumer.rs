#![feature(no_core,rustc_attrs,staged_api)]
#![no_core]
#![stable(feature="root",since="1.0.0")]
#![allow(internal_features,dead_code)]
extern crate minicore;
#[stable(feature="root",since="1.0.0")]pub const CHECK:u8=minicore::caller();
#[stable(feature="root",since="1.0.0")]#[unsafe(no_mangle)]pub extern "C" fn consumer_observed()->u8{CHECK}

fn main(){}
