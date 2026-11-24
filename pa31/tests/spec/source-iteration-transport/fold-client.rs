#![feature(no_core, const_trait_impl, const_closures)]
#![no_core]
extern crate base;

struct Values {
    first_pending: bool,
    second_pending: bool,
    first: u8,
    second: u8,
}

const impl base::Iterator for Values {
    type Item = u8;
    fn next(&mut self) -> base::Maybe<u8> {
        if self.first_pending {
            self.first_pending = false;
            base::Maybe::Present(self.first)
        } else if self.second_pending {
            self.second_pending = false;
            base::Maybe::Present(self.second)
        } else {
            base::Maybe::Absent
        }
    }
}

const fn measure(first_pending: bool, second_pending: bool, first: u8, second: u8) -> u8 {
    let values = Values {
        first_pending,
        second_pending,
        first,
        second,
    };
    base::Iterator::fold(values, 73, const |_, item| item)
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_runtime() -> u8 {
    measure(true, true, 11, 73)
}

const OBSERVED_ITEM: u8 = measure(true, true, 99, 42);
const OBSERVED_EMPTY: u8 = measure(false, false, 77, 88);

#[unsafe(no_mangle)]
pub extern "C" fn observed_const() -> u8 {
    OBSERVED_ITEM
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_empty() -> u8 {
    OBSERVED_EMPTY
}
