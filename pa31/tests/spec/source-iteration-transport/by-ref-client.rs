#![feature(no_core, const_trait_impl)]
#![no_core]
extern crate base;

struct Values { value: u8 }

const impl base::Iterator for Values {
    type Item = u8;
    fn next(&mut self) -> base::Maybe<u8> { base::Maybe::Absent }
}

const fn measure(replacement: u8) -> u8 {
    let mut values = Values { value: 0 };
    let loan = base::Iterator::by_ref(&mut values);
    loan.value = replacement;
    values.value
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_runtime() -> u8 { measure(73) }

const FROM_LOAN: u8 = measure(42);

#[unsafe(no_mangle)]
pub extern "C" fn observed_const() -> u8 { FROM_LOAN }
