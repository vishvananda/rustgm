extern crate cprov;

fn check(ok: bool) {
    if !ok {
        let z = 0u8;
        let _ = 1u8 / z;
    }
}

fn main() {
    check(cprov::IMM == 7);
    check((&cprov::IMM as *const u32) == (cprov::REF as *const u32));
    check(cprov::SAFE_EXT == 23);
    unsafe {
        cprov::MUT = 41;
        check(cprov::MUT == 41);
        check(cprov::MUT_EXT == 29);
        cprov::MUT_EXT = 31;
        check(cprov::MUT_EXT == 31);
        check(cprov::sum_va(3, 10i32, 12i32, 20i32) == 42);
    }
}
