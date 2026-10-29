#![feature(location_fixture)]
#![feature(intrinsics,pattern_types,decl_macro,marker_trait_attr,freeze_impls,allow_internal_unstable,staged_api)]
#![unstable(feature="location_fixture",issue="none")]
#![rustc_coherence_is_core]#![feature(no_core,lang_items,rustc_attrs,const_trait_impl,const_destruct,fundamental,auto_traits)]
#![no_core]
#![allow(internal_features,dead_code,unused_variables)]
#[lang = "pointee_sized"]
#[diagnostic::on_unimplemented(
    message = "values of type `{Self}` may or may not have a size",
    label = "may or may not have a known size"
)]
#[fundamental]
#[rustc_specialization_trait]
#[rustc_deny_explicit_impl]
#[rustc_coinductive]
pub trait PointeeSized {
    // Empty
}
#[lang = "meta_sized"]
#[diagnostic::on_unimplemented(
    message = "the size for values of type `{Self}` cannot be known",
    label = "doesn't have a known size"
)]
#[fundamental]
#[rustc_specialization_trait]
#[rustc_deny_explicit_impl]
// `MetaSized` being coinductive, despite having supertraits, is okay for the same reasons as
// `Sized` above.
#[rustc_coinductive]
pub trait MetaSized: PointeeSized {
    // Empty
}
#[lang = "sized"]
#[diagnostic::on_unimplemented(
    message = "the size for values of type `{Self}` cannot be known at compilation time",
    label = "doesn't have a size known at compile-time"
)]
#[fundamental] // for Default, for example, which requires that `[T]: !Default` be evaluatable
#[rustc_specialization_trait]
#[rustc_deny_explicit_impl]
#[rustc_dyn_incompatible_trait]
// `Sized` being coinductive, despite having supertraits, is okay as there are no user-written impls,
// and we know that the supertraits are always implemented if the subtrait is just by looking at
// the builtin impls.
#[rustc_coinductive]
pub trait Sized: MetaSized {
    // Empty.
}
#[lang = "legacy_receiver"]
#[doc(hidden)]
pub trait LegacyReceiver: PointeeSized {
    // Empty.
}

impl<T: PointeeSized> LegacyReceiver for &T {}

impl<T: PointeeSized> LegacyReceiver for &mut T {}
#[unstable(feature = "const_destruct", issue = "133214")]
#[rustc_const_unstable(feature = "const_destruct", issue = "133214")]
#[lang = "destruct"]
#[diagnostic::on_unimplemented(message = "can't drop `{Self}`")]
#[rustc_deny_explicit_impl]
#[rustc_dyn_incompatible_trait]
pub const trait Destruct: PointeeSized {}
#[lang = "freeze"]
pub unsafe auto trait Freeze {}
#[stable(feature = "rust1", since = "1.0.0")]
#[lang = "clone"]
#[rustc_diagnostic_item = "Clone"]
#[rustc_trivial_field_reads]
#[rustc_const_unstable(feature = "const_clone", issue = "142757")]
pub const trait Clone: Sized {
    /// Returns a duplicate of the value.
    ///
    /// Note that what "duplicate" means varies by type:
    /// - For most types, this creates a deep, independent copy
    /// - For reference types like `&T`, this creates another reference to the same value
    /// - For smart pointers like [`Arc`] or [`Rc`], this increments the reference count
    ///   but still points to the same underlying data
    ///
    /// [`Arc`]: ../../std/sync/struct.Arc.html
    /// [`Rc`]: ../../std/rc/struct.Rc.html
    ///
    /// # Examples
    ///
    /// ```
    /// # #![allow(noop_method_call)]
    /// let hello = "Hello"; // &str implements Clone
    ///
    /// assert_eq!("Hello", hello.clone());
    /// ```
    ///
    /// Example with a reference-counted type:
    ///
    /// ```
    /// use std::sync::{Arc, Mutex};
    ///
    /// let data = Arc::new(Mutex::new(vec![1, 2, 3]));
    /// let data_clone = data.clone(); // Creates another Arc pointing to the same Mutex
    ///
    /// {
    ///     let mut lock = data.lock().unwrap();
    ///     lock.push(4);
    /// }
    ///
    /// // Changes are visible through the clone because they share the same underlying data
    /// assert_eq!(*data_clone.lock().unwrap(), vec![1, 2, 3, 4]);
    /// ```
    #[must_use = "cloning is often expensive and is not expected to have side effects"]
    // Clone::clone is special because the compiler generates MIR to implement it for some types.
    // See InstanceKind::CloneShim.
    #[stable(feature = "rust1", since = "1.0.0")]
    #[lang = "clone_fn"]
    fn clone(&self) -> Self;

    /// Performs copy-assignment from `source`.
    ///
    /// `a.clone_from(&b)` is equivalent to `a = b.clone()` in functionality,
    /// but can be overridden to reuse the resources of `a` to avoid unnecessary
    /// allocations.
    #[inline]
    #[stable(feature = "rust1", since = "1.0.0")]
    fn clone_from(&mut self, source: &Self)
    where
        Self: [const] Destruct,
    {
        *self = source.clone()
    }
}
#[stable(feature = "rust1", since = "1.0.0")]
#[lang = "copy"]
#[rustc_diagnostic_item = "Copy"]
pub trait Copy: Clone {
    // Empty.
}
impl Clone for u8{fn clone(&self)->Self{*self}}impl Copy for u8{}
impl Clone for u16{fn clone(&self)->Self{*self}}impl Copy for u16{}
impl Clone for u32{fn clone(&self)->Self{*self}}impl Copy for u32{}
impl Clone for u64{fn clone(&self)->Self{*self}}impl Copy for u64{}
impl Clone for u128{fn clone(&self)->Self{*self}}impl Copy for u128{}
impl Clone for usize{fn clone(&self)->Self{*self}}impl Copy for usize{}
impl Clone for i8{fn clone(&self)->Self{*self}}impl Copy for i8{}
impl Clone for i16{fn clone(&self)->Self{*self}}impl Copy for i16{}
impl Clone for i32{fn clone(&self)->Self{*self}}impl Copy for i32{}
impl Clone for i64{fn clone(&self)->Self{*self}}impl Copy for i64{}
impl Clone for i128{fn clone(&self)->Self{*self}}impl Copy for i128{}
impl Clone for isize{fn clone(&self)->Self{*self}}impl Copy for isize{}
#[rustc_intrinsic_const_stable_indirect]
#[rustc_nounwind]
#[rustc_intrinsic]
pub const fn sub_with_overflow<T: Copy>(x: T, y: T) -> (T, bool);

