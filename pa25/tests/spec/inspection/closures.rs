const FIRST: fn(u8) -> u8 = |x| x + 1;
const ALIAS: fn(u8) -> u8 = FIRST;
const SECOND: fn(u8) -> u8 = |x| x + 2;
