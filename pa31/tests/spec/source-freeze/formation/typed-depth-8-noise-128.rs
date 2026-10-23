#![recursion_limit="1024"]
#![feature(no_core,lang_items,rustc_attrs,const_trait_impl,auto_traits,negative_impls,freeze_impls)]
#![no_core]
#![allow(internal_features,dead_code,unused_variables)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized>LegacyReceiver for &T{}
impl<T:PointeeSized>LegacyReceiver for &mut T{}
#[lang="clone"]pub const trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
impl Clone for u8{fn clone(&self)->Self{*self}}impl Copy for u8{}
#[lang="freeze"]pub unsafe auto trait Freeze{}
#[lang="destruct"]#[rustc_deny_explicit_impl]pub const trait Destruct:PointeeSized{}
struct W<T>(*const T);struct N;fn need<T:Freeze>()->u8{7}
type Layer0 = N;
type Layer1 = W<Layer0>;
type Layer2 = W<Layer1>;
type Layer3 = W<Layer2>;
type Layer4 = W<Layer3>;
type Layer5 = W<Layer4>;
type Layer6 = W<Layer5>;
type Layer7 = W<Layer6>;
type Layer8 = W<Layer7>;
#[unsafe(no_mangle)]pub extern "C" fn observe()->u8{need::<Layer8>()}
struct Unrelated0(u8);
struct Unrelated1(u8);
struct Unrelated2(u8);
struct Unrelated3(u8);
struct Unrelated4(u8);
struct Unrelated5(u8);
struct Unrelated6(u8);
struct Unrelated7(u8);
struct Unrelated8(u8);
struct Unrelated9(u8);
struct Unrelated10(u8);
struct Unrelated11(u8);
struct Unrelated12(u8);
struct Unrelated13(u8);
struct Unrelated14(u8);
struct Unrelated15(u8);
struct Unrelated16(u8);
struct Unrelated17(u8);
struct Unrelated18(u8);
struct Unrelated19(u8);
struct Unrelated20(u8);
struct Unrelated21(u8);
struct Unrelated22(u8);
struct Unrelated23(u8);
struct Unrelated24(u8);
struct Unrelated25(u8);
struct Unrelated26(u8);
struct Unrelated27(u8);
struct Unrelated28(u8);
struct Unrelated29(u8);
struct Unrelated30(u8);
struct Unrelated31(u8);
struct Unrelated32(u8);
struct Unrelated33(u8);
struct Unrelated34(u8);
struct Unrelated35(u8);
struct Unrelated36(u8);
struct Unrelated37(u8);
struct Unrelated38(u8);
struct Unrelated39(u8);
struct Unrelated40(u8);
struct Unrelated41(u8);
struct Unrelated42(u8);
struct Unrelated43(u8);
struct Unrelated44(u8);
struct Unrelated45(u8);
struct Unrelated46(u8);
struct Unrelated47(u8);
struct Unrelated48(u8);
struct Unrelated49(u8);
struct Unrelated50(u8);
struct Unrelated51(u8);
struct Unrelated52(u8);
struct Unrelated53(u8);
struct Unrelated54(u8);
struct Unrelated55(u8);
struct Unrelated56(u8);
struct Unrelated57(u8);
struct Unrelated58(u8);
struct Unrelated59(u8);
struct Unrelated60(u8);
struct Unrelated61(u8);
struct Unrelated62(u8);
struct Unrelated63(u8);
struct Unrelated64(u8);
struct Unrelated65(u8);
struct Unrelated66(u8);
struct Unrelated67(u8);
struct Unrelated68(u8);
struct Unrelated69(u8);
struct Unrelated70(u8);
struct Unrelated71(u8);
struct Unrelated72(u8);
struct Unrelated73(u8);
struct Unrelated74(u8);
struct Unrelated75(u8);
struct Unrelated76(u8);
struct Unrelated77(u8);
struct Unrelated78(u8);
struct Unrelated79(u8);
struct Unrelated80(u8);
struct Unrelated81(u8);
struct Unrelated82(u8);
struct Unrelated83(u8);
struct Unrelated84(u8);
struct Unrelated85(u8);
struct Unrelated86(u8);
struct Unrelated87(u8);
struct Unrelated88(u8);
struct Unrelated89(u8);
struct Unrelated90(u8);
struct Unrelated91(u8);
struct Unrelated92(u8);
struct Unrelated93(u8);
struct Unrelated94(u8);
struct Unrelated95(u8);
struct Unrelated96(u8);
struct Unrelated97(u8);
struct Unrelated98(u8);
struct Unrelated99(u8);
struct Unrelated100(u8);
struct Unrelated101(u8);
struct Unrelated102(u8);
struct Unrelated103(u8);
struct Unrelated104(u8);
struct Unrelated105(u8);
struct Unrelated106(u8);
struct Unrelated107(u8);
struct Unrelated108(u8);
struct Unrelated109(u8);
struct Unrelated110(u8);
struct Unrelated111(u8);
struct Unrelated112(u8);
struct Unrelated113(u8);
struct Unrelated114(u8);
struct Unrelated115(u8);
struct Unrelated116(u8);
struct Unrelated117(u8);
struct Unrelated118(u8);
struct Unrelated119(u8);
struct Unrelated120(u8);
struct Unrelated121(u8);
struct Unrelated122(u8);
struct Unrelated123(u8);
struct Unrelated124(u8);
struct Unrelated125(u8);
struct Unrelated126(u8);
struct Unrelated127(u8);
