fn check(ok: bool) { if !ok { let _ = failure(0); } }
fn failure(zero: i32) -> i32 { 1 / zero }
fn to_i8(x:f64)->i8{x as i8}
fn to_i16(x:f64)->i16{x as i16}
fn to_i32(x:f64)->i32{x as i32}
fn to_i64(x:f64)->i64{x as i64}
fn to_i128(x:f64)->i128{x as i128}
fn to_u8(x:f64)->u8{x as u8}
fn to_u16(x:f64)->u16{x as u16}
fn to_u32(x:f64)->u32{x as u32}
fn to_u64(x:f64)->u64{x as u64}
fn to_u128(x:f64)->u128{x as u128}
fn main() {
    check(to_i8(1.0/0.0)==127);
    check(to_i8(-1.0/0.0)==-128);
    check(to_i8(0.0/0.0)==0);
    check(to_i8(73.9)==73);
    check(to_i8(-73.9)==-73);
    check(to_i8(0.0)==0);
    check(to_i8(-0.0)==0);
    check(to_i16(1.0/0.0)==32767);
    check(to_i16(-1.0/0.0)==-32768);
    check(to_i16(0.0/0.0)==0);
    check(to_i16(73.9)==73);
    check(to_i16(-73.9)==-73);
    check(to_i16(0.0)==0);
    check(to_i16(-0.0)==0);
    check(to_i32(1.0/0.0)==2147483647);
    check(to_i32(-1.0/0.0)==-2147483648);
    check(to_i32(0.0/0.0)==0);
    check(to_i32(73.9)==73);
    check(to_i32(-73.9)==-73);
    check(to_i32(0.0)==0);
    check(to_i32(-0.0)==0);
    check(to_i64(1.0/0.0)==9223372036854775807);
    check(to_i64(-1.0/0.0)==-9223372036854775808);
    check(to_i64(0.0/0.0)==0);
    check(to_i64(73.9)==73);
    check(to_i64(-73.9)==-73);
    check(to_i64(0.0)==0);
    check(to_i64(-0.0)==0);
    check(to_i128(1.0/0.0)==170141183460469231731687303715884105727);
    check(to_i128(-1.0/0.0)==-170141183460469231731687303715884105728);
    check(to_i128(0.0/0.0)==0);
    check(to_i128(73.9)==73);
    check(to_i128(-73.9)==-73);
    check(to_i128(0.0)==0);
    check(to_i128(-0.0)==0);
    check(to_u8(1.0/0.0)==255);
    check(to_u8(-1.0/0.0)==0);
    check(to_u8(0.0/0.0)==0);
    check(to_u8(73.9)==73);
    check(to_u8(-73.9)==0);
    check(to_u8(0.0)==0);
    check(to_u8(-0.0)==0);
    check(to_u16(1.0/0.0)==65535);
    check(to_u16(-1.0/0.0)==0);
    check(to_u16(0.0/0.0)==0);
    check(to_u16(73.9)==73);
    check(to_u16(-73.9)==0);
    check(to_u16(0.0)==0);
    check(to_u16(-0.0)==0);
    check(to_u32(1.0/0.0)==4294967295);
    check(to_u32(-1.0/0.0)==0);
    check(to_u32(0.0/0.0)==0);
    check(to_u32(73.9)==73);
    check(to_u32(-73.9)==0);
    check(to_u32(0.0)==0);
    check(to_u32(-0.0)==0);
    check(to_u64(1.0/0.0)==18446744073709551615);
    check(to_u64(-1.0/0.0)==0);
    check(to_u64(0.0/0.0)==0);
    check(to_u64(73.9)==73);
    check(to_u64(-73.9)==0);
    check(to_u64(0.0)==0);
    check(to_u64(-0.0)==0);
    check(to_u128(1.0/0.0)==340282366920938463463374607431768211455);
    check(to_u128(-1.0/0.0)==0);
    check(to_u128(0.0/0.0)==0);
    check(to_u128(73.9)==73);
    check(to_u128(-73.9)==0);
    check(to_u128(0.0)==0);
    check(to_u128(-0.0)==0);
}
