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
pub const trait Family{type Item0;fn consume(_x0:Self::Item0,_x1:Self::Item0,_x2:Self::Item0,_x3:Self::Item0,_x4:Self::Item0,_x5:Self::Item0,_x6:Self::Item0,_x7:Self::Item0,_x8:Self::Item0,_x9:Self::Item0,_x10:Self::Item0,_x11:Self::Item0,_x12:Self::Item0,_x13:Self::Item0,_x14:Self::Item0,_x15:Self::Item0,_x16:Self::Item0,_x17:Self::Item0,_x18:Self::Item0,_x19:Self::Item0,_x20:Self::Item0,_x21:Self::Item0,_x22:Self::Item0,_x23:Self::Item0,_x24:Self::Item0,_x25:Self::Item0,_x26:Self::Item0,_x27:Self::Item0,_x28:Self::Item0,_x29:Self::Item0,_x30:Self::Item0,_x31:Self::Item0,_x32:Self::Item0,_x33:Self::Item0,_x34:Self::Item0,_x35:Self::Item0,_x36:Self::Item0,_x37:Self::Item0,_x38:Self::Item0,_x39:Self::Item0,_x40:Self::Item0,_x41:Self::Item0,_x42:Self::Item0,_x43:Self::Item0,_x44:Self::Item0,_x45:Self::Item0,_x46:Self::Item0,_x47:Self::Item0,_x48:Self::Item0,_x49:Self::Item0,_x50:Self::Item0,_x51:Self::Item0,_x52:Self::Item0,_x53:Self::Item0,_x54:Self::Item0,_x55:Self::Item0,_x56:Self::Item0,_x57:Self::Item0,_x58:Self::Item0,_x59:Self::Item0,_x60:Self::Item0,_x61:Self::Item0,_x62:Self::Item0,_x63:Self::Item0,_x64:Self::Item0,_x65:Self::Item0,_x66:Self::Item0,_x67:Self::Item0,_x68:Self::Item0,_x69:Self::Item0,_x70:Self::Item0,_x71:Self::Item0,_x72:Self::Item0,_x73:Self::Item0,_x74:Self::Item0,_x75:Self::Item0,_x76:Self::Item0,_x77:Self::Item0,_x78:Self::Item0,_x79:Self::Item0,_x80:Self::Item0,_x81:Self::Item0,_x82:Self::Item0,_x83:Self::Item0,_x84:Self::Item0,_x85:Self::Item0,_x86:Self::Item0,_x87:Self::Item0,_x88:Self::Item0,_x89:Self::Item0,_x90:Self::Item0,_x91:Self::Item0,_x92:Self::Item0,_x93:Self::Item0,_x94:Self::Item0,_x95:Self::Item0,_x96:Self::Item0,_x97:Self::Item0,_x98:Self::Item0,_x99:Self::Item0,_x100:Self::Item0,_x101:Self::Item0,_x102:Self::Item0,_x103:Self::Item0,_x104:Self::Item0,_x105:Self::Item0,_x106:Self::Item0,_x107:Self::Item0,_x108:Self::Item0,_x109:Self::Item0,_x110:Self::Item0,_x111:Self::Item0,_x112:Self::Item0,_x113:Self::Item0,_x114:Self::Item0,_x115:Self::Item0,_x116:Self::Item0,_x117:Self::Item0,_x118:Self::Item0,_x119:Self::Item0,_x120:Self::Item0,_x121:Self::Item0,_x122:Self::Item0,_x123:Self::Item0,_x124:Self::Item0,_x125:Self::Item0,_x126:Self::Item0,_x127:Self::Item0)where Self::Item0:[const]Destruct{}}
impl Family for u8{type Item0=u8;}
#[unsafe(no_mangle)]pub extern "C" fn answer()->u8{<u8 as Family>::consume(7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8);7}
