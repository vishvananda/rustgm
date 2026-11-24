#![feature(no_core)]
#![no_core]

extern crate base;

fn main() {}

const LONG: u8 = 23;

#[cfg(accepted)]
#[unsafe(no_mangle)]
pub extern "C" fn inspect(short: &u8) -> u8 {
    let window = base::Window { short, long: &LONG };
    *window.long
}

#[cfg(accepted)]
pub fn choose<'short, 'long: 'short>(value: &'long u8) -> base::Choice<'short, 'long> {
    base::Choice::Long(value)
}

#[cfg(static_mismatch)]
pub fn bad_window<'local>(long: &'local u8) -> u8 {
    let window: base::Window<'static, 'local> = base::Window { short: &LONG, long };
    *window.long
}

#[cfg(enum_mismatch)]
pub fn bad_choice<'local>(value: &'local u8) -> u8 {
    let choice: base::Choice<'static, 'local> = base::Choice::Long(value);
    match choice {
        base::Choice::Short(value) => *value,
        base::Choice::Long(value) => *value,
    }
}
