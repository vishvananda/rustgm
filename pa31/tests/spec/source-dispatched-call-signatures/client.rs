#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)] pub extern "C" fn obs_free()->u32 {base::through(11)}
#[unsafe(no_mangle)] pub extern "C" fn obs_map()->u32 {base::mapped(22)}
#[unsafe(no_mangle)] pub extern "C" fn obs_tuple()->u32 {base::tuple(33)}
#[unsafe(no_mangle)] pub extern "C" fn obs_nominal()->u32 {base::nominal(44)}
#[unsafe(no_mangle)] pub extern "C" fn obs_array()->u32 {base::array(55)}
#[unsafe(no_mangle)] pub extern "C" fn obs_distinct()->u32 {base::distinct(11)}
#[unsafe(no_mangle)] pub extern "C" fn obs_forward()->u32 {base::forwarded(66)}
#[unsafe(no_mangle)] pub extern "C" fn obs_explicit()->u32 {base::explicit_unsafe(77)}
#[unsafe(no_mangle)] pub extern "C" fn obs_scope()->u32 {unsafe {base::unsafe_scope(88)}}
