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
pub const trait Batch<T0,T1,T2,T3,T4,T5,T6,T7>:Sized where T0:[const]Destruct,T1:[const]Destruct,T2:[const]Destruct,T3:[const]Destruct,T4:[const]Destruct,T5:[const]Destruct,T6:[const]Destruct,T7:[const]Destruct{fn consume(_x0:T0,_x1:T1,_x2:T2,_x3:T3,_x4:T4,_x5:T5,_x6:T6,_x7:T7){}}
impl<T0,T1,T2,T3,T4,T5,T6,T7> Batch<T0,T1,T2,T3,T4,T5,T6,T7> for u8{}
fn noise_0()->u8{7}
fn noise_1()->u8{7}
fn noise_2()->u8{7}
fn noise_3()->u8{7}
fn noise_4()->u8{7}
fn noise_5()->u8{7}
fn noise_6()->u8{7}
fn noise_7()->u8{7}
fn noise_8()->u8{7}
fn noise_9()->u8{7}
fn noise_10()->u8{7}
fn noise_11()->u8{7}
fn noise_12()->u8{7}
fn noise_13()->u8{7}
fn noise_14()->u8{7}
fn noise_15()->u8{7}
fn noise_16()->u8{7}
fn noise_17()->u8{7}
fn noise_18()->u8{7}
fn noise_19()->u8{7}
fn noise_20()->u8{7}
fn noise_21()->u8{7}
fn noise_22()->u8{7}
fn noise_23()->u8{7}
fn noise_24()->u8{7}
fn noise_25()->u8{7}
fn noise_26()->u8{7}
fn noise_27()->u8{7}
fn noise_28()->u8{7}
fn noise_29()->u8{7}
fn noise_30()->u8{7}
fn noise_31()->u8{7}
fn noise_32()->u8{7}
fn noise_33()->u8{7}
fn noise_34()->u8{7}
fn noise_35()->u8{7}
fn noise_36()->u8{7}
fn noise_37()->u8{7}
fn noise_38()->u8{7}
fn noise_39()->u8{7}
fn noise_40()->u8{7}
fn noise_41()->u8{7}
fn noise_42()->u8{7}
fn noise_43()->u8{7}
fn noise_44()->u8{7}
fn noise_45()->u8{7}
fn noise_46()->u8{7}
fn noise_47()->u8{7}
fn noise_48()->u8{7}
fn noise_49()->u8{7}
fn noise_50()->u8{7}
fn noise_51()->u8{7}
fn noise_52()->u8{7}
fn noise_53()->u8{7}
fn noise_54()->u8{7}
fn noise_55()->u8{7}
fn noise_56()->u8{7}
fn noise_57()->u8{7}
fn noise_58()->u8{7}
fn noise_59()->u8{7}
fn noise_60()->u8{7}
fn noise_61()->u8{7}
fn noise_62()->u8{7}
fn noise_63()->u8{7}
fn noise_64()->u8{7}
fn noise_65()->u8{7}
fn noise_66()->u8{7}
fn noise_67()->u8{7}
fn noise_68()->u8{7}
fn noise_69()->u8{7}
fn noise_70()->u8{7}
fn noise_71()->u8{7}
fn noise_72()->u8{7}
fn noise_73()->u8{7}
fn noise_74()->u8{7}
fn noise_75()->u8{7}
fn noise_76()->u8{7}
fn noise_77()->u8{7}
fn noise_78()->u8{7}
fn noise_79()->u8{7}
fn noise_80()->u8{7}
fn noise_81()->u8{7}
fn noise_82()->u8{7}
fn noise_83()->u8{7}
fn noise_84()->u8{7}
fn noise_85()->u8{7}
fn noise_86()->u8{7}
fn noise_87()->u8{7}
fn noise_88()->u8{7}
fn noise_89()->u8{7}
fn noise_90()->u8{7}
fn noise_91()->u8{7}
fn noise_92()->u8{7}
fn noise_93()->u8{7}
fn noise_94()->u8{7}
fn noise_95()->u8{7}
fn noise_96()->u8{7}
fn noise_97()->u8{7}
fn noise_98()->u8{7}
fn noise_99()->u8{7}
fn noise_100()->u8{7}
fn noise_101()->u8{7}
fn noise_102()->u8{7}
fn noise_103()->u8{7}
fn noise_104()->u8{7}
fn noise_105()->u8{7}
fn noise_106()->u8{7}
fn noise_107()->u8{7}
fn noise_108()->u8{7}
fn noise_109()->u8{7}
fn noise_110()->u8{7}
fn noise_111()->u8{7}
fn noise_112()->u8{7}
fn noise_113()->u8{7}
fn noise_114()->u8{7}
fn noise_115()->u8{7}
fn noise_116()->u8{7}
fn noise_117()->u8{7}
fn noise_118()->u8{7}
fn noise_119()->u8{7}
fn noise_120()->u8{7}
fn noise_121()->u8{7}
fn noise_122()->u8{7}
fn noise_123()->u8{7}
fn noise_124()->u8{7}
fn noise_125()->u8{7}
fn noise_126()->u8{7}
fn noise_127()->u8{7}
#[unsafe(no_mangle)]pub extern "C" fn answer()->u8{<u8 as Batch<u8,u8,u8,u8,u8,u8,u8,u8>>::consume(7u8,7u8,7u8,7u8,7u8,7u8,7u8,7u8);7}
