extern crate cprov;

const BAD: u32 = unsafe { cprov::MUT_EXT };

fn main() {
    let _ = BAD;
}
