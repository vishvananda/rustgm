#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 {
    match base::keep(base::Opt::Some(42u8), |x| base::large(x)) {
        base::Opt::Some(x) => match base::keep(base::Opt::Some(2u8), |x| base::large(x)) {
            base::Opt::None => match x { 42 => 42, _ => 9 },
            base::Opt::Some(_) => 8,
        },
        base::Opt::None => 7,
    }
}
