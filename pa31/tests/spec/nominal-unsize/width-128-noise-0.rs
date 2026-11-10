// Source-role foundation adapted from the registered PA31 unsizing protocol.
#![feature(no_core,lang_items,intrinsics,rustc_attrs,const_trait_impl,auto_traits)]
#![no_core]
#[lang="unsize"]#[rustc_deny_explicit_impl]#[rustc_dyn_incompatible_trait]
pub trait Unsize<T:PointeeSized>:PointeeSized{}
#[lang="coerce_unsized"]pub trait CoerceUnsized<T:PointeeSized>:Sized{}
impl<'a,'b:'a,T:PointeeSized+Unsize<U>,U:PointeeSized> CoerceUnsized<&'a mut U> for &'b mut T{}
impl<T:PointeeSized+Unsize<U>,U:PointeeSized> CoerceUnsized<*mut U> for *mut T{}
#[lang = "freeze"]
pub unsafe auto trait Freeze {}
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="clone"]pub const trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized>LegacyReceiver for &T{}
impl<T:PointeeSized>LegacyReceiver for &mut T{}
#[lang="drop"]pub const trait Drop{fn drop(&mut self);}
pub mod intrinsics{use crate::Sized;#[rustc_intrinsic]pub const fn needs_drop<T:?Sized>()->bool;}
pub const fn needs_drop<T: ?Sized>() -> bool {
    const { intrinsics::needs_drop::<T>() }
}
#[lang="destruct"]#[rustc_deny_explicit_impl]pub const trait Destruct:PointeeSized{}
#[lang = "drop_glue"]
pub(crate) const unsafe fn drop_glue<T: PointeeSized>(_: &mut T)
where
    T: [const] Destruct,
{
    // Code here does not matter - this is replaced by the
    // real drop glue by the compiler.
}

impl<'a,'b:'a,T:PointeeSized+Unsize<U>,U:PointeeSized> CoerceUnsized<&'a U> for &'b T{}
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
#[lang="dispatch_from_dyn"]pub trait DispatchFromDyn<T>:Sized{}
impl<'a,T:PointeeSized+Unsize<U>,U:PointeeSized> DispatchFromDyn<&'a U> for &'a T{}
impl<'a,T:PointeeSized+Unsize<U>,U:PointeeSized> DispatchFromDyn<&'a mut U> for &'a mut T{}
struct Tail<T:?Sized>{prefix:u8,tail:T}
fn erase(x:&Tail<[u8;2]>)->&Tail<[u8]>{x}
fn site0(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site1(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site2(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site3(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site4(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site5(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site6(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site7(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site8(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site9(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site10(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site11(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site12(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site13(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site14(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site15(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site16(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site17(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site18(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site19(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site20(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site21(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site22(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site23(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site24(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site25(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site26(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site27(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site28(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site29(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site30(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site31(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site32(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site33(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site34(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site35(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site36(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site37(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site38(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site39(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site40(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site41(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site42(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site43(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site44(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site45(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site46(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site47(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site48(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site49(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site50(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site51(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site52(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site53(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site54(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site55(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site56(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site57(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site58(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site59(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site60(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site61(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site62(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site63(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site64(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site65(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site66(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site67(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site68(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site69(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site70(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site71(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site72(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site73(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site74(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site75(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site76(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site77(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site78(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site79(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site80(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site81(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site82(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site83(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site84(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site85(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site86(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site87(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site88(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site89(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site90(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site91(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site92(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site93(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site94(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site95(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site96(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site97(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site98(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site99(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site100(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site101(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site102(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site103(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site104(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site105(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site106(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site107(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site108(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site109(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site110(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site111(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site112(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site113(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site114(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site115(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site116(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site117(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site118(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site119(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site120(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site121(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site122(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site123(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site124(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site125(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site126(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
fn site127(x:&Tail<[u8;2]>)->&Tail<[u8]>{erase(x)}
#[unsafe(no_mangle)]pub extern "C" fn answer(b:bool)->u8{let x=Tail{prefix:9u8,tail:[17u8,23]};let y=site127(&x);if b{match &y.tail{[_,v]=>*v,_=>0}}else{y.prefix}}

impl<T:PointeeSized>Clone for &T{fn clone(&self)->Self{*self}}impl<T:PointeeSized>Copy for &T{}
