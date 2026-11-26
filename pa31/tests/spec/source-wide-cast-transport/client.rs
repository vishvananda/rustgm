#![feature(no_core,f16,f128)]
#![no_core]
extern crate base;

const SINGLE: u32 = unsafe { base::transmute_unchecked::<f32,u32>(base::half_to_single(1.5f16)) };
#[unsafe(no_mangle)]
pub extern "C" fn obs_single() -> u32 { SINGLE }

const DOUBLE: u64 = unsafe { base::transmute_unchecked::<f64,u64>(base::half_to_double(0.000000059604644775390625f16)) };
#[unsafe(no_mangle)]
pub extern "C" fn obs_double() -> u64 { DOUBLE }

const HALF_TIE: u16 = unsafe { base::transmute_unchecked::<f16,u16>(base::single_to_half(1.00048828125f32)) };
#[unsafe(no_mangle)]
pub extern "C" fn obs_half_tie() -> u16 { HALF_TIE }

const HALF_ROUND: u16 = unsafe { base::transmute_unchecked::<f16,u16>(base::double_to_half(1.000732421875f64)) };
#[unsafe(no_mangle)]
pub extern "C" fn obs_half_round() -> u16 { HALF_ROUND }

const QUAD_ROUND_SINGLE: u32 = unsafe { base::transmute_unchecked::<f32,u32>(base::quad_to_single(1.0000000596046447753906250000000001925929944387235853055977942584927318538101648215388195239938795566558837890625f128)) };
#[unsafe(no_mangle)]
pub extern "C" fn obs_quad_round_single() -> u32 { QUAD_ROUND_SINGLE }

const QUAD_ROUND_DOUBLE: u64 = unsafe { base::transmute_unchecked::<f64,u64>(base::quad_to_double(1.0000000000000000000000000000000001925929944387235853055977942584927318538101648215388195239938795566558837890625f128)) };
#[unsafe(no_mangle)]
pub extern "C" fn obs_quad_round_double() -> u64 { QUAD_ROUND_DOUBLE }

const DOUBLE_QUAD_LOW: u64 = { let [low,_] = unsafe { base::transmute_unchecked::<f128,[u64;2]>(base::double_to_quad(1.5f64)) }; low };
#[unsafe(no_mangle)]
pub extern "C" fn obs_double_quad_low() -> u64 { DOUBLE_QUAD_LOW }

const DOUBLE_QUAD_HIGH: u64 = { let [_,high] = unsafe { base::transmute_unchecked::<f128,[u64;2]>(base::double_to_quad(1.5f64)) }; high };
#[unsafe(no_mangle)]
pub extern "C" fn obs_double_quad_high() -> u64 { DOUBLE_QUAD_HIGH }

const HALF_QUAD_LOW: u64 = { let [low,_] = unsafe { base::transmute_unchecked::<f128,[u64;2]>(base::half_to_quad(1.5f16)) }; low };
#[unsafe(no_mangle)]
pub extern "C" fn obs_half_quad_low() -> u64 { HALF_QUAD_LOW }

const HALF_QUAD_HIGH: u64 = { let [_,high] = unsafe { base::transmute_unchecked::<f128,[u64;2]>(base::half_to_quad(1.5f16)) }; high };
#[unsafe(no_mangle)]
pub extern "C" fn obs_half_quad_high() -> u64 { HALF_QUAD_HIGH }

const QUAD_HALF: u16 = unsafe { base::transmute_unchecked::<f16,u16>(base::quad_to_half(1.5f128)) };
#[unsafe(no_mangle)]
pub extern "C" fn obs_quad_half() -> u16 { QUAD_HALF }

const SIGNED: i32 = base::quad_to_int(-3.75f128);
#[unsafe(no_mangle)]
pub extern "C" fn obs_signed() -> i32 { SIGNED }

const NAN: i32 = base::quad_to_int(unsafe { base::transmute_unchecked::<u128,f128>(0x7fff8000000000000000000000000001u128) });
#[unsafe(no_mangle)]
pub extern "C" fn obs_nan() -> i32 { NAN }

const SATURATED: u8 = base::half_to_byte(300.0f16);
#[unsafe(no_mangle)]
pub extern "C" fn obs_saturated() -> u8 { SATURATED }

const INTEGER_HALF: u16 = unsafe { base::transmute_unchecked::<f16,u16>(base::integer_to_half(2049u32)) };
#[unsafe(no_mangle)]
pub extern "C" fn obs_integer_half() -> u16 { INTEGER_HALF }

const INTEGER_QUAD_LOW: u64 = { let [low,_] = unsafe { base::transmute_unchecked::<f128,[u64;2]>(base::integer_to_quad(170141183460469231731687303715884105729u128)) }; low };
#[unsafe(no_mangle)]
pub extern "C" fn obs_integer_quad_low() -> u64 { INTEGER_QUAD_LOW }

const INTEGER_QUAD_HIGH: u64 = { let [_,high] = unsafe { base::transmute_unchecked::<f128,[u64;2]>(base::integer_to_quad(170141183460469231731687303715884105729u128)) }; high };
#[unsafe(no_mangle)]
pub extern "C" fn obs_integer_quad_high() -> u64 { INTEGER_QUAD_HIGH }

