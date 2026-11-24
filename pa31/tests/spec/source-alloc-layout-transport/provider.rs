#![feature(no_core, lang_items, rustc_attrs, intrinsics, auto_traits)]
#![no_core]
#![allow(internal_features)]

#[lang = "pointee_sized"]
pub trait PointeeSized {}
#[lang = "meta_sized"]
pub trait MetaSized: PointeeSized {}
#[lang = "sized"]
pub trait Sized: MetaSized {}
#[lang = "legacy_receiver"]
pub trait LegacyReceiver: PointeeSized {}
impl<T: PointeeSized> LegacyReceiver for &T {}
impl<T: PointeeSized> LegacyReceiver for &mut T {}
#[lang = "clone"]
pub trait Clone: Sized {
    #[lang = "clone_fn"]
    fn clone(&self) -> Self;
}
#[lang = "copy"]
pub trait Copy: Clone {}
#[lang = "freeze"]
pub unsafe auto trait Freeze {}

// Reduced from library/core/src/alloc/layout.rs and core/src/mem/alignment.rs.
// Layout declaration from alloc-layout-identity/original-layout.rs.
#[repr(transparent)]
pub struct Alignment {
    _inner_repr_trick: AlignmentEnum,
}
#[repr(usize)]
enum AlignmentEnum {
    _Align1Shl0 = 1,
    _Align1Shl1 = 2,
    _Align1Shl2 = 4,
    _Align1Shl3 = 8,
    _Align1Shl4 = 16,
    _Align1Shl5 = 32,
    _Align1Shl6 = 64,
    _Align1Shl7 = 128,
    _Align1Shl8 = 256,
    _Align1Shl9 = 512,
    _Align1Shl10 = 1024,
    _Align1Shl11 = 2048,
    _Align1Shl12 = 4096,
    _Align1Shl13 = 8192,
    _Align1Shl14 = 16384,
    _Align1Shl15 = 32768,
    _Align1Shl16 = 65536,
    _Align1Shl17 = 131072,
    _Align1Shl18 = 262144,
    _Align1Shl19 = 524288,
    _Align1Shl20 = 1048576,
    _Align1Shl21 = 2097152,
    _Align1Shl22 = 4194304,
    _Align1Shl23 = 8388608,
    _Align1Shl24 = 16777216,
    _Align1Shl25 = 33554432,
    _Align1Shl26 = 67108864,
    _Align1Shl27 = 134217728,
    _Align1Shl28 = 268435456,
    _Align1Shl29 = 536870912,
    _Align1Shl30 = 1073741824,
    _Align1Shl31 = 2147483648,
    _Align1Shl32 = 4294967296,
    _Align1Shl33 = 8589934592,
    _Align1Shl34 = 17179869184,
    _Align1Shl35 = 34359738368,
    _Align1Shl36 = 68719476736,
    _Align1Shl37 = 137438953472,
    _Align1Shl38 = 274877906944,
    _Align1Shl39 = 549755813888,
    _Align1Shl40 = 1099511627776,
    _Align1Shl41 = 2199023255552,
    _Align1Shl42 = 4398046511104,
    _Align1Shl43 = 8796093022208,
    _Align1Shl44 = 17592186044416,
    _Align1Shl45 = 35184372088832,
    _Align1Shl46 = 70368744177664,
    _Align1Shl47 = 140737488355328,
    _Align1Shl48 = 281474976710656,
    _Align1Shl49 = 562949953421312,
    _Align1Shl50 = 1125899906842624,
    _Align1Shl51 = 2251799813685248,
    _Align1Shl52 = 4503599627370496,
    _Align1Shl53 = 9007199254740992,
    _Align1Shl54 = 18014398509481984,
    _Align1Shl55 = 36028797018963968,
    _Align1Shl56 = 72057594037927936,
    _Align1Shl57 = 144115188075855872,
    _Align1Shl58 = 288230376151711744,
    _Align1Shl59 = 576460752303423488,
    _Align1Shl60 = 1152921504606846976,
    _Align1Shl61 = 2305843009213693952,
    _Align1Shl62 = 4611686018427387904,
    _Align1Shl63 = 9223372036854775808,
}

#[rustc_nounwind]
#[rustc_intrinsic_const_stable_indirect]
#[rustc_intrinsic]
pub const fn size_of<T>() -> usize;
#[rustc_nounwind]
#[rustc_intrinsic_const_stable_indirect]
#[rustc_intrinsic]
pub const fn align_of<T>() -> usize;
