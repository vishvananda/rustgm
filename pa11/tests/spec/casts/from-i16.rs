fn check(ok: bool) { if !ok { let _ = failure(0); } }
fn failure(zero: i32) -> i32 { 1 / zero }
fn to_i8(x:i16)->i8{x as i8}
fn to_i16(x:i16)->i16{x as i16}
fn to_i32(x:i16)->i32{x as i32}
fn to_i64(x:i16)->i64{x as i64}
fn to_i128(x:i16)->i128{x as i128}
fn to_u8(x:i16)->u8{x as u8}
fn to_u16(x:i16)->u16{x as u16}
fn to_u32(x:i16)->u32{x as u32}
fn to_u64(x:i16)->u64{x as u64}
fn to_u128(x:i16)->u128{x as u128}
fn main() {
    check(to_i8(-32768)==0);
    check(to_i8(32767)==-1);
    check(to_i8(0)==0);
    check(to_i8(73)==73);
    check(to_i16(-32768)==-32768);
    check(to_i16(32767)==32767);
    check(to_i16(0)==0);
    check(to_i16(73)==73);
    check(to_i32(-32768)==-32768);
    check(to_i32(32767)==32767);
    check(to_i32(0)==0);
    check(to_i32(73)==73);
    check(to_i64(-32768)==-32768);
    check(to_i64(32767)==32767);
    check(to_i64(0)==0);
    check(to_i64(73)==73);
    check(to_i128(-32768)==-32768);
    check(to_i128(32767)==32767);
    check(to_i128(0)==0);
    check(to_i128(73)==73);
    check(to_u8(-32768)==0);
    check(to_u8(32767)==255);
    check(to_u8(0)==0);
    check(to_u8(73)==73);
    check(to_u16(-32768)==32768);
    check(to_u16(32767)==32767);
    check(to_u16(0)==0);
    check(to_u16(73)==73);
    check(to_u32(-32768)==4294934528);
    check(to_u32(32767)==32767);
    check(to_u32(0)==0);
    check(to_u32(73)==73);
    check(to_u64(-32768)==18446744073709518848);
    check(to_u64(32767)==32767);
    check(to_u64(0)==0);
    check(to_u64(73)==73);
    check(to_u128(-32768)==340282366920938463463374607431768178688);
    check(to_u128(32767)==32767);
    check(to_u128(0)==0);
    check(to_u128(73)==73);
}
