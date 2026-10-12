fn check(ok: bool) { if !ok { let _ = failure(0); } }
fn failure(zero: i32) -> i32 { 1 / zero }
fn to_i8(x:i8)->i8{x as i8}
fn to_i16(x:i8)->i16{x as i16}
fn to_i32(x:i8)->i32{x as i32}
fn to_i64(x:i8)->i64{x as i64}
fn to_i128(x:i8)->i128{x as i128}
fn to_u8(x:i8)->u8{x as u8}
fn to_u16(x:i8)->u16{x as u16}
fn to_u32(x:i8)->u32{x as u32}
fn to_u64(x:i8)->u64{x as u64}
fn to_u128(x:i8)->u128{x as u128}
fn main() {
    check(to_i8(-128)==-128);
    check(to_i8(127)==127);
    check(to_i8(0)==0);
    check(to_i8(73)==73);
    check(to_i16(-128)==-128);
    check(to_i16(127)==127);
    check(to_i16(0)==0);
    check(to_i16(73)==73);
    check(to_i32(-128)==-128);
    check(to_i32(127)==127);
    check(to_i32(0)==0);
    check(to_i32(73)==73);
    check(to_i64(-128)==-128);
    check(to_i64(127)==127);
    check(to_i64(0)==0);
    check(to_i64(73)==73);
    check(to_i128(-128)==-128);
    check(to_i128(127)==127);
    check(to_i128(0)==0);
    check(to_i128(73)==73);
    check(to_u8(-128)==128);
    check(to_u8(127)==127);
    check(to_u8(0)==0);
    check(to_u8(73)==73);
    check(to_u16(-128)==65408);
    check(to_u16(127)==127);
    check(to_u16(0)==0);
    check(to_u16(73)==73);
    check(to_u32(-128)==4294967168);
    check(to_u32(127)==127);
    check(to_u32(0)==0);
    check(to_u32(73)==73);
    check(to_u64(-128)==18446744073709551488);
    check(to_u64(127)==127);
    check(to_u64(0)==0);
    check(to_u64(73)==73);
    check(to_u128(-128)==340282366920938463463374607431768211328);
    check(to_u128(127)==127);
    check(to_u128(0)==0);
    check(to_u128(73)==73);
}
