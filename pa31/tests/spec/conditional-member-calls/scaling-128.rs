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
#[lang="deref"]pub const trait Deref:PointeeSized{#[lang="deref_target"]type Target:PointeeSized;fn deref(&self)->&Self::Target;}
#[lang="deref_mut"]pub const trait DerefMut:[const]Deref+PointeeSized{fn deref_mut(&mut self)->&mut Self::Target;}
pub struct Smart<T>(T);
const impl<T> Deref for Smart<T>{type Target=T;fn deref(&self)->&T{&self.0}}
const impl<T> DerefMut for Smart<T>{fn deref_mut(&mut self)->&mut T{&mut self.0}}
pub struct Pair<T>{inner:T}
pub const fn field_0<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_1<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_2<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_3<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_4<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_5<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_6<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_7<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_8<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_9<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_10<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_11<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_12<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_13<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_14<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_15<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_16<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_17<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_18<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_19<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_20<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_21<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_22<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_23<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_24<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_25<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_26<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_27<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_28<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_29<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_30<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_31<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_32<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_33<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_34<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_35<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_36<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_37<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_38<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_39<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_40<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_41<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_42<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_43<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_44<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_45<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_46<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_47<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_48<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_49<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_50<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_51<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_52<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_53<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_54<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_55<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_56<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_57<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_58<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_59<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_60<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_61<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_62<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_63<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_64<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_65<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_66<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_67<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_68<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_69<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_70<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_71<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_72<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_73<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_74<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_75<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_76<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_77<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_78<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_79<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_80<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_81<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_82<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_83<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_84<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_85<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_86<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_87<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_88<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_89<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_90<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_91<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_92<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_93<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_94<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_95<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_96<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_97<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_98<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_99<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_100<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_101<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_102<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_103<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_104<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_105<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_106<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_107<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_108<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_109<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_110<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_111<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_112<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_113<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_114<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_115<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_116<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_117<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_118<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_119<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_120<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_121<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_122<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_123<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_124<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_125<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_126<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn field_127<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
