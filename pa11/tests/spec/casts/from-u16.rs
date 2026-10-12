fn check(ok: bool) { if !ok { let _ = failure(0); } }
fn failure(zero: i32) -> i32 { 1 / zero }
fn to_i8(x:u16)->i8{x as i8}
fn to_i16(x:u16)->i16{x as i16}
fn to_i32(x:u16)->i32{x as i32}
fn to_i64(x:u16)->i64{x as i64}
fn to_i128(x:u16)->i128{x as i128}
fn to_u8(x:u16)->u8{x as u8}
fn to_u16(x:u16)->u16{x as u16}
fn to_u32(x:u16)->u32{x as u32}
fn to_u64(x:u16)->u64{x as u64}
fn to_u128(x:u16)->u128{x as u128}
fn main() {
    check(to_i8(0)==0);
    check(to_i8(65535)==-1);
    check(to_i8(0)==0);
    check(to_i8(73)==73);
    check(to_i16(0)==0);
    check(to_i16(65535)==-1);
    check(to_i16(0)==0);
    check(to_i16(73)==73);
    check(to_i32(0)==0);
    check(to_i32(65535)==65535);
    check(to_i32(0)==0);
    check(to_i32(73)==73);
    check(to_i64(0)==0);
    check(to_i64(65535)==65535);
    check(to_i64(0)==0);
    check(to_i64(73)==73);
    check(to_i128(0)==0);
    check(to_i128(65535)==65535);
    check(to_i128(0)==0);
    check(to_i128(73)==73);
    check(to_u8(0)==0);
    check(to_u8(65535)==255);
    check(to_u8(0)==0);
    check(to_u8(73)==73);
    check(to_u16(0)==0);
    check(to_u16(65535)==65535);
    check(to_u16(0)==0);
    check(to_u16(73)==73);
    check(to_u32(0)==0);
    check(to_u32(65535)==65535);
    check(to_u32(0)==0);
    check(to_u32(73)==73);
    check(to_u64(0)==0);
    check(to_u64(65535)==65535);
    check(to_u64(0)==0);
    check(to_u64(73)==73);
    check(to_u128(0)==0);
    check(to_u128(65535)==65535);
    check(to_u128(0)==0);
    check(to_u128(73)==73);
}
