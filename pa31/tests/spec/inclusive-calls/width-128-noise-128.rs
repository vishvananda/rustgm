#![feature(const_destruct)]
#![feature(staged_api,const_trait_impl,auto_traits,negative_impls)]
#![unstable(feature="marker_fixture",issue="none")]
#![feature(no_core,lang_items,rustc_attrs,fundamental)]
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
pub trait LegacyReceiver: PointeeSized {}
impl<T: PointeeSized> LegacyReceiver for &T {}
impl<T: PointeeSized> LegacyReceiver for &mut T {}
#[lang = "clone"]
pub trait Clone: Sized { #[lang = "clone_fn"] fn clone(&self) -> Self; }
#[lang = "copy"]
pub trait Copy: Clone {}

#[lang="freeze"]pub unsafe auto trait Freeze{}
impl Clone for u8{fn clone(&self)->Self{*self}}impl Copy for u8{}
#[unstable(feature = "const_destruct", issue = "133214")]
#[rustc_const_unstable(feature = "const_destruct", issue = "133214")]
#[lang = "destruct"]
#[diagnostic::on_unimplemented(message = "can't drop `{Self}`")]
#[rustc_deny_explicit_impl]
#[rustc_dyn_incompatible_trait]
pub const trait Destruct: PointeeSized {}
#[lang="RangeInclusive"]pub struct Bounds<T>{pub(crate) start:T,pub(crate) end:T,pub(crate) exhausted:bool}
impl<T> Bounds<T>{#[lang="range_inclusive_new"]pub const fn create(start:T,end:T)->Self{Self{start,end,exhausted:false}}}
pub fn f0(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f1(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f2(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f3(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f4(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f5(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f6(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f7(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f8(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f9(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f10(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f11(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f12(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f13(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f14(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f15(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f16(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f17(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f18(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f19(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f20(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f21(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f22(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f23(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f24(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f25(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f26(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f27(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f28(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f29(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f30(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f31(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f32(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f33(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f34(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f35(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f36(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f37(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f38(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f39(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f40(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f41(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f42(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f43(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f44(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f45(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f46(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f47(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f48(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f49(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f50(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f51(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f52(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f53(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f54(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f55(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f56(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f57(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f58(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f59(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f60(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f61(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f62(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f63(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f64(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f65(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f66(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f67(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f68(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f69(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f70(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f71(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f72(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f73(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f74(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f75(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f76(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f77(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f78(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f79(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f80(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f81(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f82(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f83(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f84(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f85(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f86(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f87(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f88(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f89(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f90(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f91(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f92(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f93(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f94(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f95(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f96(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f97(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f98(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f99(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f100(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f101(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f102(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f103(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f104(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f105(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f106(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f107(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f108(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f109(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f110(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f111(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f112(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f113(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f114(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f115(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f116(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f117(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f118(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f119(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f120(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f121(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f122(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f123(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f124(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f125(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f126(x:u8,y:u8)->Bounds<u8>{x..=y}
pub fn f127(x:u8,y:u8)->Bounds<u8>{x..=y}
pub struct Noise0;
pub struct Noise1;
pub struct Noise2;
pub struct Noise3;
pub struct Noise4;
pub struct Noise5;
pub struct Noise6;
pub struct Noise7;
pub struct Noise8;
pub struct Noise9;
pub struct Noise10;
pub struct Noise11;
pub struct Noise12;
pub struct Noise13;
pub struct Noise14;
pub struct Noise15;
pub struct Noise16;
pub struct Noise17;
pub struct Noise18;
pub struct Noise19;
pub struct Noise20;
pub struct Noise21;
pub struct Noise22;
pub struct Noise23;
pub struct Noise24;
pub struct Noise25;
pub struct Noise26;
pub struct Noise27;
pub struct Noise28;
pub struct Noise29;
pub struct Noise30;
pub struct Noise31;
pub struct Noise32;
pub struct Noise33;
pub struct Noise34;
pub struct Noise35;
pub struct Noise36;
pub struct Noise37;
pub struct Noise38;
pub struct Noise39;
pub struct Noise40;
pub struct Noise41;
pub struct Noise42;
pub struct Noise43;
pub struct Noise44;
pub struct Noise45;
pub struct Noise46;
pub struct Noise47;
pub struct Noise48;
pub struct Noise49;
pub struct Noise50;
pub struct Noise51;
pub struct Noise52;
pub struct Noise53;
pub struct Noise54;
pub struct Noise55;
pub struct Noise56;
pub struct Noise57;
pub struct Noise58;
pub struct Noise59;
pub struct Noise60;
pub struct Noise61;
pub struct Noise62;
pub struct Noise63;
pub struct Noise64;
pub struct Noise65;
pub struct Noise66;
pub struct Noise67;
pub struct Noise68;
pub struct Noise69;
pub struct Noise70;
pub struct Noise71;
pub struct Noise72;
pub struct Noise73;
pub struct Noise74;
pub struct Noise75;
pub struct Noise76;
pub struct Noise77;
pub struct Noise78;
pub struct Noise79;
pub struct Noise80;
pub struct Noise81;
pub struct Noise82;
pub struct Noise83;
pub struct Noise84;
pub struct Noise85;
pub struct Noise86;
pub struct Noise87;
pub struct Noise88;
pub struct Noise89;
pub struct Noise90;
pub struct Noise91;
pub struct Noise92;
pub struct Noise93;
pub struct Noise94;
pub struct Noise95;
pub struct Noise96;
pub struct Noise97;
pub struct Noise98;
pub struct Noise99;
pub struct Noise100;
pub struct Noise101;
pub struct Noise102;
pub struct Noise103;
pub struct Noise104;
pub struct Noise105;
pub struct Noise106;
pub struct Noise107;
pub struct Noise108;
pub struct Noise109;
pub struct Noise110;
pub struct Noise111;
pub struct Noise112;
pub struct Noise113;
pub struct Noise114;
pub struct Noise115;
pub struct Noise116;
pub struct Noise117;
pub struct Noise118;
pub struct Noise119;
pub struct Noise120;
pub struct Noise121;
pub struct Noise122;
pub struct Noise123;
pub struct Noise124;
pub struct Noise125;
pub struct Noise126;
pub struct Noise127;
