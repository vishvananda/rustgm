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
pub struct Relay<F>{pub f:F}
const impl<F:[const]FnMut(u8)->u8>FnOnce<(u8,)> for &mut Relay<F>{type Output=u8;extern "rust-call" fn call_once(mut self,args:(u8,))->u8{self.call_mut(args)}}
const impl<F:[const]FnMut(u8)->u8>FnMut<(u8,)> for &mut Relay<F>{extern "rust-call" fn call_mut(&mut self,(value,):(u8,))->u8{(self.f)(value)}}
pub const fn invoke_0<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_1<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_2<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_3<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_4<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_5<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_6<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_7<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_8<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_9<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_10<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_11<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_12<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_13<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_14<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_15<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_16<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_17<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_18<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_19<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_20<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_21<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_22<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_23<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_24<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_25<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_26<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_27<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_28<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_29<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_30<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_31<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_32<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_33<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_34<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_35<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_36<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_37<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_38<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_39<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_40<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_41<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_42<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_43<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_44<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_45<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_46<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_47<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_48<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_49<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_50<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_51<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_52<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_53<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_54<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_55<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_56<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_57<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_58<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_59<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_60<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_61<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_62<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_63<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_64<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_65<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_66<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_67<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_68<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_69<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_70<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_71<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_72<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_73<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_74<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_75<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_76<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_77<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_78<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_79<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_80<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_81<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_82<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_83<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_84<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_85<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_86<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_87<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_88<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_89<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_90<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_91<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_92<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_93<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_94<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_95<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_96<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_97<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_98<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_99<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_100<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_101<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_102<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_103<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_104<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_105<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_106<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_107<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_108<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_109<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_110<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_111<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_112<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_113<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_114<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_115<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_116<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_117<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_118<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_119<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_120<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_121<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_122<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_123<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_124<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_125<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_126<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_127<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
fn main(){}
