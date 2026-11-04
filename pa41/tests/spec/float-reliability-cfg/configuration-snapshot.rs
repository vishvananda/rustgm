#![allow(internal_features,unused_macros,dead_code,unexpected_cfgs)]
#![feature(no_core,lang_items,rustc_attrs,allow_internal_unstable)]
#![no_core]
#[rustc_builtin_macro]macro_rules!cfg{($($t:tt)*)=>{}}
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[allow_internal_unstable(cfg_target_has_reliable_f16_f128,cfg_target_has_atomic,cfg_ub_checks,cfg_sanitize)]
macro_rules! snapshot {()=>{pub const RELIABLE_F16:bool=cfg!(target_has_reliable_f16);pub const RELIABLE_F16_MATH:bool=cfg!(target_has_reliable_f16_math);pub const RELIABLE_F128:bool=cfg!(target_has_reliable_f128);pub const RELIABLE_F128_MATH:bool=cfg!(target_has_reliable_f128_math);pub const UNIX:bool=cfg!(unix);pub const WINDOWS:bool=cfg!(windows);pub const SSE2:bool=cfg!(target_feature="sse2");pub const X64:bool=cfg!(target_arch="x86_64");pub const WIDTH64:bool=cfg!(target_pointer_width="64");pub const ABORT:bool=cfg!(panic="abort");pub const ATOMIC64:bool=cfg!(target_has_atomic="64");pub const ATOMIC128:bool=cfg!(target_has_atomic="128");pub const ATOMIC_LOAD:bool=cfg!(target_has_atomic_load_store="64");pub const UB:bool=cfg!(ub_checks);pub const SANITIZER:bool=cfg!(sanitize="cfi");}}
snapshot!();
