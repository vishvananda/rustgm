#![feature(arbitrary_self_types)]
#![feature(no_core,lang_items,rustc_attrs,staged_api)]
#![no_core]
#![unstable(feature="receiver_fixture",issue="none")]
#![allow(internal_features,dead_code,unused_variables)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized> LegacyReceiver for &T{}
impl<T:PointeeSized> LegacyReceiver for &mut T{}
#[lang="clone"]pub trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
impl Clone for u32{fn clone(&self)->Self{*self}}impl Copy for u32{}
impl<T:PointeeSized> Clone for *const T{fn clone(&self)->Self{*self}}impl<T:PointeeSized> Copy for *const T{}
impl<T:PointeeSized> Clone for *mut T{fn clone(&self)->Self{*self}}impl<T:PointeeSized> Copy for *mut T{}
#[lang="deref"]pub trait Deref:PointeeSized{#[lang="deref_target"]type Target:?Sized;fn deref(&self)->&Self::Target;}
impl<T:?Sized> Deref for &T{type Target=T;fn deref(&self)->&T{*self}}
impl<T:?Sized> Deref for &mut T{type Target=T;fn deref(&self)->&T{self}}
#[lang = "receiver"]
#[unstable(feature = "arbitrary_self_types", issue = "44874")]
pub trait Receiver: PointeeSized {
    /// The target type on which the method may be called.
    #[rustc_diagnostic_item = "receiver_target"]
    #[lang = "receiver_target"]
    #[unstable(feature = "arbitrary_self_types", issue = "44874")]
    type Target: ?Sized;
}

#[unstable(feature = "arbitrary_self_types", issue = "44874")]
impl<P: ?Sized, T: ?Sized> Receiver for P
where
    P: Deref<Target = T>,
{
    type Target = T;
}

pub struct Node{value:u32}
pub struct Pointer<T:?Sized>{pointer:*const T}
impl<T:?Sized> Receiver for Pointer<T>{type Target=T;}
impl Node{pub fn read(self:Pointer<Self>)->u32{unsafe{(*self.pointer).value}}}
pub fn probe()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}

pub fn call_0()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_1()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_2()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_3()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_4()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_5()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_6()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_7()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_8()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_9()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_10()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_11()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_12()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_13()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_14()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_15()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_16()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_17()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_18()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_19()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_20()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_21()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_22()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_23()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_24()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_25()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_26()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_27()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_28()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_29()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_30()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_31()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_32()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_33()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_34()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_35()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_36()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_37()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_38()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_39()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_40()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_41()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_42()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_43()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_44()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_45()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_46()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_47()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_48()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_49()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_50()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_51()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_52()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_53()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_54()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_55()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_56()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_57()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_58()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_59()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_60()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_61()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_62()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_63()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_64()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_65()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_66()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_67()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_68()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_69()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_70()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_71()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_72()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_73()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_74()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_75()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_76()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_77()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_78()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_79()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_80()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_81()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_82()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_83()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_84()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_85()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_86()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_87()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_88()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_89()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_90()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_91()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_92()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_93()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_94()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_95()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_96()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_97()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_98()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_99()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_100()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_101()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_102()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_103()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_104()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_105()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_106()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_107()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_108()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_109()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_110()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_111()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_112()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_113()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_114()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_115()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_116()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_117()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_118()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_119()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_120()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_121()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_122()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_123()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_124()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_125()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_126()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_127()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
