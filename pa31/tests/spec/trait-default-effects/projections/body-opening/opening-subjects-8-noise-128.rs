#![feature(no_core,lang_items,intrinsics,rustc_attrs,const_trait_impl,const_destruct,auto_traits)]
#![no_core]
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

pub const trait Family<'a>{type Item0;type Item1;type Item2;type Item3;type Item4;type Item5;type Item6;type Item7;fn consume(x0:Self::Item0,x1:Self::Item1,x2:Self::Item2,x3:Self::Item3,x4:Self::Item4,x5:Self::Item5,x6:Self::Item6,x7:Self::Item7)where Self::Item0:[const]Destruct,Self::Item1:[const]Destruct,Self::Item2:[const]Destruct,Self::Item3:[const]Destruct,Self::Item4:[const]Destruct,Self::Item5:[const]Destruct,Self::Item6:[const]Destruct,Self::Item7:[const]Destruct{let _moved0=x0;let _moved1=x1;let _moved2=x2;let _moved3=x3;let _moved4=x4;let _moved5=x5;let _moved6=x6;let _moved7=x7;}}
impl<'a>Family<'a>for u8{type Item0=u8;type Item1=u8;type Item2=u8;type Item3=u8;type Item4=u8;type Item5=u8;type Item6=u8;type Item7=u8;}
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
#[unsafe(no_mangle)]pub extern "C" fn answer()->u8{<u8 as Family<'_>>::consume(7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8);7}
