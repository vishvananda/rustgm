#![no_std]
// `Some` and `None` come from the injected edition prelude, which re-exports
// core's variants; `Present` and `Absent` rename them through a module path.
fn pick(value: Option<u32>) -> u32 {
    match value {
        Some(_) => forty_two!(),
        None => core::base(),
    }
}
fn renamed(value: Option<u32>) -> Option<u32> {
    match value {
        core::reexports::Present(v) => core::reexports::Present(v),
        core::reexports::Absent => core::prelude::v1::None,
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn nostd_prelude() -> u32 {
    pick(renamed(Some(7)))
}
#[unsafe(no_mangle)]
pub extern "C" fn nostd_path() -> u32 {
    pick(renamed(core::prelude::v1::None))
}
