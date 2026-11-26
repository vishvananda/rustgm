#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)] pub extern "C" fn obs_outer()->u32 {let value=11;base::through(&base::Outer(base::Inner{value:&value}))}
#[unsafe(no_mangle)] pub extern "C" fn obs_borrow()->u32 {let value=22;*base::borrowed(&base::Outer(base::Inner{value:&value}))}
#[unsafe(no_mangle)] pub extern "C" fn obs_nested()->u32 {let value=33;base::nested(&base::Double(base::Outer(base::Inner{value:&value})))}
#[unsafe(no_mangle)] pub extern "C" fn obs_nested_borrow()->u32 {let value=44;*base::nested_borrow(&base::Double(base::Outer(base::Inner{value:&value})))}
#[unsafe(no_mangle)] pub extern "C" fn obs_coerced()->u32 {let value=55;base::coerced(&base::Outer(base::Inner{value:&value}))}
#[unsafe(no_mangle)] pub extern "C" fn obs_field()->u32 {let value=66;base::field(&base::Outer(base::Inner{value:&value}))}
#[unsafe(no_mangle)] pub extern "C" fn obs_indexed()->u32 {let value=77;base::indexed(&base::Outer(base::Inner{value:&value}))}
#[unsafe(no_mangle)] pub extern "C" fn obs_direct_index()->u32 {let value=88;base::direct_index(&base::Inner{value:&value})}
#[unsafe(no_mangle)] pub extern "C" fn obs_replace_old()->u32 {let value=99;let next=111;let mut outer=base::Outer(base::Inner{value:&value});base::replaced(&mut outer,&next)}
#[unsafe(no_mangle)] pub extern "C" fn obs_replace_new()->u32 {let value=99;let next=111;let mut outer=base::Outer(base::Inner{value:&value});base::replaced(&mut outer,&next);base::through(&outer)}
#[unsafe(no_mangle)] pub extern "C" fn obs_control()->u32 {let value=123;base::control(&base::Inner{value:&value})}
