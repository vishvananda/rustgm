#![feature(const_trait_impl,const_destruct)]
#![feature(no_core,lang_items,rustc_attrs,intrinsics,auto_traits)]
#![no_core]
#![allow(internal_features,dead_code)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="freeze"]pub unsafe auto trait Freeze{}
#[lang="clone"]pub trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized>LegacyReceiver for &T{}
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
#[lang="sync"]pub unsafe auto trait Sync{}
#[lang="destruct"]#[rustc_deny_explicit_impl]pub const trait Destruct:PointeeSized{}
#[lang="drop_glue"]pub(crate) const unsafe fn drop_glue<T:PointeeSized>(_:&mut T)where T:[const]Destruct{}
#[lang="unsize"]#[rustc_deny_explicit_impl]#[rustc_dyn_incompatible_trait]pub trait Unsize<T:PointeeSized>:PointeeSized{}
#[lang="coerce_unsized"]pub trait CoerceUnsized<T:PointeeSized>:Sized{}
impl<'a,'b:'a,T:PointeeSized+Unsize<U>,U:PointeeSized>CoerceUnsized<&'a U> for &'b T{}

pub trait Has{type Meta:Copy;}pub const fn consume<T:Has>(x0:T::Meta,x1:T::Meta,x2:T::Meta,x3:T::Meta,x4:T::Meta,x5:T::Meta,x6:T::Meta,x7:T::Meta,x8:T::Meta,x9:T::Meta,x10:T::Meta,x11:T::Meta,x12:T::Meta,x13:T::Meta,x14:T::Meta,x15:T::Meta,x16:T::Meta,x17:T::Meta,x18:T::Meta,x19:T::Meta,x20:T::Meta,x21:T::Meta,x22:T::Meta,x23:T::Meta,x24:T::Meta,x25:T::Meta,x26:T::Meta,x27:T::Meta,x28:T::Meta,x29:T::Meta,x30:T::Meta,x31:T::Meta,x32:T::Meta,x33:T::Meta,x34:T::Meta,x35:T::Meta,x36:T::Meta,x37:T::Meta,x38:T::Meta,x39:T::Meta,x40:T::Meta,x41:T::Meta,x42:T::Meta,x43:T::Meta,x44:T::Meta,x45:T::Meta,x46:T::Meta,x47:T::Meta,x48:T::Meta,x49:T::Meta,x50:T::Meta,x51:T::Meta,x52:T::Meta,x53:T::Meta,x54:T::Meta,x55:T::Meta,x56:T::Meta,x57:T::Meta,x58:T::Meta,x59:T::Meta,x60:T::Meta,x61:T::Meta,x62:T::Meta,x63:T::Meta,x64:T::Meta,x65:T::Meta,x66:T::Meta,x67:T::Meta,x68:T::Meta,x69:T::Meta,x70:T::Meta,x71:T::Meta,x72:T::Meta,x73:T::Meta,x74:T::Meta,x75:T::Meta,x76:T::Meta,x77:T::Meta,x78:T::Meta,x79:T::Meta,x80:T::Meta,x81:T::Meta,x82:T::Meta,x83:T::Meta,x84:T::Meta,x85:T::Meta,x86:T::Meta,x87:T::Meta,x88:T::Meta,x89:T::Meta,x90:T::Meta,x91:T::Meta,x92:T::Meta,x93:T::Meta,x94:T::Meta,x95:T::Meta,x96:T::Meta,x97:T::Meta,x98:T::Meta,x99:T::Meta,x100:T::Meta,x101:T::Meta,x102:T::Meta,x103:T::Meta,x104:T::Meta,x105:T::Meta,x106:T::Meta,x107:T::Meta,x108:T::Meta,x109:T::Meta,x110:T::Meta,x111:T::Meta,x112:T::Meta,x113:T::Meta,x114:T::Meta,x115:T::Meta,x116:T::Meta,x117:T::Meta,x118:T::Meta,x119:T::Meta,x120:T::Meta,x121:T::Meta,x122:T::Meta,x123:T::Meta,x124:T::Meta,x125:T::Meta,x126:T::Meta,x127:T::Meta){}pub struct One;impl Has for One{type Meta=u8;}const ANSWER:u8={consume::<One>(7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7);7};
#[unsafe(no_mangle)]pub extern "C" fn observe()->u8{ANSWER}
