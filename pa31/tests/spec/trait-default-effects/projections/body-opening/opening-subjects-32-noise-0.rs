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

pub const trait Family<'a>{type Item0;type Item1;type Item2;type Item3;type Item4;type Item5;type Item6;type Item7;type Item8;type Item9;type Item10;type Item11;type Item12;type Item13;type Item14;type Item15;type Item16;type Item17;type Item18;type Item19;type Item20;type Item21;type Item22;type Item23;type Item24;type Item25;type Item26;type Item27;type Item28;type Item29;type Item30;type Item31;fn consume(x0:Self::Item0,x1:Self::Item1,x2:Self::Item2,x3:Self::Item3,x4:Self::Item4,x5:Self::Item5,x6:Self::Item6,x7:Self::Item7,x8:Self::Item8,x9:Self::Item9,x10:Self::Item10,x11:Self::Item11,x12:Self::Item12,x13:Self::Item13,x14:Self::Item14,x15:Self::Item15,x16:Self::Item16,x17:Self::Item17,x18:Self::Item18,x19:Self::Item19,x20:Self::Item20,x21:Self::Item21,x22:Self::Item22,x23:Self::Item23,x24:Self::Item24,x25:Self::Item25,x26:Self::Item26,x27:Self::Item27,x28:Self::Item28,x29:Self::Item29,x30:Self::Item30,x31:Self::Item31)where Self::Item0:[const]Destruct,Self::Item1:[const]Destruct,Self::Item2:[const]Destruct,Self::Item3:[const]Destruct,Self::Item4:[const]Destruct,Self::Item5:[const]Destruct,Self::Item6:[const]Destruct,Self::Item7:[const]Destruct,Self::Item8:[const]Destruct,Self::Item9:[const]Destruct,Self::Item10:[const]Destruct,Self::Item11:[const]Destruct,Self::Item12:[const]Destruct,Self::Item13:[const]Destruct,Self::Item14:[const]Destruct,Self::Item15:[const]Destruct,Self::Item16:[const]Destruct,Self::Item17:[const]Destruct,Self::Item18:[const]Destruct,Self::Item19:[const]Destruct,Self::Item20:[const]Destruct,Self::Item21:[const]Destruct,Self::Item22:[const]Destruct,Self::Item23:[const]Destruct,Self::Item24:[const]Destruct,Self::Item25:[const]Destruct,Self::Item26:[const]Destruct,Self::Item27:[const]Destruct,Self::Item28:[const]Destruct,Self::Item29:[const]Destruct,Self::Item30:[const]Destruct,Self::Item31:[const]Destruct{let _moved0=x0;let _moved1=x1;let _moved2=x2;let _moved3=x3;let _moved4=x4;let _moved5=x5;let _moved6=x6;let _moved7=x7;let _moved8=x8;let _moved9=x9;let _moved10=x10;let _moved11=x11;let _moved12=x12;let _moved13=x13;let _moved14=x14;let _moved15=x15;let _moved16=x16;let _moved17=x17;let _moved18=x18;let _moved19=x19;let _moved20=x20;let _moved21=x21;let _moved22=x22;let _moved23=x23;let _moved24=x24;let _moved25=x25;let _moved26=x26;let _moved27=x27;let _moved28=x28;let _moved29=x29;let _moved30=x30;let _moved31=x31;}}
impl<'a>Family<'a>for u8{type Item0=u8;type Item1=u8;type Item2=u8;type Item3=u8;type Item4=u8;type Item5=u8;type Item6=u8;type Item7=u8;type Item8=u8;type Item9=u8;type Item10=u8;type Item11=u8;type Item12=u8;type Item13=u8;type Item14=u8;type Item15=u8;type Item16=u8;type Item17=u8;type Item18=u8;type Item19=u8;type Item20=u8;type Item21=u8;type Item22=u8;type Item23=u8;type Item24=u8;type Item25=u8;type Item26=u8;type Item27=u8;type Item28=u8;type Item29=u8;type Item30=u8;type Item31=u8;}
#[unsafe(no_mangle)]pub extern "C" fn answer()->u8{<u8 as Family<'_>>::consume(7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8);7}
