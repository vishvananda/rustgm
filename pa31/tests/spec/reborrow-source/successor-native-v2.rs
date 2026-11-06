#![feature(no_core,lang_items,rustc_attrs,staged_api,auto_traits,const_trait_impl,reborrow)]
#![no_core]
#![unstable(feature="reborrow_fixture",issue="none")]
#![allow(internal_features,dead_code)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized>LegacyReceiver for &T{}
impl<T:PointeeSized>LegacyReceiver for &mut T{}
#[lang="freeze"]pub unsafe auto trait Freeze{}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]#[lang="destruct"]#[rustc_deny_explicit_impl]pub const trait Destruct:PointeeSized{}
#[lang="drop_glue"]pub const unsafe fn drop_glue<T:PointeeSized>(_:&mut T)where T:[const]Destruct{}
#[lang="clone"]pub trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
impl Clone for u8{fn clone(&self)->Self{*self}}impl Copy for u8{}
impl<T:PointeeSized>Clone for &T{fn clone(&self)->Self{*self}}impl<T:PointeeSized>Copy for &T{}
/// Allows value to be reborrowed as exclusive, creating a copy of the value
/// that disables the source for reads and writes for the lifetime of the copy.
#[lang = "reborrow"]
#[unstable(feature = "reborrow", issue = "145612")]
pub trait Reborrow {
    /* compiler built-in */
}

/// Allows reborrowable value to be reborrowed as shared, creating a copy
/// that disables the source for writes for the lifetime of the copy.
#[lang = "coerce_shared"]
#[unstable(feature = "reborrow", issue = "145612")]
pub trait CoerceShared<Target: Copy>: Reborrow {
    /* compiler built-in */
}

// Authored support uses the actual pinned PhantomData declaration.
#[lang = "phantom_data"]
#[stable(feature = "rust1", since = "1.0.0")]
pub struct PhantomData<T: PointeeSized>;
impl<T:PointeeSized>Clone for PhantomData<T>{fn clone(&self)->Self{Self}}
impl<T:PointeeSized>Copy for PhantomData<T>{}
struct Multi<'a,T>{value:&'a mut u8,tag:T}
impl<'a,T:Copy>Reborrow for Multi<'a,T>{}
fn concrete(value:Multi<'_,u8>,next:u8)->u8{*value.value=next;value.tag}
fn explicit<T:Copy>(value:Multi<'_,T>,next:u8)->T{*value.value=next;value.tag}
unsafe extern "C"{fn mark()->u8;}
#[unsafe(no_mangle)]pub extern "C" fn multi_concrete(value:&mut u8,next:u8)->u8{
    let value=Multi{value,tag:17u8};let _=concrete(value,next);concrete(value,next)
}
#[unsafe(no_mangle)]pub extern "C" fn multi_explicit(value:&mut u8,next:u8)->u8{
    let value=Multi{value,tag:19u8};let _=explicit::<u8>(value,next);explicit::<u8>(value,next)
}
struct InnerMut<'a>{value:&'a mut u8}
impl<'a>Reborrow for InnerMut<'a>{}
struct InnerRef<'a>{value:&'a u8}
impl Clone for InnerRef<'_>{fn clone(&self)->Self{unsafe{let _=mark();}Self{value:self.value}}}
impl Copy for InnerRef<'_>{}
impl<'a>CoerceShared<InnerRef<'a>>for InnerMut<'a>{}
struct OuterMut<'a>{inner:InnerMut<'a>}
impl<'a>Reborrow for OuterMut<'a>{}
struct OuterRef<'a>{inner:InnerRef<'a>}
impl Clone for OuterRef<'_>{fn clone(&self)->Self{unsafe{let _=mark();}Self{inner:self.inner}}}
impl Copy for OuterRef<'_>{}
impl<'a>CoerceShared<OuterRef<'a>>for OuterMut<'a>{}
fn read_outer(value:OuterRef<'_>)->u8{*value.inner.value}
#[unsafe(no_mangle)]pub extern "C" fn nested(value:&mut u8)->u8{
    let inner=InnerMut{value};let value=OuterMut{inner};let _=read_outer(value);read_outer(value)
}
struct ZeroMut<'a>{marker:PhantomData<&'a mut u8>}
impl<'a>Reborrow for ZeroMut<'a>{}
struct ZeroRef<'a>{marker:PhantomData<&'a u8>}
impl Clone for ZeroRef<'_>{fn clone(&self)->Self{unsafe{let _=mark();}Self{marker:PhantomData}}}
impl Copy for ZeroRef<'_>{}
impl<'a>CoerceShared<ZeroRef<'a>>for ZeroMut<'a>{}
fn consume_zero(_:ZeroRef<'_>){}
struct Holder<'a>{inner:Multi<'a,u8>}
fn locate<'a>(pointer:*mut Holder<'a>)->*mut Holder<'a>{unsafe{let _=mark();}pointer}
#[unsafe(no_mangle)]pub extern "C" fn projected(value:&mut u8,next:u8)->u8{
    let inner=Multi{value,tag:23u8};let mut holder=Holder{inner};
    concrete(unsafe{(*locate(&raw mut holder)).inner},next)
}