#[rustc_intrinsic_const_stable_indirect]
#[rustc_nounwind]
#[rustc_intrinsic]
pub const unsafe fn unchecked_sub<T: Copy>(x: T, y: T) -> T;

#[rustc_intrinsic]
#[rustc_nounwind]
#[miri::intrinsic_fallback_is_spec]
#[cold]
pub const fn cold_path() {}

const VALUE_u8: u8 = unsafe {unchecked_sub(9u8,4u8)};
const PAIR_u8:(u8,bool) = sub_with_overflow(0u8,1u8);
#[unsafe(no_mangle)]pub extern "C" fn unchecked_u8(a:u8,b:u8)->u8{unsafe{forward_unchecked(a,b)}}
#[unsafe(no_mangle)]pub extern "C" fn wrapping_u8(a:u8,b:u8)->u8{sub_with_overflow(a,b).0}
#[unsafe(no_mangle)]pub extern "C" fn overflow_u8(a:u8,b:u8)->bool{sub_with_overflow(a,b).1}
#[unsafe(no_mangle)]pub extern "C" fn folded_u8()->u8{VALUE_u8}
#[unsafe(no_mangle)]pub extern "C" fn folded_flag_u8()->bool{PAIR_u8.1}

const VALUE_u16: u16 = unsafe {unchecked_sub(9u16,4u16)};
const PAIR_u16:(u16,bool) = sub_with_overflow(0u16,1u16);
#[unsafe(no_mangle)]pub extern "C" fn unchecked_u16(a:u16,b:u16)->u16{unsafe{forward_unchecked(a,b)}}
#[unsafe(no_mangle)]pub extern "C" fn wrapping_u16(a:u16,b:u16)->u16{sub_with_overflow(a,b).0}
#[unsafe(no_mangle)]pub extern "C" fn overflow_u16(a:u16,b:u16)->bool{sub_with_overflow(a,b).1}
#[unsafe(no_mangle)]pub extern "C" fn folded_u16()->u16{VALUE_u16}
#[unsafe(no_mangle)]pub extern "C" fn folded_flag_u16()->bool{PAIR_u16.1}

