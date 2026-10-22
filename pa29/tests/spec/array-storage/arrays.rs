pub struct Cell { pub tag: u16, pub values: [u8; 3] }
static HIDDEN: [Cell; 2] = [Cell { tag: 7, values: [1, 2, 3] }, Cell { tag: 9, values: [4, 5, 6] }];
pub static LIST: [u32; 3] = [11, 22, 33];
pub static REPEAT: [u32; 32] = [9; 32];
pub static EMPTY: [u8; 0] = [];
pub static NESTED: [[u8; 3]; 2] = [[1, 2, 3], [4, 5, 6]];
pub static BYTES: [u8; 4] = *b"ab\0z";
pub static REF: &'static u32 = &LIST[1];
pub static FIELD: &'static u8 = &HIDDEN[1].values[2];
pub static RECORD: Cell = Cell { tag: 10, values: [7, 8, 9] };
pub static REFS: [&'static u32; 3] = [&LIST[0], &LIST[1], &LIST[2]];
pub static ZERO: [(); 8589934593] = [(); 8589934593];
pub static HIGH: &'static () = &ZERO[8589934592];
