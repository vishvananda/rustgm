extern crate cprov;

fn check(ok: bool) {
    if !ok {
        let z = 0u8;
        let _ = 1u8 / z;
    }
}

fn main() {
    let f: extern "C" fn(u32, u32) -> u32 = cprov::add;
    check(f(20, 22) == 42);
    check(cprov::real(1.25, 2.5) == 3.75);
    check(unsafe { cprov::guarded(41) } == 42);
    check(cprov::exact(38) == 42);
    let m = cprov::bump_mixed(cprov::Mixed { n: 10, x: 1.5 });
    check(m.n == 13 && m.x == 4.0);
    let l = cprov::bump_large(cprov::Large {
        cells: [1, 2, 3, 4, 35],
    });
    check(l.cells[0] == 1 && l.cells[4] == 42);
}
