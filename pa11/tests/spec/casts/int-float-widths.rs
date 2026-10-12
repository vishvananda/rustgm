fn check(ok: bool) { if !ok { let _ = failure(0); } }
fn failure(zero: i32) -> i32 { 1 / zero }
fn main() {
    check(73i128 as f32 == 73.0);
    check((-73i128) as f64 == -73.0);
    check(18446744073709551616u128 as f64 == 18446744073709551616.0);
    check(18446744073709551615u64 as f32 == 18446744073709551616.0);
    check(1.25f32 as f64 == 1.25);
    check(1.25f64 as f32 == 1.25);
    check(true as i128 == 1);
    check(false as u8 == 0);
    check('🦀' as u32 == 129408);
    check(65u8 as char == 'A');
}
