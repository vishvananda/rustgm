#![feature(no_core)]
#![no_core]
extern crate base;

#[unsafe(no_mangle)]
pub extern "C" fn read_shared(first: u8, second: u8) -> u8 {
    let first = base::Byte::<3>(&first);
    let second = base::Byte::<3>(&second);
    let mut cursor = &first;
    let previous = base::Advance::advance(&mut cursor, &second);
    match previous {
        17 => *cursor.0,
        _ => previous,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn read_unique(first: u8, second: u8) -> u8 {
    let mut first = base::Byte::<3>(&first);
    let mut second = base::Byte::<3>(&second);
    let mut cursor = &mut first;
    let previous = base::Advance::advance_mut(&mut cursor, &mut second);
    match previous {
        25 => *cursor.0,
        _ => previous,
    }
}
