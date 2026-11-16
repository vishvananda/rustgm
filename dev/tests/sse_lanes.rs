//! The nine SIMD intrinsics of the 16-byte demand (`simd_eq`, `simd_gt`,
//! `simd_lt`, `simd_and`, `simd_or`, `simd_bitmask`, `simd_reduce_all`,
//! `simd_shuffle`, `simd_splat`) over a mini core that declares them as core
//! does, with stdarch-shaped `_mm_*` functions over `__m128i`.
//!
//! Every outcome was declared before either compiler ran, with rustc 1.98.0
//! (`RUSTC_BOOTSTRAP=1 rustc +1.98.0 --edition=2024 -Cpanic=abort
//! -Copt-level=0 --cap-lints=allow`, a staticlib linked by `cc`) as the
//! oracle. One declaration was wrong: `simd_reduce_all` was declared to read
//! each lane's low bit, and rustc prints `all 1 0 0 1 0 0` because it reads
//! the sign bit; this compiler follows rustc.
use rustgm::{expand::config::Config, pipeline, support::diagnostics::LintCap};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

const CORE: &str = r#"#![feature(no_core, lang_items, rustc_attrs, intrinsics, repr_simd, auto_traits)]
#![feature(const_trait_impl)]
#![no_core]
#![allow(internal_features, dead_code, unused_unsafe, non_camel_case_types)]
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
#[lang = "sync"]
pub unsafe auto trait Sync {}
#[lang = "freeze"]
pub unsafe auto trait Freeze {}
#[lang = "destruct"]
#[rustc_deny_explicit_impl]
pub const trait Destruct: PointeeSized {}
#[lang = "drop_glue"]
pub const unsafe fn drop_glue<T: PointeeSized>(_: &mut T)
where
    T: [const] Destruct,
{
}
#[lang = "neg"]
pub trait Neg {
    type Output;
    fn neg(self) -> Self::Output;
}
impl Neg for i8 {
    type Output = i8;
    fn neg(self) -> i8 {
        -self
    }
}
impl<T: Copy, const N: usize> Clone for [T; N] {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: Copy, const N: usize> Copy for [T; N] {}
impl Clone for i8 {
    fn clone(&self) -> Self {
        *self
    }
}
impl Copy for i8 {}
impl Clone for u8 {
    fn clone(&self) -> Self {
        *self
    }
}
impl Copy for u8 {}
impl Clone for i16 {
    fn clone(&self) -> Self {
        *self
    }
}
impl Copy for i16 {}
impl Clone for u32 {
    fn clone(&self) -> Self {
        *self
    }
}
impl Copy for u32 {}
impl Clone for i64 {
    fn clone(&self) -> Self {
        *self
    }
}
impl Copy for i64 {}
impl Clone for u64 {
    fn clone(&self) -> Self {
        *self
    }
}
impl Copy for u64 {}
impl Clone for f32 {
    fn clone(&self) -> Self {
        *self
    }
}
impl Copy for f32 {}
pub mod intrinsics {
    #[rustc_nounwind]
    #[rustc_intrinsic]
    pub const unsafe fn transmute<Src, Dst>(src: Src) -> Dst;
    pub mod simd {
        #[rustc_intrinsic]
        #[rustc_nounwind]
        pub const unsafe fn simd_eq<T, U>(x: T, y: T) -> U;
        #[rustc_intrinsic]
        #[rustc_nounwind]
        pub const unsafe fn simd_lt<T, U>(x: T, y: T) -> U;
        #[rustc_intrinsic]
        #[rustc_nounwind]
        pub const unsafe fn simd_gt<T, U>(x: T, y: T) -> U;
        #[rustc_intrinsic]
        #[rustc_nounwind]
        pub const unsafe fn simd_and<T>(x: T, y: T) -> T;
        #[rustc_intrinsic]
        #[rustc_nounwind]
        pub const unsafe fn simd_or<T>(x: T, y: T) -> T;
        #[rustc_intrinsic]
        #[rustc_nounwind]
        pub const unsafe fn simd_bitmask<T, U>(x: T) -> U;
        #[rustc_intrinsic]
        #[rustc_nounwind]
        pub const unsafe fn simd_reduce_all<T>(x: T) -> bool;
        #[rustc_intrinsic]
        #[rustc_nounwind]
        pub const unsafe fn simd_shuffle<T, U, V>(x: T, y: T, idx: U) -> V;
        #[rustc_nounwind]
        #[rustc_intrinsic]
        pub const unsafe fn simd_splat<T, U>(value: U) -> T;
    }
}
pub mod simd {
    #[repr(simd)]
    pub struct i8x16(pub [i8; 16]);
    #[repr(simd)]
    pub struct u8x16(pub [u8; 16]);
    #[repr(simd)]
    pub struct i64x2(pub [i64; 2]);
    #[repr(simd)]
    pub struct i16x8(pub [i16; 8]);
    #[repr(simd)]
    pub struct u16x8(pub [u16; 8]);
    #[repr(simd)]
    pub struct f32x4(pub [f32; 4]);
    #[repr(simd)]
    pub struct Idx16(pub [u32; 16]);
    impl crate::Clone for i8x16 {
        fn clone(&self) -> Self {
            *self
        }
    }
    impl crate::Copy for i8x16 {}
    impl crate::Clone for u8x16 {
        fn clone(&self) -> Self {
            *self
        }
    }
    impl crate::Copy for u8x16 {}
    impl crate::Clone for i64x2 {
        fn clone(&self) -> Self {
            *self
        }
    }
    impl crate::Copy for i64x2 {}
    impl crate::Clone for f32x4 {
        fn clone(&self) -> Self {
            *self
        }
    }
    impl crate::Copy for f32x4 {}
}
pub mod x86 {
    use crate::intrinsics::{simd::*, transmute};
    use crate::simd::i8x16;
    #[repr(simd)]
    pub struct __m128i([i64; 2]);
    impl crate::Clone for __m128i {
        fn clone(&self) -> Self {
            *self
        }
    }
    impl crate::Copy for __m128i {}
    impl __m128i {
        #[inline]
        pub const fn as_i8x16(self) -> i8x16 {
            unsafe { transmute(self) }
        }
    }
    impl i8x16 {
        #[inline]
        pub const fn as_m128i(self) -> __m128i {
            unsafe { transmute(self) }
        }
        pub const ZERO: Self = unsafe { simd_splat(0i8) };
    }
    #[inline(never)]
    pub const fn _mm_cmpeq_epi8(a: __m128i, b: __m128i) -> __m128i {
        unsafe { transmute::<i8x16, _>(simd_eq(a.as_i8x16(), b.as_i8x16())) }
    }
    #[inline(never)]
    pub const fn _mm_cmpgt_epi8(a: __m128i, b: __m128i) -> __m128i {
        unsafe { transmute::<i8x16, _>(simd_gt(a.as_i8x16(), b.as_i8x16())) }
    }
    #[inline(never)]
    pub const fn _mm_movemask_epi8(a: __m128i) -> i32 {
        unsafe {
            let z = i8x16::ZERO;
            let m: i8x16 = simd_lt(a.as_i8x16(), z);
            simd_bitmask::<_, u16>(m) as u32 as i32
        }
    }
    #[inline(never)]
    pub const fn _mm_or_si128(a: __m128i, b: __m128i) -> __m128i {
        unsafe { simd_or(a, b) }
    }
    #[inline(never)]
    pub const fn _mm_set1_epi8(a: i8) -> __m128i {
        unsafe { simd_splat::<i8x16, i8>(a) }.as_m128i()
    }
    #[inline(never)]
    pub const unsafe fn _mm_load_si128(mem_addr: *const __m128i) -> __m128i {
        *mem_addr
    }
    #[inline(never)]
    pub const unsafe fn _mm_store_si128(mem_addr: *mut __m128i, a: __m128i) {
        *mem_addr = a;
    }
}
"#;
const PROGRAM: &str = r#"pub mod program {
    use crate::intrinsics::{simd::*, transmute};
    use crate::simd::{Idx16, i64x2, i8x16, u8x16};
    use crate::x86::*;
    #[link(name = "c")]
    unsafe extern "C" {
        fn printf(format: *const u8, ...) -> i32;
    }
    #[repr(C)]
    struct Halves {
        lo: u64,
        hi: u64,
    }
    fn text<const N: usize>(bytes: &[u8; N]) -> *const u8 {
        bytes as *const [u8; N] as *const u8
    }
    const FIVES: i8x16 = unsafe { simd_splat(0x5a_i8) };
    #[inline(never)]
    fn halves(v: i8x16) -> Halves {
        unsafe { transmute(v) }
    }
    #[inline(never)]
    fn show(name: *const u8, v: i8x16) {
        let h = halves(v);
        unsafe {
            printf(text(b"%s %016llx %016llx\n\0"), name, h.hi, h.lo);
        }
    }
    #[inline(never)]
    fn show_mask(name: *const u8, v: i8x16) {
        let h = halves(v);
        let mask: u16 = unsafe { simd_bitmask(v) };
        unsafe {
            printf(
                text(b"%s %016llx %016llx %04x\n\0"),
                name,
                h.hi,
                h.lo,
                mask as u32,
            );
        }
    }
    #[inline(never)]
    fn eq(a: i8x16, b: i8x16) -> i8x16 {
        unsafe { simd_eq(a, b) }
    }
    #[inline(never)]
    fn all(v: i8x16) -> u32 {
        unsafe { simd_reduce_all(v) as u32 }
    }
    #[inline(never)]
    fn reversed(x: u8x16, y: u8x16) -> u8x16 {
        unsafe { simd_shuffle(x, y, const { Idx16([15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0]) }) }
    }
    #[inline(never)]
    fn zipped(x: u8x16, y: u8x16) -> u8x16 {
        unsafe { simd_shuffle(x, y, const { Idx16([0, 16, 1, 17, 2, 18, 3, 19, 4, 20, 5, 21, 6, 22, 7, 23]) }) }
    }
    #[inline(never)]
    fn lasts(x: u8x16, y: u8x16) -> u8x16 {
        unsafe { simd_shuffle(x, y, const { Idx16([31; 16]) }) }
    }
    #[unsafe(no_mangle)]
    pub extern "C" fn main() -> i32 {
        let a = i8x16([0, 1, -1, 127, -128, 5, 5, 0, 1, 2, 3, 4, -5, -6, 7, 8]);
        let b = i8x16([0, 2, -1, -128, 127, 5, 6, 0, 1, 2, 3, 4, -5, 6, -7, 8]);
        unsafe {
            show_mask(text(b"eq\0"), eq(a, b));
            show_mask(text(b"gt\0"), simd_gt(a, b));
            show_mask(text(b"lt\0"), simd_lt(a, b));
            let ua: u8x16 = transmute(a);
            let ub: u8x16 = transmute(b);
            show_mask(text(b"equ\0"), simd_eq::<u8x16, i8x16>(ua, ub));
            show(text(b"and\0"), simd_and(a, b));
            show(text(b"or\0"), simd_or(a, b));
            let wa: i64x2 = transmute(a);
            let wb: i64x2 = transmute(b);
            show(text(b"and64\0"), transmute(simd_and(wa, wb)));
            show(text(b"or64\0"), transmute(simd_or(wa, wb)));
            let high: u16 = simd_bitmask(a);
            let same: u16 = simd_bitmask(eq(a, a));
            let none: u16 = simd_bitmask(eq(a, simd_splat(100_i8)));
            printf(
                text(b"mask %04x %04x %04x\n\0"),
                high as u32,
                same as u32,
                none as u32,
            );
            let last = i8x16([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2]);
            let first = i8x16([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
            printf(
                text(b"all %u %u %u %u %u %u\n\0"),
                all(eq(a, a)),
                all(eq(a, b)),
                all(simd_splat(1_i8)),
                all(simd_splat(-2_i8)),
                all(last),
                all(first),
            );
            let s = halves(simd_splat(-3_i8));
            let t = halves(transmute(simd_splat::<u8x16, u8>(0x80)));
            printf(
                text(b"splat %016llx %016llx %016llx %016llx\n\0"),
                s.hi,
                s.lo,
                t.hi,
                t.lo,
            );
            let f = halves(FIVES);
            let z = halves(i8x16::ZERO);
            printf(
                text(b"const %016llx %016llx %016llx %016llx\n\0"),
                f.hi,
                f.lo,
                z.hi,
                z.lo,
            );
            show(text(b"rev\0"), transmute(reversed(ua, ub)));
            show(text(b"zip\0"), transmute(zipped(ua, ub)));
            show(text(b"last\0"), transmute(lasts(ua, ub)));
            let am = a.as_m128i();
            let bm = b.as_m128i();
            let equal = _mm_cmpeq_epi8(am, bm);
            show(text(b"cmpeq\0"), equal.as_i8x16());
            printf(
                text(b"movemask %d %d\n\0"),
                _mm_movemask_epi8(equal),
                _mm_movemask_epi8(am),
            );
            let special = _mm_cmpgt_epi8(i8x16::ZERO.as_m128i(), am);
            show(text(b"special\0"), special.as_i8x16());
            let converted = _mm_or_si128(special, _mm_set1_epi8(0x80_u8 as i8));
            show(text(b"convert\0"), converted.as_i8x16());
            let mut out = i8x16::ZERO.as_m128i();
            _mm_store_si128(&mut out, _mm_load_si128(&equal));
            show(text(b"stored\0"), out.as_i8x16());
        }
        0
    }
}
"#;
/// rustc's run of the program, byte for byte.
const RUSTC: &str = "eq ff0000ffffffffff ff00ff0000ff00ff 9fa5
gt 00ff000000000000 00000000ff000000 4008
lt 0000ff0000000000 00ff00ff0000ff00 2052
equ ff0000ffffffffff ff00ff0000ff00ff 9fa5
and 080102fb04030201 0004050000ff0000
or 08fffefb04030201 000705ffffff0300
and64 080102fb04030201 0004050000ff0000
or64 08fffefb04030201 000705ffffff0300
mask 3014 ffff 0000
all 1 0 0 1 0 0
splat fdfdfdfdfdfdfdfd fdfdfdfdfdfdfdfd 8080808080808080 8080808080808080
const 5a5a5a5a5a5a5a5a 5a5a5a5a5a5a5a5a 0000000000000000 0000000000000000
rev 0001ff7f80050500 01020304fbfa0708
zip 0000060505057f80 807fffff02010000
last 0808080808080808 0808080808080808
cmpeq ff0000ffffffffff ff00ff0000ff00ff
movemask 40869 12308
special 0000ffff00000000 000000ff00ff0000
convert 8080ffff80808080 808080ff80ff8080
stored ff0000ffffffffff ff00ff0000ff00ff
";

/// One crate lowered from its exports as an executable.
fn program(source: &str) -> Result<rustgm::lowir::program::Program> {
    let mut config = Config::default();
    config.set_lint_cap(LintCap::Allow);
    let report = crate::resolve::source_provider_config(source, "sse_lanes", &config)?;
    let analysis = pipeline::analyze_source(&report)?;
    let (mut facts, prepared) = (analysis.facts, analysis.prepared);
    let roots = rustgm::lower::Roots {
        entry: None,
        exports: &[],
    };
    Ok(rustgm::lower::instances::build_roots_prepared(
        &report, &mut facts, roots, prepared,
    )?)
}
/// The program with one exported control function after it.
fn control(body: &str) -> String {
    format!(
        "{CORE}{PROGRAM}pub mod control {{\n    use crate::intrinsics::simd::*;\n    use crate::simd::*;\n    #[unsafe(no_mangle)]\n{body}}}\n"
    )
}
fn failure(source: &str) -> String {
    program(source).map_or_else(|error| error.to_string(), |_| "accepted".to_owned())
}

/// rustc accepts the program; each intrinsic becomes the SSE2 operation its
/// LLVM lowering selects, and no call of a SIMD declaration remains.
#[test]
fn sixteen_byte_intrinsics_lower_to_sse2_vector_operations() -> Result {
    let mut text = Vec::new();
    rustgm::lowir::writer::write(&program(&format!("{CORE}{PROGRAM}"))?, &mut text)?;
    let text = String::from_utf8(text)?;
    for operation in [
        "vector pcmpeqb ",
        "vector pcmpgtb ",
        "vector pand ",
        "vector por ",
        "vector pmovmskb ",
        "vector fromi32 ",
        "vector pshufd ",
    ] {
        assert!(text.contains(operation), "{operation}\n{text}");
    }
    assert!(!text.contains("simd_"), "{text}");
    Ok(())
}

/// rustc rejects each control with E0511 (`eq-length`, `and-float`,
/// `bitmask-type`, `reduce-float`, `shuffle-length`, `shuffle-bound`,
/// `splat-type`) or its constant-index error (`shuffle-dynamic`), as its
/// results record; this compiler names the same rule without the types.
#[test]
fn shapes_rustc_rejects_are_rejected() {
    let cases = [
        (
            "    pub extern \"C\" fn bad(a: i8x16) -> i64x2 {\n        unsafe { simd_eq::<i8x16, i64x2>(a, a) }\n    }\n",
            "expected a return type with the input's length",
        ),
        (
            "    pub extern \"C\" fn bad(a: f32x4) -> f32x4 {\n        unsafe { simd_and(a, a) }\n    }\n",
            "unsupported operation on a vector of non-integer elements",
        ),
        (
            "    pub extern \"C\" fn bad(a: i8x16) -> u32 {\n        unsafe { simd_bitmask::<i8x16, u32>(a) }\n    }\n",
            "invalid monomorphization of simd_bitmask: cannot return this type",
        ),
        (
            "    pub extern \"C\" fn bad(a: f32x4) -> bool {\n        unsafe { simd_reduce_all(a) }\n    }\n",
            "unsupported operation on a vector of non-integer elements",
        ),
        (
            "    pub extern \"C\" fn bad(a: u8x16) -> i64x2 {\n        unsafe { simd_shuffle::<u8x16, Idx16, i64x2>(a, a, const { Idx16([0; 16]) }) }\n    }\n",
            "expected a return type with the input's length",
        ),
        (
            "    pub extern \"C\" fn bad(a: u8x16) -> u8x16 {\n        unsafe { simd_shuffle(a, a, const { Idx16([32; 16]) }) }\n    }\n",
            "a shuffle index is out of bounds",
        ),
        (
            "    pub extern \"C\" fn bad(v: i32) -> i8x16 {\n        unsafe { simd_splat::<i8x16, i32>(v) }\n    }\n",
            "the value's type is not the vector's element type",
        ),
        (
            "    pub fn bad(a: u8x16, i: Idx16) -> u8x16 {\n        unsafe { simd_shuffle(a, a, i) }\n    }\n",
            "3rd argument of `simd_shuffle` is required to be a `const` item",
        ),
    ];
    for (body, expected) in cases {
        let found = failure(&control(body));
        assert!(found.contains(expected), "{expected}\n{found}");
    }
}

/// rustc accepts both (`eq-i16x8`, `gt-unsigned`); outside the 16-byte
/// lowering they keep their named fences.
#[test]
fn shapes_outside_the_sixteen_byte_lowering_keep_named_fences() {
    for (body, fence) in [
        (
            "    pub extern \"C\" fn fenced(a: i16x8, b: i16x8) -> i16x8 {\n        unsafe { simd_eq(a, b) }\n    }\n",
            "simd_eq lowering covers sixteen 8-bit integer lanes",
        ),
        (
            "    pub extern \"C\" fn fenced(a: u8x16) -> u8x16 {\n        unsafe { simd_gt(a, a) }\n    }\n",
            "simd_gt lowering covers sixteen signed 8-bit lanes",
        ),
    ] {
        let found = failure(&control(body));
        assert!(found.contains(fence), "{fence}\n{found}");
    }
}

/// rustc's run of the program (`program` in its results).
#[test]
#[ignore = "requires the pinned CPPGM backend and a C compiler"]
fn sixteen_byte_programs_print_what_rustcs_print() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let folder = root.join(format!("obj/parallel/sse-lanes/run-{}", std::process::id()));
    std::fs::create_dir_all(&folder)?;
    let built = program(&format!("{CORE}{PROGRAM}"))?;
    let client = "/* The Rust program exports `main`. */\n";
    crate::ffi_program::check_status(&root, &folder, "program", &built, client, RUSTC, 0)?;
    Ok(())
}
