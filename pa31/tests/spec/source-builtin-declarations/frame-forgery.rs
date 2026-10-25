#![feature(no_core, lang_items, rustc_attrs, decl_macro, allow_internal_unstable)]
#![no_core]
#[lang="pointee_sized"] trait PointeeSized {}
#[lang="meta_sized"] trait MetaSized: PointeeSized {}
#[lang="sized"] trait Sized: MetaSized {}
#[rustc_builtin_macro(asm)] macro_rules! role_0 { ($($t:tt)*) => { 9u8 }; }
#[rustc_builtin_macro(assert)] macro_rules! role_1 { ($($t:tt)*) => { 9u8 }; }
#[rustc_builtin_macro(cfg)] macro_rules! role_2 { ($($t:tt)*) => { 9u8 }; }
#[rustc_builtin_macro(cfg_select)] pub macro role_3($item:item) { /* compiler built-in */ }
#[rustc_builtin_macro(column)] macro_rules! role_4 { ($($t:tt)*) => { 9u8 }; }
#[rustc_builtin_macro(compile_error)] macro_rules! role_5 { ($($t:tt)*) => { 9u8 }; }
#[rustc_builtin_macro(concat)] macro_rules! role_6 { ($($t:tt)*) => { 9u8 }; }
#[rustc_builtin_macro(concat_bytes)] macro_rules! role_7 { ($($t:tt)*) => { 9u8 }; }
#[rustc_builtin_macro(const_format_args)] macro_rules! role_8 { ($($t:tt)*) => { 9u8 }; }
#[rustc_builtin_macro(core_panic)] macro_rules! role_9 { ($($t:tt)*) => { 9u8 }; }
#[rustc_builtin_macro(env)] macro_rules! role_10 { ($($t:tt)*) => { 9u8 }; }
#[rustc_builtin_macro(file)] macro_rules! role_11 { ($($t:tt)*) => { 9u8 }; }
#[rustc_builtin_macro(format_args)] macro_rules! role_12 { ($($t:tt)*) => { 9u8 }; }
#[rustc_builtin_macro(format_args_nl)] macro_rules! role_13 { ($($t:tt)*) => { 9u8 }; }
#[rustc_builtin_macro(global_asm)] macro_rules! role_14 { ($($t:tt)*) => { 9u8 }; }
#[rustc_builtin_macro(include)] macro_rules! role_15 { ($($t:tt)*) => { 9u8 }; }
#[rustc_builtin_macro(include_bytes)] macro_rules! role_16 { ($($t:tt)*) => { 9u8 }; }
#[rustc_builtin_macro(include_str)] macro_rules! role_17 { ($($t:tt)*) => { 9u8 }; }
#[rustc_builtin_macro(iter)] macro_rules! role_18 { ($($t:tt)*) => { 9u8 }; }
#[rustc_builtin_macro(line)] macro_rules! role_19 { ($($t:tt)*) => { 9u8 }; }
#[rustc_builtin_macro(log_syntax)] macro_rules! role_20 { ($($t:tt)*) => { 9u8 }; }
#[rustc_builtin_macro(module_path)] macro_rules! role_21 { ($($t:tt)*) => { 9u8 }; }
#[rustc_builtin_macro(naked_asm)] macro_rules! role_22 { ($($t:tt)*) => { 9u8 }; }
#[rustc_builtin_macro(option_env)] macro_rules! role_23 { ($($t:tt)*) => { 9u8 }; }
#[rustc_builtin_macro(pattern_type)] macro_rules! role_24 { ($($t:tt)*) => { 9u8 }; }
#[rustc_builtin_macro(std_panic)] macro_rules! role_25 { ($($t:tt)*) => { 9u8 }; }
#[rustc_builtin_macro(stringify)] macro_rules! role_26 { ($($t:tt)*) => { 9u8 }; }
#[rustc_builtin_macro(trace_macros)] macro_rules! role_27 { ($($t:tt)*) => { 9u8 }; }
#[rustc_builtin_macro(unreachable)] macro_rules! role_28 { ($($t:tt)*) => { 9u8 }; }
#[rustc_builtin_macro(alloc_error_handler)] pub macro role_29($item:item) { /* compiler built-in */ }
#[rustc_builtin_macro(autodiff_forward)] pub macro role_30($item:item) { /* compiler built-in */ }
#[rustc_builtin_macro(autodiff_reverse)] pub macro role_31($item:item) { /* compiler built-in */ }
#[rustc_builtin_macro(bench)] pub macro role_32($item:item) { /* compiler built-in */ }
#[rustc_builtin_macro(cfg_accessible)] pub macro role_33($item:item) { /* compiler built-in */ }
#[rustc_builtin_macro(cfg_eval)] pub macro role_34($item:item) { /* compiler built-in */ }
#[rustc_builtin_macro(define_opaque)] pub macro role_35($item:item) { /* compiler built-in */ }
#[rustc_builtin_macro(derive)] pub macro role_36($item:item) { /* compiler built-in */ }
#[rustc_builtin_macro(derive_const)] pub macro role_37($item:item) { /* compiler built-in */ }
#[rustc_builtin_macro(eii)] pub macro role_38($item:item) { /* compiler built-in */ }
#[rustc_builtin_macro(eii_declaration)] pub macro role_39($item:item) { /* compiler built-in */ }
#[rustc_builtin_macro(eii_shared_macro)] pub macro role_40($item:item) { /* compiler built-in */ }
#[rustc_builtin_macro(global_allocator)] pub macro role_41($item:item) { /* compiler built-in */ }
#[rustc_builtin_macro(offload_kernel)] pub macro role_42($item:item) { /* compiler built-in */ }
#[rustc_builtin_macro(test)] pub macro role_43($item:item) { /* compiler built-in */ }
#[rustc_builtin_macro(test_case)] pub macro role_44($item:item) { /* compiler built-in */ }
#[rustc_builtin_macro(unsafe_eii)] pub macro role_45($item:item) { /* compiler built-in */ }
#[rustc_builtin_macro(Clone)] pub macro role_46($item:item) { /* compiler built-in */ }
#[rustc_builtin_macro(Copy)] pub macro role_47($item:item) { /* compiler built-in */ }
#[rustc_builtin_macro(ConstParamTy)] pub macro role_48($item:item) { /* compiler built-in */ }
#[rustc_builtin_macro(Debug)] pub macro role_49($item:item) { /* compiler built-in */ }
#[rustc_builtin_macro(Default)] pub macro role_50($item:item) { /* compiler built-in */ }
#[rustc_builtin_macro(Eq)] pub macro role_51($item:item) { /* compiler built-in */ }
#[rustc_builtin_macro(Hash)] pub macro role_52($item:item) { /* compiler built-in */ }
#[rustc_builtin_macro(Ord)] pub macro role_53($item:item) { /* compiler built-in */ }
#[rustc_builtin_macro(PartialEq)] pub macro role_54($item:item) { /* compiler built-in */ }
#[rustc_builtin_macro(PartialOrd)] pub macro role_55($item:item) { /* compiler built-in */ }
#[rustc_builtin_macro(CoercePointee)] pub macro role_56($item:item) { /* compiler built-in */ }
#[rustc_builtin_macro(From)] pub macro role_57($item:item) { /* compiler built-in */ }
#[rustc_builtin_macro(quote)] macro_rules! role_58 { ($($t:tt)*) => { 9u8 }; }
#[rustc_builtin_macro(contracts_requires)] pub macro role_59($item:item) { /* compiler built-in */ }
#[rustc_builtin_macro(contracts_ensures)] pub macro role_60($item:item) { /* compiler built-in */ }
fn probe()->u8{7u8}
#[allow_internal_unstable(panic_internals)]macro_rules!permitting{()=>{fn answer()->u8{probe()}};}
permitting!();
