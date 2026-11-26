extern crate base;

use base::select as renamed;

const OLD: usize = renamed(2 + 3, 7, true, 4);

#[unsafe(no_mangle)]
pub extern "C" fn legacy() -> usize {
    OLD
}

#[unsafe(no_mangle)]
pub extern "C" fn legacy_runtime(first: usize, second: usize) -> usize {
    base::select(5, first, true, second)
}

#[unsafe(no_mangle)]
pub extern "C" fn modern() -> usize {
    base::select::<5, false>(7, 4)
}

#[unsafe(no_mangle)]
pub extern "C" fn alias() -> usize {
    let select = base::select::<5, true>;
    select(4, 7)
}
