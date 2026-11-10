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
trait Show{fn show(&self)->u8;}impl Show for u8{fn show(&self)->u8{*self}}impl Show for u16{fn show(&self)->u8{7}}impl Show for u32{fn show(&self)->u8{7}}impl Show for u64{fn show(&self)->u8{7}}impl Show for u128{fn show(&self)->u8{7}}impl Show for usize{fn show(&self)->u8{7}}impl Show for i8{fn show(&self)->u8{7}}impl Show for i16{fn show(&self)->u8{7}}impl Show for i32{fn show(&self)->u8{7}}impl Show for i64{fn show(&self)->u8{7}}impl Show for i128{fn show(&self)->u8{7}}impl Show for isize{fn show(&self)->u8{7}}trait Kind{type Tag:Copy+Show;}struct K;impl Kind for K{type Tag=u8;}fn field(x:&(dyn Show))->u8{x.show()}fn generic<T:Show>(x:&T)->u8{x.show()}struct Wrap<T:Kind>(<T as Kind>::Tag);impl<T:Kind> Show for Wrap<T> {fn show(&self)->u8{field(&self.0)}}fn repeated<T:Kind>(x:&T::Tag){let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);let _=field(x);}struct Noise0;struct Noise1;struct Noise2;struct Noise3;struct Noise4;struct Noise5;struct Noise6;struct Noise7;struct Noise8;struct Noise9;struct Noise10;struct Noise11;struct Noise12;struct Noise13;struct Noise14;struct Noise15;struct Noise16;struct Noise17;struct Noise18;struct Noise19;struct Noise20;struct Noise21;struct Noise22;struct Noise23;struct Noise24;struct Noise25;struct Noise26;struct Noise27;struct Noise28;struct Noise29;struct Noise30;struct Noise31;struct Noise32;struct Noise33;struct Noise34;struct Noise35;struct Noise36;struct Noise37;struct Noise38;struct Noise39;struct Noise40;struct Noise41;struct Noise42;struct Noise43;struct Noise44;struct Noise45;struct Noise46;struct Noise47;struct Noise48;struct Noise49;struct Noise50;struct Noise51;struct Noise52;struct Noise53;struct Noise54;struct Noise55;struct Noise56;struct Noise57;struct Noise58;struct Noise59;struct Noise60;struct Noise61;struct Noise62;struct Noise63;struct Noise64;struct Noise65;struct Noise66;struct Noise67;struct Noise68;struct Noise69;struct Noise70;struct Noise71;struct Noise72;struct Noise73;struct Noise74;struct Noise75;struct Noise76;struct Noise77;struct Noise78;struct Noise79;struct Noise80;struct Noise81;struct Noise82;struct Noise83;struct Noise84;struct Noise85;struct Noise86;struct Noise87;struct Noise88;struct Noise89;struct Noise90;struct Noise91;struct Noise92;struct Noise93;struct Noise94;struct Noise95;struct Noise96;struct Noise97;struct Noise98;struct Noise99;struct Noise100;struct Noise101;struct Noise102;struct Noise103;struct Noise104;struct Noise105;struct Noise106;struct Noise107;struct Noise108;struct Noise109;struct Noise110;struct Noise111;struct Noise112;struct Noise113;struct Noise114;struct Noise115;struct Noise116;struct Noise117;struct Noise118;struct Noise119;struct Noise120;struct Noise121;struct Noise122;struct Noise123;struct Noise124;struct Noise125;struct Noise126;struct Noise127;#[unsafe(no_mangle)]pub extern "C" fn answer(b:bool)->u8{let v=if b{23u8}else{9u8};repeated::<K>(&v);Wrap::<K>(v).show()}

