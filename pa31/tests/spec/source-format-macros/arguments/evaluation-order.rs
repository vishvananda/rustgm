use std::cell::Cell;

fn next(log: &Cell<u32>, digit: u32) -> u32 {
    log.set(log.get() * 10 + digit);
    digit
}

fn main() {
    let log = Cell::new(0);
    let args = format_args!("{1:?} {0:?} {1:?}", next(&log, 1), next(&log, 2));
    println!("{args}|{}", log.get());
}
