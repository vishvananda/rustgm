fn check(ok: bool) { if !ok { let _ = failure(0); } }
fn failure(zero: i32) -> i32 { 1 / zero }
fn to_i8(x:u32)->i8{x as i8}
fn to_i16(x:u32)->i16{x as i16}
fn to_i32(x:u32)->i32{x as i32}
fn to_i64(x:u32)->i64{x as i64}
fn to_i128(x:u32)->i128{x as i128}
fn to_u8(x:u32)->u8{x as u8}
fn to_u16(x:u32)->u16{x as u16}
fn to_u32(x:u32)->u32{x as u32}
fn to_u64(x:u32)->u64{x as u64}
fn to_u128(x:u32)->u128{x as u128}
fn main() {
    check(to_i8(0)==0);
    check(to_i8(4294967295)==-1);
    check(to_i8(0)==0);
    check(to_i8(73)==73);
    check(to_i16(0)==0);
    check(to_i16(4294967295)==-1);
    check(to_i16(0)==0);
    check(to_i16(73)==73);
    check(to_i32(0)==0);
    check(to_i32(4294967295)==-1);
    check(to_i32(0)==0);
    check(to_i32(73)==73);
    check(to_i64(0)==0);
    check(to_i64(4294967295)==4294967295);
    check(to_i64(0)==0);
    check(to_i64(73)==73);
    check(to_i128(0)==0);
    check(to_i128(4294967295)==4294967295);
    check(to_i128(0)==0);
    check(to_i128(73)==73);
    check(to_u8(0)==0);
    check(to_u8(4294967295)==255);
    check(to_u8(0)==0);
    check(to_u8(73)==73);
    check(to_u16(0)==0);
    check(to_u16(4294967295)==65535);
    check(to_u16(0)==0);
    check(to_u16(73)==73);
    check(to_u32(0)==0);
    check(to_u32(4294967295)==4294967295);
    check(to_u32(0)==0);
    check(to_u32(73)==73);
    check(to_u64(0)==0);
    check(to_u64(4294967295)==4294967295);
    check(to_u64(0)==0);
    check(to_u64(73)==73);
    check(to_u128(0)==0);
    check(to_u128(4294967295)==4294967295);
    check(to_u128(0)==0);
    check(to_u128(73)==73);
}