const VALUE_u32: u32 = unsafe {unchecked_sub(9u32,4u32)};
const PAIR_u32:(u32,bool) = sub_with_overflow(0u32,1u32);
#[unsafe(no_mangle)]pub extern "C" fn unchecked_u32(a:u32,b:u32)->u32{unsafe{forward_unchecked(a,b)}}
#[unsafe(no_mangle)]pub extern "C" fn wrapping_u32(a:u32,b:u32)->u32{sub_with_overflow(a,b).0}
#[unsafe(no_mangle)]pub extern "C" fn overflow_u32(a:u32,b:u32)->bool{sub_with_overflow(a,b).1}
#[unsafe(no_mangle)]pub extern "C" fn folded_u32()->u32{VALUE_u32}
#[unsafe(no_mangle)]pub extern "C" fn folded_flag_u32()->bool{PAIR_u32.1}

const VALUE_u64: u64 = unsafe {unchecked_sub(9u64,4u64)};
const PAIR_u64:(u64,bool) = sub_with_overflow(0u64,1u64);
#[unsafe(no_mangle)]pub extern "C" fn unchecked_u64(a:u64,b:u64)->u64{unsafe{forward_unchecked(a,b)}}
#[unsafe(no_mangle)]pub extern "C" fn wrapping_u64(a:u64,b:u64)->u64{sub_with_overflow(a,b).0}
#[unsafe(no_mangle)]pub extern "C" fn overflow_u64(a:u64,b:u64)->bool{sub_with_overflow(a,b).1}
#[unsafe(no_mangle)]pub extern "C" fn folded_u64()->u64{VALUE_u64}
#[unsafe(no_mangle)]pub extern "C" fn folded_flag_u64()->bool{PAIR_u64.1}

const VALUE_u128: u128 = unsafe {unchecked_sub(9u128,4u128)};
const PAIR_u128:(u128,bool) = sub_with_overflow(0u128,1u128);
#[unsafe(no_mangle)]pub extern "C" fn unchecked_u128(a:u128,b:u128)->u128{unsafe{forward_unchecked(a,b)}}
#[unsafe(no_mangle)]pub extern "C" fn wrapping_u128(a:u128,b:u128)->u128{sub_with_overflow(a,b).0}
#[unsafe(no_mangle)]pub extern "C" fn overflow_u128(a:u128,b:u128)->bool{sub_with_overflow(a,b).1}
#[unsafe(no_mangle)]pub extern "C" fn folded_u128()->u128{VALUE_u128}
#[unsafe(no_mangle)]pub extern "C" fn folded_flag_u128()->bool{PAIR_u128.1}

const VALUE_usize: usize = unsafe {unchecked_sub(9usize,4usize)};
const PAIR_usize:(usize,bool) = sub_with_overflow(0usize,1usize);
#[unsafe(no_mangle)]pub extern "C" fn unchecked_usize(a:usize,b:usize)->usize{unsafe{forward_unchecked(a,b)}}
#[unsafe(no_mangle)]pub extern "C" fn wrapping_usize(a:usize,b:usize)->usize{sub_with_overflow(a,b).0}
#[unsafe(no_mangle)]pub extern "C" fn overflow_usize(a:usize,b:usize)->bool{sub_with_overflow(a,b).1}
#[unsafe(no_mangle)]pub extern "C" fn folded_usize()->usize{VALUE_usize}
#[unsafe(no_mangle)]pub extern "C" fn folded_flag_usize()->bool{PAIR_usize.1}

const VALUE_i8: i8 = unsafe {unchecked_sub(9i8,4i8)};
const PAIR_i8:(i8,bool) = sub_with_overflow((128u8 as i8),1i8);
#[unsafe(no_mangle)]pub extern "C" fn unchecked_i8(a:i8,b:i8)->i8{unsafe{forward_unchecked(a,b)}}
#[unsafe(no_mangle)]pub extern "C" fn wrapping_i8(a:i8,b:i8)->i8{sub_with_overflow(a,b).0}
#[unsafe(no_mangle)]pub extern "C" fn overflow_i8(a:i8,b:i8)->bool{sub_with_overflow(a,b).1}
#[unsafe(no_mangle)]pub extern "C" fn folded_i8()->i8{VALUE_i8}
#[unsafe(no_mangle)]pub extern "C" fn folded_flag_i8()->bool{PAIR_i8.1}

