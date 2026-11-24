#![feature(no_core)]
#![no_core]

extern crate base;

fn init() -> u8 {
    41
}

struct Local;

impl base::Output for Local {
    type Value = u8;
}

impl base::Fallback for Local {
    fn answer(value: u8) -> u8 {
        value
    }
}

impl base::Closed for Local {
    fn answer(_: ()) -> u8 {
        17
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_flow() -> u8 {
    let step: base::Flow<u8> = base::Flow::Continue(());
    match step {
        base::Flow::Break(value) => value,
        base::Flow::Continue(()) => 19,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_lazy() -> u8 {
    let lazy: base::Lazy<u8> = base::Lazy { value: 9, init };
    (lazy.init)()
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_closed() -> u8 {
    <Local as base::Closed>::answer(())
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_projection() -> u8 {
    <Local as base::Fallback>::answer(71)
}

#[cfg(incompatible_default)]
fn wrong_init() -> u16 {
    1
}

#[cfg(incompatible_default)]
pub fn wrong() {
    let _: base::Lazy<u8> = base::Lazy { value: 9, init: wrong_init };
}
