#![feature(no_core,lang_items,rustc_attrs,auto_traits,min_adt_const_params)]
#![no_core]
#![allow(internal_features,dead_code)]
#[lang="pointee_sized"] pub trait PointeeSized {}
#[lang="meta_sized"] pub trait MetaSized:PointeeSized {}
#[lang="sized"] pub trait Sized:MetaSized {}
#[lang="legacy_receiver"] pub trait LegacyReceiver:PointeeSized {}
impl<T:PointeeSized> LegacyReceiver for &T {}
impl<T:PointeeSized> LegacyReceiver for &mut T {}
#[lang="freeze"] pub unsafe auto trait Freeze {}
#[lang="clone"] pub trait Clone:Sized { #[lang="clone_fn"] fn clone(&self)->Self; }
#[lang="copy"] pub trait Copy:Clone {}
impl Clone for bool { fn clone(&self)->Self {*self} }
impl Copy for bool {}
impl Clone for u8 { fn clone(&self)->Self {*self} }
impl Copy for u8 {}
#[lang="structural_peq"] pub trait Structural {}
#[rustc_diagnostic_item="Eq"] pub trait Equal {}
#[lang="const_param_ty"] pub trait Key:Structural+Equal {}
impl Structural for bool {}
impl Equal for bool {}
impl Key for bool {}
impl Structural for usize{} impl Equal for usize{} impl Key for usize{}

#[lang="panic_bounds_check"]pub fn marker_138()->u8{9}
#[unsafe(no_mangle)]pub extern "C" fn observed_138()->u8{marker_138()}
#[lang="panic_misaligned_pointer_dereference"]pub fn marker_139()->u8{10}
#[unsafe(no_mangle)]pub extern "C" fn observed_139()->u8{marker_139()}
#[lang="panic_cannot_unwind"]pub fn marker_140()->u8{11}
#[unsafe(no_mangle)]pub extern "C" fn observed_140()->u8{marker_140()}
#[lang="panic_in_cleanup"]pub fn marker_141()->u8{12}
#[unsafe(no_mangle)]pub extern "C" fn observed_141()->u8{marker_141()}
#[lang="panic_null_pointer_dereference"]pub fn marker_142()->u8{13}
#[unsafe(no_mangle)]pub extern "C" fn observed_142()->u8{marker_142()}
#[lang="panic_invalid_enum_construction"]pub fn marker_143()->u8{14}
#[unsafe(no_mangle)]pub extern "C" fn observed_143()->u8{marker_143()}
#[lang="panic_const_add_overflow"]pub fn marker_187()->u8{15}
#[unsafe(no_mangle)]pub extern "C" fn observed_187()->u8{marker_187()}
#[lang="panic_const_sub_overflow"]pub fn marker_188()->u8{16}
#[unsafe(no_mangle)]pub extern "C" fn observed_188()->u8{marker_188()}
#[lang="panic_const_mul_overflow"]pub fn marker_189()->u8{17}
#[unsafe(no_mangle)]pub extern "C" fn observed_189()->u8{marker_189()}
#[lang="panic_const_div_overflow"]pub fn marker_190()->u8{18}
#[unsafe(no_mangle)]pub extern "C" fn observed_190()->u8{marker_190()}
#[lang="panic_const_rem_overflow"]pub fn marker_191()->u8{19}
#[unsafe(no_mangle)]pub extern "C" fn observed_191()->u8{marker_191()}
#[lang="panic_const_neg_overflow"]pub fn marker_192()->u8{20}
#[unsafe(no_mangle)]pub extern "C" fn observed_192()->u8{marker_192()}
#[lang="panic_const_shr_overflow"]pub fn marker_193()->u8{21}
#[unsafe(no_mangle)]pub extern "C" fn observed_193()->u8{marker_193()}
#[lang="panic_const_shl_overflow"]pub fn marker_194()->u8{22}
#[unsafe(no_mangle)]pub extern "C" fn observed_194()->u8{marker_194()}
#[lang="panic_const_div_by_zero"]pub fn marker_195()->u8{23}
#[unsafe(no_mangle)]pub extern "C" fn observed_195()->u8{marker_195()}
#[lang="panic_const_rem_by_zero"]pub fn marker_196()->u8{24}
#[unsafe(no_mangle)]pub extern "C" fn observed_196()->u8{marker_196()}
#[lang="panic_const_coroutine_resumed"]pub fn marker_197()->u8{25}
#[unsafe(no_mangle)]pub extern "C" fn observed_197()->u8{marker_197()}
#[lang="panic_const_async_fn_resumed"]pub fn marker_198()->u8{26}
#[unsafe(no_mangle)]pub extern "C" fn observed_198()->u8{marker_198()}
#[lang="panic_const_async_gen_fn_resumed"]pub fn marker_199()->u8{27}
#[unsafe(no_mangle)]pub extern "C" fn observed_199()->u8{marker_199()}
#[lang="panic_const_gen_fn_none"]pub fn marker_200()->u8{28}
#[unsafe(no_mangle)]pub extern "C" fn observed_200()->u8{marker_200()}
#[lang="panic_const_coroutine_resumed_panic"]pub fn marker_201()->u8{29}
#[unsafe(no_mangle)]pub extern "C" fn observed_201()->u8{marker_201()}
#[lang="panic_const_async_fn_resumed_panic"]pub fn marker_202()->u8{30}
#[unsafe(no_mangle)]pub extern "C" fn observed_202()->u8{marker_202()}
#[lang="panic_const_async_gen_fn_resumed_panic"]pub fn marker_203()->u8{31}
#[unsafe(no_mangle)]pub extern "C" fn observed_203()->u8{marker_203()}
#[lang="panic_const_gen_fn_none_panic"]pub fn marker_204()->u8{32}
#[unsafe(no_mangle)]pub extern "C" fn observed_204()->u8{marker_204()}
#[lang="panic_const_coroutine_resumed_drop"]pub fn marker_205()->u8{33}
#[unsafe(no_mangle)]pub extern "C" fn observed_205()->u8{marker_205()}
#[lang="panic_const_async_fn_resumed_drop"]pub fn marker_206()->u8{34}
#[unsafe(no_mangle)]pub extern "C" fn observed_206()->u8{marker_206()}
#[lang="panic_const_async_gen_fn_resumed_drop"]pub fn marker_207()->u8{35}
#[unsafe(no_mangle)]pub extern "C" fn observed_207()->u8{marker_207()}
#[lang="panic_const_gen_fn_none_drop"]pub fn marker_208()->u8{36}
#[unsafe(no_mangle)]pub extern "C" fn observed_208()->u8{marker_208()}
