#![feature(no_core)]
#![no_core]
extern crate base;

#[unsafe(no_mangle)]
pub extern "C" fn routed_line() -> u32 {
    base::route::line!()
}

#[unsafe(no_mangle)]
pub extern "C" fn alternate_line() -> u32 {
    base::alternate::line!()
}

#[unsafe(no_mangle)]
pub extern "C" fn nested_line() -> u32 {
    base::route::nested::line!()
}

fn provider_value() -> u32 { 99 }

#[unsafe(no_mangle)]
pub extern "C" fn forwarded() -> u32 {
    base::forward!(51)
}

#[unsafe(no_mangle)]
pub extern "C" fn opaque_at_provider() -> u32 {
    base::qualified_opaque!()
}

#[unsafe(no_mangle)]
pub extern "C" fn transparent_at_client() -> u32 {
    let value = 71;
    base::caller_value!()
}
