#![feature(no_core,asm_native_fixture)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)] pub extern "C" fn obs_increment()->u64 {base::increment::<u64>(10)}
#[unsafe(no_mangle)] pub extern "C" fn obs_late()->u64 {base::late::<u64>(77)}
#[unsafe(no_mangle)] pub extern "C" fn obs_aliased()->u64 {base::aliased::<u64>()}
#[unsafe(no_mangle)] pub extern "C" fn obs_stored()->u64 {let mut value=0; base::stored::<u64>(&mut value,99); value}
#[unsafe(no_mangle)] pub extern "C" fn obs_att()->i64 {base::att::<u64>(7)}
