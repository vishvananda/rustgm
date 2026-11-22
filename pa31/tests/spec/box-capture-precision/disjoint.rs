#![feature(no_core,lang_items,intrinsics,rustc_attrs,const_trait_impl,auto_traits,unboxed_closures)]
#![no_core]
#![allow(internal_features,dead_code,unused_variables,unused_mut)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized>LegacyReceiver for &T{}
impl<T:PointeeSized>LegacyReceiver for &mut T{}
#[lang="clone"]pub const trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
impl Clone for u8{fn clone(&self)->Self{*self}}impl Copy for u8{}
impl Clone for usize{fn clone(&self)->Self{*self}}impl Copy for usize{}
impl<T:PointeeSized> Clone for *const T{fn clone(&self)->Self{*self}}impl<T:PointeeSized> Copy for *const T{}
#[lang="unsize"]#[rustc_deny_explicit_impl]#[rustc_dyn_incompatible_trait]pub trait Unsize<T:PointeeSized>:PointeeSized{}
#[lang="coerce_unsized"]pub trait CoerceUnsized<T:PointeeSized>:Sized{}
#[lang="dispatch_from_dyn"]pub trait DispatchFromDyn<T>:Sized{}
impl<'a,T:PointeeSized+Unsize<U>,U:PointeeSized>DispatchFromDyn<&'a U> for &'a T{}
#[lang="freeze"]pub unsafe auto trait Freeze{}
impl<T:PointeeSized+Unsize<U>,U:PointeeSized> CoerceUnsized<*const U> for *const T{}
#[lang="drop"]pub const trait Drop{fn drop(&mut self);}
#[lang="destruct"]#[rustc_deny_explicit_impl]pub const trait Destruct:PointeeSized{}
#[lang="drop_glue"]pub(crate) const unsafe fn drop_glue<T:PointeeSized>(_:&mut T)where T:[const]Destruct{}
#[lang="phantom_data"]pub struct PhantomData<T:PointeeSized>;
#[lang="manually_drop"]#[repr(transparent)]pub struct ManuallyDrop<T:?Sized>{value:T}
pub mod intrinsics{use crate::Sized;#[rustc_intrinsic]pub const fn size_of<T>()->usize;}
unsafe extern "C"{fn host_alloc(size:usize)->*mut u8;fn host_free(pointer:*mut u8);fn host_drop(value:u8);}
pub struct NonNull<T:PointeeSized>{pointer:*const T}
impl<T:PointeeSized+Unsize<U>,U:PointeeSized> CoerceUnsized<NonNull<U>> for NonNull<T>{}
pub struct Unique<T:PointeeSized>{pointer:NonNull<T>,marker:PhantomData<T>}
impl<T:PointeeSized+Unsize<U>,U:PointeeSized> CoerceUnsized<Unique<U>> for Unique<T>{}
pub struct Global;
#[lang="owned_box"]pub struct Box<T:?Sized,A=Global>(Unique<T>,A);
impl<T:?Sized+Unsize<U>,U:?Sized,A> CoerceUnsized<Box<U,A>> for Box<T,A>{}
impl<T:?Sized,A> LegacyReceiver for Box<T,A>{}
impl<T> Box<T>{pub fn new(value:T)->Box<T>{unsafe{let pointer=host_alloc(const{intrinsics::size_of::<T>()}) as *mut ManuallyDrop<T>;*pointer=ManuallyDrop{value};Box(Unique{pointer:NonNull{pointer:pointer as *const T},marker:PhantomData},Global)}}}
impl<T:?Sized,A> Drop for Box<T,A>{fn drop(&mut self){unsafe{host_free(self.0.pointer.pointer as *mut u8)}}}
pub struct Noisy(pub u8);
impl Drop for Noisy{fn drop(&mut self){unsafe{host_drop(self.0)}}}
pub struct Pair{pub left:Noisy,pub right:Noisy}
#[lang = "tuple_trait"]
#[rustc_deny_explicit_impl]
pub trait Tuple {}
#[lang = "fn_once"]
pub const trait FnOnce<Args: Tuple> {
    #[lang = "fn_once_output"]
    type Output;
    extern "rust-call" fn call_once(self, args: Args) -> Self::Output;
}
#[lang = "fn_mut"]
pub const trait FnMut<Args: Tuple>: FnOnce<Args> {
    extern "rust-call" fn call_mut(&mut self, args: Args) -> Self::Output;
}
#[lang = "fn"]
pub const trait Fn<Args: Tuple>: [const] FnMut<Args> {
    extern "rust-call" fn call(&self, args: Args) -> Self::Output;
}

fn update_fields(boxed: &mut Box<Pair>) -> u8 {
    let mut update = || {
        boxed.left.0 = 7;
    };
    boxed.right.0 = 9;
    update();
    boxed.left.0
}

#[cfg(moving_box)]
fn move_captures_box(mut boxed: Box<Pair>) -> Noisy {
    let take = || boxed.left;
    boxed.right.0 = 9;
    take()
}

#[unsafe(no_mangle)]
pub extern "C" fn run() -> u8 {
    let mut boxed = Box::new(Pair { left: Noisy(1), right: Noisy(2) });
    update_fields(&mut boxed)
}

fn main() {}
