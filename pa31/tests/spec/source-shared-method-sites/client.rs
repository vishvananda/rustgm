#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)] pub extern "C" fn obs_direct()->u32 { base::direct(&base::Value(11),&base::Value(22),true) }
#[unsafe(no_mangle)] pub extern "C" fn obs_outer()->u32 { base::through(&base::Outer(base::Value(33)),&base::Value(44),true) }
#[unsafe(no_mangle)] pub extern "C" fn obs_outer_rhs()->u32 { base::through(&base::Outer(base::Value(33)),&base::Value(44),false) }
#[unsafe(no_mangle)] pub extern "C" fn obs_nested()->u32 { base::through_nested(&base::Double(base::Outer(base::Value(55))),&base::Value(66),true) }
#[unsafe(no_mangle)] pub extern "C" fn obs_nested_rhs()->u32 { base::through_nested(&base::Double(base::Outer(base::Value(55))),&base::Value(66),false) }
#[unsafe(no_mangle)] pub extern "C" fn obs_generic()->u32 { base::through_generic(&base::Outer(base::Value(77)),&base::Value(88),true) }
#[unsafe(no_mangle)] pub extern "C" fn obs_generic_rhs()->u32 { base::through_generic(&base::Outer(base::Value(77)),&base::Value(88),false) }
#[unsafe(no_mangle)] pub extern "C" fn obs_trait()->u32 { base::through_trait(&base::Outer(base::Value(11)),&base::Value(99),false) }
