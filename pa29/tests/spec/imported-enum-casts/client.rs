extern crate base;

const NEXT: u8 = base::Header::Next as u8;

#[unsafe(no_mangle)]
pub extern "C" fn explicit() -> u8 {
    base::Header::Explicit as u8
}

#[unsafe(no_mangle)]
pub extern "C" fn implicit() -> u8 {
    NEXT
}

#[unsafe(no_mangle)]
pub extern "C" fn empty_tuple() -> u8 {
    base::EmptyShapes::Tuple() as u8
}

#[unsafe(no_mangle)]
pub extern "C" fn empty_record() -> u8 {
    base::EmptyShapes::Record {} as u8
}
