#![feature(no_core)]
#![no_core]
extern crate base;
fn bit(value:bool)->u32 { if value {1} else {0} }
#[unsafe(no_mangle)] pub extern "C" fn obs_wrap_equal()->u32 { bit(base::compare(&base::Wrap(11),&base::Wrap(11))) }
#[unsafe(no_mangle)] pub extern "C" fn obs_wrap_different()->u32 { bit(base::compare(&base::Wrap(11),&base::Wrap(22))) }
#[unsafe(no_mangle)] pub extern "C" fn obs_reference_equal()->u32 { bit(base::references(&33,&33)) }
#[unsafe(no_mangle)] pub extern "C" fn obs_reference_different()->u32 { bit(base::references(&33,&44)) }
#[unsafe(no_mangle)] pub extern "C" fn obs_nested_equal()->u32 { bit(base::nested(&&55,&&55)) }
#[unsafe(no_mangle)] pub extern "C" fn obs_nested_different()->u32 { bit(base::nested(&&55,&&66)) }
#[unsafe(no_mangle)] pub extern "C" fn obs_generic_equal()->u32 { bit(base::forwarded(77,77)) }
#[unsafe(no_mangle)] pub extern "C" fn obs_generic_different()->u32 { bit(base::forwarded(77,88)) }
