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
#[rustc_intrinsic]pub const fn wrapping_add<T:Copy>(a:T,b:T)->T;
#[rustc_intrinsic]pub const fn wrapping_sub<T:Copy>(a:T,b:T)->T;
#[rustc_intrinsic]pub const fn wrapping_mul<T:Copy>(a:T,b:T)->T;
const VALUE_add_u8:u8=wrapping_add((255u8 as u8),(1u8 as u8));
#[unsafe(no_mangle)]pub extern "C" fn wrap_add_u8(a:u8,b:u8)->u8{wrapping_add(a,b)}
#[unsafe(no_mangle)]pub extern "C" fn fold_add_u8()->u8{VALUE_add_u8}
const VALUE_sub_u8:u8=wrapping_sub((0u8 as u8),(1u8 as u8));
#[unsafe(no_mangle)]pub extern "C" fn wrap_sub_u8(a:u8,b:u8)->u8{wrapping_sub(a,b)}
#[unsafe(no_mangle)]pub extern "C" fn fold_sub_u8()->u8{VALUE_sub_u8}
const VALUE_mul_u8:u8=wrapping_mul((128u8 as u8),(2u8 as u8));
#[unsafe(no_mangle)]pub extern "C" fn wrap_mul_u8(a:u8,b:u8)->u8{wrapping_mul(a,b)}
#[unsafe(no_mangle)]pub extern "C" fn fold_mul_u8()->u8{VALUE_mul_u8}
const VALUE_add_u16:u16=wrapping_add((65535u16 as u16),(1u16 as u16));
#[unsafe(no_mangle)]pub extern "C" fn wrap_add_u16(a:u16,b:u16)->u16{wrapping_add(a,b)}
#[unsafe(no_mangle)]pub extern "C" fn fold_add_u16()->u16{VALUE_add_u16}
const VALUE_sub_u16:u16=wrapping_sub((0u16 as u16),(1u16 as u16));
#[unsafe(no_mangle)]pub extern "C" fn wrap_sub_u16(a:u16,b:u16)->u16{wrapping_sub(a,b)}
#[unsafe(no_mangle)]pub extern "C" fn fold_sub_u16()->u16{VALUE_sub_u16}
const VALUE_mul_u16:u16=wrapping_mul((32768u16 as u16),(2u16 as u16));
#[unsafe(no_mangle)]pub extern "C" fn wrap_mul_u16(a:u16,b:u16)->u16{wrapping_mul(a,b)}
#[unsafe(no_mangle)]pub extern "C" fn fold_mul_u16()->u16{VALUE_mul_u16}
const VALUE_add_u32:u32=wrapping_add((4294967295u32 as u32),(1u32 as u32));
#[unsafe(no_mangle)]pub extern "C" fn wrap_add_u32(a:u32,b:u32)->u32{wrapping_add(a,b)}
#[unsafe(no_mangle)]pub extern "C" fn fold_add_u32()->u32{VALUE_add_u32}
const VALUE_sub_u32:u32=wrapping_sub((0u32 as u32),(1u32 as u32));
#[unsafe(no_mangle)]pub extern "C" fn wrap_sub_u32(a:u32,b:u32)->u32{wrapping_sub(a,b)}
#[unsafe(no_mangle)]pub extern "C" fn fold_sub_u32()->u32{VALUE_sub_u32}
const VALUE_mul_u32:u32=wrapping_mul((2147483648u32 as u32),(2u32 as u32));
#[unsafe(no_mangle)]pub extern "C" fn wrap_mul_u32(a:u32,b:u32)->u32{wrapping_mul(a,b)}
#[unsafe(no_mangle)]pub extern "C" fn fold_mul_u32()->u32{VALUE_mul_u32}
const VALUE_add_u64:u64=wrapping_add((18446744073709551615u64 as u64),(1u64 as u64));
#[unsafe(no_mangle)]pub extern "C" fn wrap_add_u64(a:u64,b:u64)->u64{wrapping_add(a,b)}
#[unsafe(no_mangle)]pub extern "C" fn fold_add_u64()->u64{VALUE_add_u64}
const VALUE_sub_u64:u64=wrapping_sub((0u64 as u64),(1u64 as u64));
#[unsafe(no_mangle)]pub extern "C" fn wrap_sub_u64(a:u64,b:u64)->u64{wrapping_sub(a,b)}
#[unsafe(no_mangle)]pub extern "C" fn fold_sub_u64()->u64{VALUE_sub_u64}
const VALUE_mul_u64:u64=wrapping_mul((9223372036854775808u64 as u64),(2u64 as u64));
#[unsafe(no_mangle)]pub extern "C" fn wrap_mul_u64(a:u64,b:u64)->u64{wrapping_mul(a,b)}
#[unsafe(no_mangle)]pub extern "C" fn fold_mul_u64()->u64{VALUE_mul_u64}
const VALUE_add_u128:u128=wrapping_add((340282366920938463463374607431768211455u128 as u128),(1u128 as u128));
#[unsafe(no_mangle)]pub extern "C" fn wrap_add_u128(a:u128,b:u128)->u128{wrapping_add(a,b)}
#[unsafe(no_mangle)]pub extern "C" fn fold_add_u128()->u128{VALUE_add_u128}
const VALUE_sub_u128:u128=wrapping_sub((0u128 as u128),(1u128 as u128));
#[unsafe(no_mangle)]pub extern "C" fn wrap_sub_u128(a:u128,b:u128)->u128{wrapping_sub(a,b)}
#[unsafe(no_mangle)]pub extern "C" fn fold_sub_u128()->u128{VALUE_sub_u128}
const VALUE_mul_u128:u128=wrapping_mul((170141183460469231731687303715884105728u128 as u128),(2u128 as u128));
#[unsafe(no_mangle)]pub extern "C" fn wrap_mul_u128(a:u128,b:u128)->u128{wrapping_mul(a,b)}
#[unsafe(no_mangle)]pub extern "C" fn fold_mul_u128()->u128{VALUE_mul_u128}
const VALUE_add_usize:usize=wrapping_add((18446744073709551615u64 as usize),(1u64 as usize));
#[unsafe(no_mangle)]pub extern "C" fn wrap_add_usize(a:usize,b:usize)->usize{wrapping_add(a,b)}
#[unsafe(no_mangle)]pub extern "C" fn fold_add_usize()->usize{VALUE_add_usize}
const VALUE_sub_usize:usize=wrapping_sub((0u64 as usize),(1u64 as usize));
#[unsafe(no_mangle)]pub extern "C" fn wrap_sub_usize(a:usize,b:usize)->usize{wrapping_sub(a,b)}
#[unsafe(no_mangle)]pub extern "C" fn fold_sub_usize()->usize{VALUE_sub_usize}
const VALUE_mul_usize:usize=wrapping_mul((9223372036854775808u64 as usize),(2u64 as usize));
#[unsafe(no_mangle)]pub extern "C" fn wrap_mul_usize(a:usize,b:usize)->usize{wrapping_mul(a,b)}
#[unsafe(no_mangle)]pub extern "C" fn fold_mul_usize()->usize{VALUE_mul_usize}
const VALUE_add_i8:i8=wrapping_add((127u8 as i8),(1u8 as i8));
#[unsafe(no_mangle)]pub extern "C" fn wrap_add_i8(a:i8,b:i8)->i8{wrapping_add(a,b)}
#[unsafe(no_mangle)]pub extern "C" fn fold_add_i8()->i8{VALUE_add_i8}
const VALUE_sub_i8:i8=wrapping_sub((128u8 as i8),(1u8 as i8));
#[unsafe(no_mangle)]pub extern "C" fn wrap_sub_i8(a:i8,b:i8)->i8{wrapping_sub(a,b)}
#[unsafe(no_mangle)]pub extern "C" fn fold_sub_i8()->i8{VALUE_sub_i8}
const VALUE_mul_i8:i8=wrapping_mul((128u8 as i8),(2u8 as i8));
#[unsafe(no_mangle)]pub extern "C" fn wrap_mul_i8(a:i8,b:i8)->i8{wrapping_mul(a,b)}
#[unsafe(no_mangle)]pub extern "C" fn fold_mul_i8()->i8{VALUE_mul_i8}
const VALUE_add_i16:i16=wrapping_add((32767u16 as i16),(1u16 as i16));
#[unsafe(no_mangle)]pub extern "C" fn wrap_add_i16(a:i16,b:i16)->i16{wrapping_add(a,b)}
#[unsafe(no_mangle)]pub extern "C" fn fold_add_i16()->i16{VALUE_add_i16}
const VALUE_sub_i16:i16=wrapping_sub((32768u16 as i16),(1u16 as i16));
#[unsafe(no_mangle)]pub extern "C" fn wrap_sub_i16(a:i16,b:i16)->i16{wrapping_sub(a,b)}
#[unsafe(no_mangle)]pub extern "C" fn fold_sub_i16()->i16{VALUE_sub_i16}
const VALUE_mul_i16:i16=wrapping_mul((32768u16 as i16),(2u16 as i16));
#[unsafe(no_mangle)]pub extern "C" fn wrap_mul_i16(a:i16,b:i16)->i16{wrapping_mul(a,b)}
#[unsafe(no_mangle)]pub extern "C" fn fold_mul_i16()->i16{VALUE_mul_i16}
const VALUE_add_i32:i32=wrapping_add((2147483647u32 as i32),(1u32 as i32));
#[unsafe(no_mangle)]pub extern "C" fn wrap_add_i32(a:i32,b:i32)->i32{wrapping_add(a,b)}
#[unsafe(no_mangle)]pub extern "C" fn fold_add_i32()->i32{VALUE_add_i32}
const VALUE_sub_i32:i32=wrapping_sub((2147483648u32 as i32),(1u32 as i32));
#[unsafe(no_mangle)]pub extern "C" fn wrap_sub_i32(a:i32,b:i32)->i32{wrapping_sub(a,b)}
#[unsafe(no_mangle)]pub extern "C" fn fold_sub_i32()->i32{VALUE_sub_i32}
const VALUE_mul_i32:i32=wrapping_mul((2147483648u32 as i32),(2u32 as i32));
#[unsafe(no_mangle)]pub extern "C" fn wrap_mul_i32(a:i32,b:i32)->i32{wrapping_mul(a,b)}
#[unsafe(no_mangle)]pub extern "C" fn fold_mul_i32()->i32{VALUE_mul_i32}
const VALUE_add_i64:i64=wrapping_add((9223372036854775807u64 as i64),(1u64 as i64));
#[unsafe(no_mangle)]pub extern "C" fn wrap_add_i64(a:i64,b:i64)->i64{wrapping_add(a,b)}
#[unsafe(no_mangle)]pub extern "C" fn fold_add_i64()->i64{VALUE_add_i64}
const VALUE_sub_i64:i64=wrapping_sub((9223372036854775808u64 as i64),(1u64 as i64));
#[unsafe(no_mangle)]pub extern "C" fn wrap_sub_i64(a:i64,b:i64)->i64{wrapping_sub(a,b)}
#[unsafe(no_mangle)]pub extern "C" fn fold_sub_i64()->i64{VALUE_sub_i64}
const VALUE_mul_i64:i64=wrapping_mul((9223372036854775808u64 as i64),(2u64 as i64));
#[unsafe(no_mangle)]pub extern "C" fn wrap_mul_i64(a:i64,b:i64)->i64{wrapping_mul(a,b)}
#[unsafe(no_mangle)]pub extern "C" fn fold_mul_i64()->i64{VALUE_mul_i64}
const VALUE_add_i128:i128=wrapping_add((170141183460469231731687303715884105727u128 as i128),(1u128 as i128));
#[unsafe(no_mangle)]pub extern "C" fn wrap_add_i128(a:i128,b:i128)->i128{wrapping_add(a,b)}
#[unsafe(no_mangle)]pub extern "C" fn fold_add_i128()->i128{VALUE_add_i128}
const VALUE_sub_i128:i128=wrapping_sub((170141183460469231731687303715884105728u128 as i128),(1u128 as i128));
#[unsafe(no_mangle)]pub extern "C" fn wrap_sub_i128(a:i128,b:i128)->i128{wrapping_sub(a,b)}
#[unsafe(no_mangle)]pub extern "C" fn fold_sub_i128()->i128{VALUE_sub_i128}
const VALUE_mul_i128:i128=wrapping_mul((170141183460469231731687303715884105728u128 as i128),(2u128 as i128));
#[unsafe(no_mangle)]pub extern "C" fn wrap_mul_i128(a:i128,b:i128)->i128{wrapping_mul(a,b)}
#[unsafe(no_mangle)]pub extern "C" fn fold_mul_i128()->i128{VALUE_mul_i128}
const VALUE_add_isize:isize=wrapping_add((9223372036854775807u64 as isize),(1u64 as isize));
#[unsafe(no_mangle)]pub extern "C" fn wrap_add_isize(a:isize,b:isize)->isize{wrapping_add(a,b)}
#[unsafe(no_mangle)]pub extern "C" fn fold_add_isize()->isize{VALUE_add_isize}
const VALUE_sub_isize:isize=wrapping_sub((9223372036854775808u64 as isize),(1u64 as isize));
#[unsafe(no_mangle)]pub extern "C" fn wrap_sub_isize(a:isize,b:isize)->isize{wrapping_sub(a,b)}
#[unsafe(no_mangle)]pub extern "C" fn fold_sub_isize()->isize{VALUE_sub_isize}
const VALUE_mul_isize:isize=wrapping_mul((9223372036854775808u64 as isize),(2u64 as isize));
#[unsafe(no_mangle)]pub extern "C" fn wrap_mul_isize(a:isize,b:isize)->isize{wrapping_mul(a,b)}
#[unsafe(no_mangle)]pub extern "C" fn fold_mul_isize()->isize{VALUE_mul_isize}
fn forward<T:Copy>(a:T,b:T)->T{wrapping_add(a,b)}
#[unsafe(no_mangle)]pub extern "C" fn forward_add(a:u32,b:u32)->u32{forward(a,b)}
