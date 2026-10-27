#![feature(no_core,lang_items,rustc_attrs,intrinsics,core_intrinsics,auto_traits,extern_types,dispatch_from_dyn,pattern_types,pattern_type_macro,staged_api,const_trait_impl,const_destruct,unsize,coerce_unsized)]
#![no_core]
#![unstable(feature="pointer_metadata_fixture",issue="none")]
#![allow(internal_features,dead_code)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized>LegacyReceiver for &T{}
impl<T:PointeeSized>LegacyReceiver for &mut T{}
#[lang="clone"]pub trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
#[lang="freeze"]pub unsafe auto trait Freeze{}
#[unstable(feature="const_destruct",issue="133214")]
#[rustc_const_unstable(feature="const_destruct",issue="133214")]
#[lang="destruct"]pub const trait Destruct:PointeeSized{}
#[unstable(feature="unsize",issue="18598")]
#[lang="unsize"]#[rustc_deny_explicit_impl]#[rustc_dyn_incompatible_trait]
pub trait Unsize<T:PointeeSized>:PointeeSized{}
#[unstable(feature="coerce_unsized",issue="18598")]
#[lang="coerce_unsized"]pub trait CoerceUnsized<T:PointeeSized>:Sized{}
#[unstable(feature="coerce_unsized",issue="18598")]
impl<T:PointeeSized+Unsize<U>,U:PointeeSized>CoerceUnsized<*const U> for *const T{}
impl<T:PointeeSized>Clone for *const T{fn clone(&self)->Self{*self}}
impl<T:PointeeSized>Copy for *const T{}
impl Clone for usize{fn clone(&self)->Self{*self}}
impl Copy for usize{}
impl Clone for u8{fn clone(&self)->Self{*self}}
impl Copy for u8{}
#[lang="pointee_trait"]pub trait Pointee:PointeeSized{
 #[lang="metadata_type"]type Metadata:Sized;
}
#[unstable(feature="core_intrinsics",issue="none")]
#[rustc_nounwind]
#[rustc_intrinsic_const_stable_indirect]
#[rustc_intrinsic]
pub const fn ptr_metadata<P:Pointee<Metadata=M>+PointeeSized,M>(ptr:*const P)->M;
// core::ptr::metadata's actual source wrapper body; generic result remains a projection.
pub const fn metadata<T:PointeeSized>(ptr:*const T)-><T as Pointee>::Metadata{ptr_metadata(ptr)}

#[unstable(feature="pattern_type_macro",issue="123646")]
#[rustc_builtin_macro(pattern_type)]
macro_rules! pattern_type{($($args:tt)*)=>{}}
#[lang="phantom_data"]pub struct PhantomData<T:PointeeSized>;
#[repr(transparent)]
#[rustc_nonnull_optimization_guaranteed]
#[rustc_diagnostic_item="NonNull"]
pub struct NonNull<T:PointeeSized>{pointer:pattern_type!(*const T is !null)}
unsafe extern "C"{type VTable;}
#[lang="dyn_metadata"]
pub struct DynMetadata<Dyn:PointeeSized>{_vtable_ptr:NonNull<VTable>,_phantom:PhantomData<Dyn>}
#[unstable(feature="dispatch_from_dyn",issue="none")]
#[lang="dispatch_from_dyn"]pub trait DispatchFromDyn<T>:Sized{}
#[unstable(feature="dispatch_from_dyn",issue="none")]
impl<'a,T:PointeeSized+Unsize<U>,U:PointeeSized>DispatchFromDyn<&'a U> for &'a T{}
pub trait Value:PointeeSized{fn value(&self)->u8;}
pub fn actual_dynamic(ptr:*const dyn Value)->DynMetadata<dyn Value>{metadata(ptr)}
