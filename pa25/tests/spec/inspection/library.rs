const fn add(n: u8) -> u8 { n + 1 }
const NUMBER: u8 = add(6);
static STORAGE: u8 = 7;
const LEFT: &u8 = &STORAGE;
const RIGHT: &u8 = &STORAGE;
