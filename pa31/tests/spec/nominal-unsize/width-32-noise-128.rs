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
struct Unused0{v:u8}
struct Unused1{v:u8}
struct Unused2{v:u8}
struct Unused3{v:u8}
struct Unused4{v:u8}
struct Unused5{v:u8}
struct Unused6{v:u8}
struct Unused7{v:u8}
struct Unused8{v:u8}
struct Unused9{v:u8}
struct Unused10{v:u8}
struct Unused11{v:u8}
struct Unused12{v:u8}
struct Unused13{v:u8}
struct Unused14{v:u8}
struct Unused15{v:u8}
struct Unused16{v:u8}
struct Unused17{v:u8}
struct Unused18{v:u8}
struct Unused19{v:u8}
struct Unused20{v:u8}
struct Unused21{v:u8}
struct Unused22{v:u8}
struct Unused23{v:u8}
struct Unused24{v:u8}
struct Unused25{v:u8}
struct Unused26{v:u8}
struct Unused27{v:u8}
struct Unused28{v:u8}
struct Unused29{v:u8}
struct Unused30{v:u8}
struct Unused31{v:u8}
struct Unused32{v:u8}
struct Unused33{v:u8}
struct Unused34{v:u8}
struct Unused35{v:u8}
struct Unused36{v:u8}
struct Unused37{v:u8}
struct Unused38{v:u8}
struct Unused39{v:u8}
struct Unused40{v:u8}
struct Unused41{v:u8}
struct Unused42{v:u8}
struct Unused43{v:u8}
struct Unused44{v:u8}
struct Unused45{v:u8}
struct Unused46{v:u8}
struct Unused47{v:u8}
struct Unused48{v:u8}
struct Unused49{v:u8}
struct Unused50{v:u8}
struct Unused51{v:u8}
struct Unused52{v:u8}
struct Unused53{v:u8}
struct Unused54{v:u8}
struct Unused55{v:u8}
struct Unused56{v:u8}
struct Unused57{v:u8}
struct Unused58{v:u8}
struct Unused59{v:u8}
struct Unused60{v:u8}
struct Unused61{v:u8}
struct Unused62{v:u8}
struct Unused63{v:u8}
struct Unused64{v:u8}
struct Unused65{v:u8}
struct Unused66{v:u8}
struct Unused67{v:u8}
struct Unused68{v:u8}
struct Unused69{v:u8}
struct Unused70{v:u8}
struct Unused71{v:u8}
struct Unused72{v:u8}
struct Unused73{v:u8}
struct Unused74{v:u8}
struct Unused75{v:u8}
struct Unused76{v:u8}
struct Unused77{v:u8}
struct Unused78{v:u8}
struct Unused79{v:u8}
struct Unused80{v:u8}
struct Unused81{v:u8}
struct Unused82{v:u8}
struct Unused83{v:u8}
struct Unused84{v:u8}
struct Unused85{v:u8}
struct Unused86{v:u8}
struct Unused87{v:u8}
struct Unused88{v:u8}
struct Unused89{v:u8}
struct Unused90{v:u8}
struct Unused91{v:u8}
struct Unused92{v:u8}
struct Unused93{v:u8}
struct Unused94{v:u8}
struct Unused95{v:u8}
struct Unused96{v:u8}
struct Unused97{v:u8}
struct Unused98{v:u8}
struct Unused99{v:u8}
struct Unused100{v:u8}
struct Unused101{v:u8}
struct Unused102{v:u8}
struct Unused103{v:u8}
struct Unused104{v:u8}
struct Unused105{v:u8}
struct Unused106{v:u8}
struct Unused107{v:u8}
struct Unused108{v:u8}
struct Unused109{v:u8}
struct Unused110{v:u8}
struct Unused111{v:u8}
struct Unused112{v:u8}
struct Unused113{v:u8}
struct Unused114{v:u8}
struct Unused115{v:u8}
struct Unused116{v:u8}
struct Unused117{v:u8}
struct Unused118{v:u8}
struct Unused119{v:u8}
struct Unused120{v:u8}
struct Unused121{v:u8}
struct Unused122{v:u8}
struct Unused123{v:u8}
struct Unused124{v:u8}
struct Unused125{v:u8}
struct Unused126{v:u8}
struct Unused127{v:u8}
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
#[unsafe(no_mangle)]pub extern "C" fn answer(b:bool)->u8{let x=Tail{prefix:9u8,tail:[17u8,23]};let y=site31(&x);if b{match &y.tail{[_,v]=>*v,_=>0}}else{y.prefix}}

impl<T:PointeeSized>Clone for &T{fn clone(&self)->Self{*self}}impl<T:PointeeSized>Copy for &T{}