const QUAD_INTEGER_LOW: u64 = { let [low,_] = unsafe { base::transmute_unchecked::<u128,[u64;2]>(base::quad_to_integer(170141183460469231731687303715884105728.0f128)) }; low };
#[unsafe(no_mangle)]
pub extern "C" fn obs_quad_integer_low() -> u64 { QUAD_INTEGER_LOW }

const QUAD_INTEGER_HIGH: u64 = { let [_,high] = unsafe { base::transmute_unchecked::<u128,[u64;2]>(base::quad_to_integer(170141183460469231731687303715884105728.0f128)) }; high };
#[unsafe(no_mangle)]
pub extern "C" fn obs_quad_integer_high() -> u64 { QUAD_INTEGER_HIGH }

const GENERIC: u32 = unsafe { base::transmute_unchecked::<f32,u32>(base::generic_half::<u8>(1.5f16)) };
#[unsafe(no_mangle)]
pub extern "C" fn obs_generic() -> u32 { GENERIC }

const POSITIVE_INF: i32 = base::quad_to_int(unsafe { base::transmute_unchecked::<u128,f128>(0x7fff0000000000000000000000000000u128) });
#[unsafe(no_mangle)]
pub extern "C" fn obs_positive_inf() -> i32 { POSITIVE_INF }

const NEGATIVE_INF: i32 = base::quad_to_int(unsafe { base::transmute_unchecked::<u128,f128>(0xffff0000000000000000000000000000u128) });
#[unsafe(no_mangle)]
pub extern "C" fn obs_negative_inf() -> i32 { NEGATIVE_INF }

const NEGATIVE_BYTE: u8 = base::quad_to_byte(-3.75f128);
#[unsafe(no_mangle)]
pub extern "C" fn obs_negative_byte() -> u8 { NEGATIVE_BYTE }

const SIGNED_INTEGER_HALF: u16 = unsafe { base::transmute_unchecked::<f16,u16>(base::integer_to_half_signed(base::transmute_unchecked::<u128,i128>(0x80000000000000000000000000000000u128))) };
#[unsafe(no_mangle)]
pub extern "C" fn obs_signed_integer_half() -> u16 { SIGNED_INTEGER_HALF }

const QUAD_SIGNED_LOW: u64 = { let [low,_] = unsafe { base::transmute_unchecked::<i128,[u64;2]>(base::quad_to_signed(-170141183460469231731687303715884105728.0f128)) }; low };
#[unsafe(no_mangle)]
pub extern "C" fn obs_quad_signed_low() -> u64 { QUAD_SIGNED_LOW }

const QUAD_SIGNED_HIGH: u64 = { let [_,high] = unsafe { base::transmute_unchecked::<i128,[u64;2]>(base::quad_to_signed(-170141183460469231731687303715884105728.0f128)) }; high };
#[unsafe(no_mangle)]
pub extern "C" fn obs_quad_signed_high() -> u64 { QUAD_SIGNED_HIGH }

const QUAD_SATURATE_LOW: u64 = { let [low,_] = unsafe { base::transmute_unchecked::<u128,[u64;2]>(base::quad_to_integer(base::transmute_unchecked::<u128,f128>(0x7ffeffffffffffffffffffffffffffffu128))) }; low };
#[unsafe(no_mangle)]
pub extern "C" fn obs_quad_saturate_low() -> u64 { QUAD_SATURATE_LOW }

const QUAD_SATURATE_HIGH: u64 = { let [_,high] = unsafe { base::transmute_unchecked::<u128,[u64;2]>(base::quad_to_integer(base::transmute_unchecked::<u128,f128>(0x7ffeffffffffffffffffffffffffffffu128))) }; high };
#[unsafe(no_mangle)]
pub extern "C" fn obs_quad_saturate_high() -> u64 { QUAD_SATURATE_HIGH }

const SUBNORMAL_TIE: u16 = unsafe { base::transmute_unchecked::<f16,u16>(base::quad_to_half(0.0000000298023223876953125f128)) };
#[unsafe(no_mangle)]
pub extern "C" fn obs_subnormal_tie() -> u16 { SUBNORMAL_TIE }

const SUBNORMAL_ABOVE: u16 = unsafe { base::transmute_unchecked::<f16,u16>(base::quad_to_half(2.98023223876953125000000001925929944387235853055977942584927318538101648215388195239938795566558837890625E-8f128)) };
#[unsafe(no_mangle)]
pub extern "C" fn obs_subnormal_above() -> u16 { SUBNORMAL_ABOVE }

const NEGATIVE_HALF: u16 = unsafe { base::transmute_unchecked::<f16,u16>(base::quad_to_half(-1.5f128)) };
#[unsafe(no_mangle)]
pub extern "C" fn obs_negative_half() -> u16 { NEGATIVE_HALF }

const NEGATIVE_ZERO_LOW: u64 = { let [low,_] = unsafe { base::transmute_unchecked::<f128,[u64;2]>(base::half_to_quad(base::transmute_unchecked::<u16,f16>(0x8000u16))) }; low };
#[unsafe(no_mangle)]
pub extern "C" fn obs_negative_zero_low() -> u64 { NEGATIVE_ZERO_LOW }

const NEGATIVE_ZERO_HIGH: u64 = { let [_,high] = unsafe { base::transmute_unchecked::<f128,[u64;2]>(base::half_to_quad(base::transmute_unchecked::<u16,f16>(0x8000u16))) }; high };
#[unsafe(no_mangle)]
pub extern "C" fn obs_negative_zero_high() -> u64 { NEGATIVE_ZERO_HIGH }

