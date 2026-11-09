#![feature(no_core)]
#![no_core]
extern crate base;
// core's FromBytesWithNulError shape: `InteriorNul` built by field name,
// copied through the imported Copy impl and matched by name.
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 {
    let e = base::NulError::InteriorNul { position: 42 };
    let copy = e;
    match copy {
        base::NulError::InteriorNul { position } => match position {
            42 => match base::position(e) {
                42 => match base::position(base::interior(7)) {
                    7 => match base::NulError::NotNulTerminated {
                        base::NulError::NotNulTerminated => 42,
                        base::NulError::InteriorNul { .. } => 5,
                    },
                    _ => 4,
                },
                _ => 3,
            },
            _ => 2,
        },
        base::NulError::NotNulTerminated => 1,
    }
}
