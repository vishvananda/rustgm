#![feature(const_destruct)]
#![feature(auto_traits)]
#![feature(unboxed_closures,const_trait_impl)]
#![feature(no_core,lang_items,rustc_attrs,fundamental)]
#![no_core]
#![allow(internal_features,dead_code,unused_variables)]
#[lang = "pointee_sized"]
#[diagnostic::on_unimplemented(
    message = "values of type `{Self}` may or may not have a size",
    label = "may or may not have a known size"
)]
#[fundamental]
#[rustc_specialization_trait]
#[rustc_deny_explicit_impl]
#[rustc_coinductive]
pub trait PointeeSized {
    // Empty
}
#[lang = "meta_sized"]
#[diagnostic::on_unimplemented(
    message = "the size for values of type `{Self}` cannot be known",
    label = "doesn't have a known size"
)]
#[fundamental]
#[rustc_specialization_trait]
#[rustc_deny_explicit_impl]
// `MetaSized` being coinductive, despite having supertraits, is okay for the same reasons as
// `Sized` above.
#[rustc_coinductive]
pub trait MetaSized: PointeeSized {
    // Empty
}
#[lang = "sized"]
#[diagnostic::on_unimplemented(
    message = "the size for values of type `{Self}` cannot be known at compilation time",
    label = "doesn't have a size known at compile-time"
)]
#[fundamental] // for Default, for example, which requires that `[T]: !Default` be evaluatable
#[rustc_specialization_trait]
#[rustc_deny_explicit_impl]
#[rustc_dyn_incompatible_trait]
// `Sized` being coinductive, despite having supertraits, is okay as there are no user-written impls,
// and we know that the supertraits are always implemented if the subtrait is just by looking at
// the builtin impls.
#[rustc_coinductive]
pub trait Sized: MetaSized {
    // Empty.
}
#[lang = "legacy_receiver"]
pub trait LegacyReceiver: PointeeSized {}
impl<T: PointeeSized> LegacyReceiver for &T {}
impl<T: PointeeSized> LegacyReceiver for &mut T {}
#[lang = "clone"]
pub trait Clone: Sized { #[lang = "clone_fn"] fn clone(&self) -> Self; }
#[lang = "copy"]
pub trait Copy: Clone {}
pub fn identity<T>(x: T) -> T { x }

impl Clone for u8{fn clone(&self)->Self{*self}}impl Copy for u8{}

impl<T:PointeeSized> Clone for &T{fn clone(&self)->Self{*self}}
impl<T:PointeeSized> Copy for &T{}

#[lang="freeze"]pub unsafe auto trait Freeze{}

#[lang="destruct"]#[rustc_deny_explicit_impl]#[rustc_dyn_incompatible_trait]
pub const trait Destruct:PointeeSized{}
#[lang="drop_glue"]pub const unsafe fn drop_glue<T:PointeeSized>(_:&mut T)where T:[const]Destruct{}
#[lang="tuple_trait"]#[rustc_deny_explicit_impl]pub trait Tuple{}
#[lang="fn_once"]pub const trait FnOnce<Args:Tuple>{#[lang="fn_once_output"]type Output;extern "rust-call" fn call_once(self,args:Args)->Self::Output;}
#[lang="fn_mut"]pub const trait FnMut<Args:Tuple>:FnOnce<Args>{extern "rust-call" fn call_mut(&mut self,args:Args)->Self::Output;}
#[lang="fn"]pub const trait Fn<Args:Tuple>:[const]FnMut<Args>{extern "rust-call" fn call(&self,args:Args)->Self::Output;}

#[lang="drop"]pub const trait Drop{fn drop(&mut self);}
pub const trait Base{fn read(&self)->u8;}
pub const trait Derived:[const]Base{
fn through_0(&self)->u8{Base::read(self)}
fn through_1(&self)->u8{Base::read(self)}
fn through_2(&self)->u8{Base::read(self)}
fn through_3(&self)->u8{Base::read(self)}
fn through_4(&self)->u8{Base::read(self)}
fn through_5(&self)->u8{Base::read(self)}
fn through_6(&self)->u8{Base::read(self)}
fn through_7(&self)->u8{Base::read(self)}
fn through_8(&self)->u8{Base::read(self)}
fn through_9(&self)->u8{Base::read(self)}
fn through_10(&self)->u8{Base::read(self)}
fn through_11(&self)->u8{Base::read(self)}
fn through_12(&self)->u8{Base::read(self)}
fn through_13(&self)->u8{Base::read(self)}
fn through_14(&self)->u8{Base::read(self)}
fn through_15(&self)->u8{Base::read(self)}
fn through_16(&self)->u8{Base::read(self)}
fn through_17(&self)->u8{Base::read(self)}
fn through_18(&self)->u8{Base::read(self)}
fn through_19(&self)->u8{Base::read(self)}
fn through_20(&self)->u8{Base::read(self)}
fn through_21(&self)->u8{Base::read(self)}
fn through_22(&self)->u8{Base::read(self)}
fn through_23(&self)->u8{Base::read(self)}
fn through_24(&self)->u8{Base::read(self)}
fn through_25(&self)->u8{Base::read(self)}
fn through_26(&self)->u8{Base::read(self)}
fn through_27(&self)->u8{Base::read(self)}
fn through_28(&self)->u8{Base::read(self)}
fn through_29(&self)->u8{Base::read(self)}
fn through_30(&self)->u8{Base::read(self)}
fn through_31(&self)->u8{Base::read(self)}
fn through_32(&self)->u8{Base::read(self)}
fn through_33(&self)->u8{Base::read(self)}
fn through_34(&self)->u8{Base::read(self)}
fn through_35(&self)->u8{Base::read(self)}
fn through_36(&self)->u8{Base::read(self)}
fn through_37(&self)->u8{Base::read(self)}
fn through_38(&self)->u8{Base::read(self)}
fn through_39(&self)->u8{Base::read(self)}
fn through_40(&self)->u8{Base::read(self)}
fn through_41(&self)->u8{Base::read(self)}
fn through_42(&self)->u8{Base::read(self)}
fn through_43(&self)->u8{Base::read(self)}
fn through_44(&self)->u8{Base::read(self)}
fn through_45(&self)->u8{Base::read(self)}
fn through_46(&self)->u8{Base::read(self)}
fn through_47(&self)->u8{Base::read(self)}
fn through_48(&self)->u8{Base::read(self)}
fn through_49(&self)->u8{Base::read(self)}
fn through_50(&self)->u8{Base::read(self)}
fn through_51(&self)->u8{Base::read(self)}
fn through_52(&self)->u8{Base::read(self)}
fn through_53(&self)->u8{Base::read(self)}
fn through_54(&self)->u8{Base::read(self)}
fn through_55(&self)->u8{Base::read(self)}
fn through_56(&self)->u8{Base::read(self)}
fn through_57(&self)->u8{Base::read(self)}
fn through_58(&self)->u8{Base::read(self)}
fn through_59(&self)->u8{Base::read(self)}
fn through_60(&self)->u8{Base::read(self)}
fn through_61(&self)->u8{Base::read(self)}
fn through_62(&self)->u8{Base::read(self)}
fn through_63(&self)->u8{Base::read(self)}
fn through_64(&self)->u8{Base::read(self)}
fn through_65(&self)->u8{Base::read(self)}
fn through_66(&self)->u8{Base::read(self)}
fn through_67(&self)->u8{Base::read(self)}
fn through_68(&self)->u8{Base::read(self)}
fn through_69(&self)->u8{Base::read(self)}
fn through_70(&self)->u8{Base::read(self)}
fn through_71(&self)->u8{Base::read(self)}
fn through_72(&self)->u8{Base::read(self)}
fn through_73(&self)->u8{Base::read(self)}
fn through_74(&self)->u8{Base::read(self)}
fn through_75(&self)->u8{Base::read(self)}
fn through_76(&self)->u8{Base::read(self)}
fn through_77(&self)->u8{Base::read(self)}
fn through_78(&self)->u8{Base::read(self)}
fn through_79(&self)->u8{Base::read(self)}
fn through_80(&self)->u8{Base::read(self)}
fn through_81(&self)->u8{Base::read(self)}
fn through_82(&self)->u8{Base::read(self)}
fn through_83(&self)->u8{Base::read(self)}
fn through_84(&self)->u8{Base::read(self)}
fn through_85(&self)->u8{Base::read(self)}
fn through_86(&self)->u8{Base::read(self)}
fn through_87(&self)->u8{Base::read(self)}
fn through_88(&self)->u8{Base::read(self)}
fn through_89(&self)->u8{Base::read(self)}
fn through_90(&self)->u8{Base::read(self)}
fn through_91(&self)->u8{Base::read(self)}
fn through_92(&self)->u8{Base::read(self)}
fn through_93(&self)->u8{Base::read(self)}
fn through_94(&self)->u8{Base::read(self)}
fn through_95(&self)->u8{Base::read(self)}
fn through_96(&self)->u8{Base::read(self)}
fn through_97(&self)->u8{Base::read(self)}
fn through_98(&self)->u8{Base::read(self)}
fn through_99(&self)->u8{Base::read(self)}
fn through_100(&self)->u8{Base::read(self)}
fn through_101(&self)->u8{Base::read(self)}
fn through_102(&self)->u8{Base::read(self)}
fn through_103(&self)->u8{Base::read(self)}
fn through_104(&self)->u8{Base::read(self)}
fn through_105(&self)->u8{Base::read(self)}
fn through_106(&self)->u8{Base::read(self)}
fn through_107(&self)->u8{Base::read(self)}
fn through_108(&self)->u8{Base::read(self)}
fn through_109(&self)->u8{Base::read(self)}
fn through_110(&self)->u8{Base::read(self)}
fn through_111(&self)->u8{Base::read(self)}
fn through_112(&self)->u8{Base::read(self)}
fn through_113(&self)->u8{Base::read(self)}
fn through_114(&self)->u8{Base::read(self)}
fn through_115(&self)->u8{Base::read(self)}
fn through_116(&self)->u8{Base::read(self)}
fn through_117(&self)->u8{Base::read(self)}
fn through_118(&self)->u8{Base::read(self)}
fn through_119(&self)->u8{Base::read(self)}
fn through_120(&self)->u8{Base::read(self)}
fn through_121(&self)->u8{Base::read(self)}
fn through_122(&self)->u8{Base::read(self)}
fn through_123(&self)->u8{Base::read(self)}
fn through_124(&self)->u8{Base::read(self)}
fn through_125(&self)->u8{Base::read(self)}
fn through_126(&self)->u8{Base::read(self)}
fn through_127(&self)->u8{Base::read(self)}
}
fn main(){}
