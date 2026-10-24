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

struct D<'a>(&'a mut u8);
impl<'a> Drop for D<'a>{fn drop(&mut self){*self.0=7;}}
pub const trait Family{type Item0;type Item1;type Item2;type Item3;type Item4;type Item5;type Item6;type Item7;type Item8;type Item9;type Item10;type Item11;type Item12;type Item13;type Item14;type Item15;type Item16;type Item17;type Item18;type Item19;type Item20;type Item21;type Item22;type Item23;type Item24;type Item25;type Item26;type Item27;type Item28;type Item29;type Item30;type Item31;fn consume(_x0:Self::Item0,_x1:Self::Item1,_x2:Self::Item2,_x3:Self::Item3,_x4:Self::Item4,_x5:Self::Item5,_x6:Self::Item6,_x7:Self::Item7,_x8:Self::Item8,_x9:Self::Item9,_x10:Self::Item10,_x11:Self::Item11,_x12:Self::Item12,_x13:Self::Item13,_x14:Self::Item14,_x15:Self::Item15,_x16:Self::Item16,_x17:Self::Item17,_x18:Self::Item18,_x19:Self::Item19,_x20:Self::Item20,_x21:Self::Item21,_x22:Self::Item22,_x23:Self::Item23,_x24:Self::Item24,_x25:Self::Item25,_x26:Self::Item26,_x27:Self::Item27,_x28:Self::Item28,_x29:Self::Item29,_x30:Self::Item30,_x31:Self::Item31)where Self::Item0:[const]Destruct,Self::Item1:[const]Destruct,Self::Item2:[const]Destruct,Self::Item3:[const]Destruct,Self::Item4:[const]Destruct,Self::Item5:[const]Destruct,Self::Item6:[const]Destruct,Self::Item7:[const]Destruct,Self::Item8:[const]Destruct,Self::Item9:[const]Destruct,Self::Item10:[const]Destruct,Self::Item11:[const]Destruct,Self::Item12:[const]Destruct,Self::Item13:[const]Destruct,Self::Item14:[const]Destruct,Self::Item15:[const]Destruct,Self::Item16:[const]Destruct,Self::Item17:[const]Destruct,Self::Item18:[const]Destruct,Self::Item19:[const]Destruct,Self::Item20:[const]Destruct,Self::Item21:[const]Destruct,Self::Item22:[const]Destruct,Self::Item23:[const]Destruct,Self::Item24:[const]Destruct,Self::Item25:[const]Destruct,Self::Item26:[const]Destruct,Self::Item27:[const]Destruct,Self::Item28:[const]Destruct,Self::Item29:[const]Destruct,Self::Item30:[const]Destruct,Self::Item31:[const]Destruct{}}
impl Family for u8{type Item0=u8;type Item1=u8;type Item2=u8;type Item3=u8;type Item4=u8;type Item5=u8;type Item6=u8;type Item7=u8;type Item8=u8;type Item9=u8;type Item10=u8;type Item11=u8;type Item12=u8;type Item13=u8;type Item14=u8;type Item15=u8;type Item16=u8;type Item17=u8;type Item18=u8;type Item19=u8;type Item20=u8;type Item21=u8;type Item22=u8;type Item23=u8;type Item24=u8;type Item25=u8;type Item26=u8;type Item27=u8;type Item28=u8;type Item29=u8;type Item30=u8;type Item31=u8;}
#[unsafe(no_mangle)]pub extern "C" fn answer()->u8{<u8 as Family>::consume(7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8);7}
