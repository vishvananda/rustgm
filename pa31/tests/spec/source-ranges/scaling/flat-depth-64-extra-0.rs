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
#[lang="Range"]pub struct Bounds<A,B>{pub start:A,pub end:B}pub fn observe(value:u8)->Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,u8>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>{let r1=23u8..value;let r2=23u8..r1;let r3=23u8..r2;let r4=23u8..r3;let r5=23u8..r4;let r6=23u8..r5;let r7=23u8..r6;let r8=23u8..r7;let r9=23u8..r8;let r10=23u8..r9;let r11=23u8..r10;let r12=23u8..r11;let r13=23u8..r12;let r14=23u8..r13;let r15=23u8..r14;let r16=23u8..r15;let r17=23u8..r16;let r18=23u8..r17;let r19=23u8..r18;let r20=23u8..r19;let r21=23u8..r20;let r22=23u8..r21;let r23=23u8..r22;let r24=23u8..r23;let r25=23u8..r24;let r26=23u8..r25;let r27=23u8..r26;let r28=23u8..r27;let r29=23u8..r28;let r30=23u8..r29;let r31=23u8..r30;let r32=23u8..r31;let r33=23u8..r32;let r34=23u8..r33;let r35=23u8..r34;let r36=23u8..r35;let r37=23u8..r36;let r38=23u8..r37;let r39=23u8..r38;let r40=23u8..r39;let r41=23u8..r40;let r42=23u8..r41;let r43=23u8..r42;let r44=23u8..r43;let r45=23u8..r44;let r46=23u8..r45;let r47=23u8..r46;let r48=23u8..r47;let r49=23u8..r48;let r50=23u8..r49;let r51=23u8..r50;let r52=23u8..r51;let r53=23u8..r52;let r54=23u8..r53;let r55=23u8..r54;let r56=23u8..r55;let r57=23u8..r56;let r58=23u8..r57;let r59=23u8..r58;let r60=23u8..r59;let r61=23u8..r60;let r62=23u8..r61;let r63=23u8..r62;let r64=23u8..r63;r64}