const VALUE_i16: i16 = unsafe {unchecked_sub(9i16,4i16)};
const PAIR_i16:(i16,bool) = sub_with_overflow((32768u16 as i16),1i16);
#[unsafe(no_mangle)]pub extern "C" fn unchecked_i16(a:i16,b:i16)->i16{unsafe{forward_unchecked(a,b)}}
#[unsafe(no_mangle)]pub extern "C" fn wrapping_i16(a:i16,b:i16)->i16{sub_with_overflow(a,b).0}
#[unsafe(no_mangle)]pub extern "C" fn overflow_i16(a:i16,b:i16)->bool{sub_with_overflow(a,b).1}
#[unsafe(no_mangle)]pub extern "C" fn folded_i16()->i16{VALUE_i16}
#[unsafe(no_mangle)]pub extern "C" fn folded_flag_i16()->bool{PAIR_i16.1}

const VALUE_i32: i32 = unsafe {unchecked_sub(9i32,4i32)};
const PAIR_i32:(i32,bool) = sub_with_overflow((2147483648u32 as i32),1i32);
#[unsafe(no_mangle)]pub extern "C" fn unchecked_i32(a:i32,b:i32)->i32{unsafe{forward_unchecked(a,b)}}
#[unsafe(no_mangle)]pub extern "C" fn wrapping_i32(a:i32,b:i32)->i32{sub_with_overflow(a,b).0}
#[unsafe(no_mangle)]pub extern "C" fn overflow_i32(a:i32,b:i32)->bool{sub_with_overflow(a,b).1}
#[unsafe(no_mangle)]pub extern "C" fn folded_i32()->i32{VALUE_i32}
#[unsafe(no_mangle)]pub extern "C" fn folded_flag_i32()->bool{PAIR_i32.1}

const VALUE_i64: i64 = unsafe {unchecked_sub(9i64,4i64)};
const PAIR_i64:(i64,bool) = sub_with_overflow((9223372036854775808u64 as i64),1i64);
#[unsafe(no_mangle)]pub extern "C" fn unchecked_i64(a:i64,b:i64)->i64{unsafe{forward_unchecked(a,b)}}
#[unsafe(no_mangle)]pub extern "C" fn wrapping_i64(a:i64,b:i64)->i64{sub_with_overflow(a,b).0}
#[unsafe(no_mangle)]pub extern "C" fn overflow_i64(a:i64,b:i64)->bool{sub_with_overflow(a,b).1}
#[unsafe(no_mangle)]pub extern "C" fn folded_i64()->i64{VALUE_i64}
#[unsafe(no_mangle)]pub extern "C" fn folded_flag_i64()->bool{PAIR_i64.1}

const VALUE_i128: i128 = unsafe {unchecked_sub(9i128,4i128)};
const PAIR_i128:(i128,bool) = sub_with_overflow((170141183460469231731687303715884105728u128 as i128),1i128);
#[unsafe(no_mangle)]pub extern "C" fn unchecked_i128(a:i128,b:i128)->i128{unsafe{forward_unchecked(a,b)}}
#[unsafe(no_mangle)]pub extern "C" fn wrapping_i128(a:i128,b:i128)->i128{sub_with_overflow(a,b).0}
#[unsafe(no_mangle)]pub extern "C" fn overflow_i128(a:i128,b:i128)->bool{sub_with_overflow(a,b).1}
#[unsafe(no_mangle)]pub extern "C" fn folded_i128()->i128{VALUE_i128}
#[unsafe(no_mangle)]pub extern "C" fn folded_flag_i128()->bool{PAIR_i128.1}

const VALUE_isize: isize = unsafe {unchecked_sub(9isize,4isize)};
const PAIR_isize:(isize,bool) = sub_with_overflow((9223372036854775808usize as isize),1isize);
#[unsafe(no_mangle)]pub extern "C" fn unchecked_isize(a:isize,b:isize)->isize{unsafe{forward_unchecked(a,b)}}
#[unsafe(no_mangle)]pub extern "C" fn wrapping_isize(a:isize,b:isize)->isize{sub_with_overflow(a,b).0}
#[unsafe(no_mangle)]pub extern "C" fn overflow_isize(a:isize,b:isize)->bool{sub_with_overflow(a,b).1}
#[unsafe(no_mangle)]pub extern "C" fn folded_isize()->isize{VALUE_isize}
#[unsafe(no_mangle)]pub extern "C" fn folded_flag_isize()->bool{PAIR_isize.1}

#[unsafe(no_mangle)]pub extern "C" fn cold_observed()->u32{cold_path();7}
const COLD:()=cold_path();

pub const unsafe fn forward_unchecked<T:Copy>(a:T,b:T)->T{unsafe{unchecked_sub(a,b)}}
