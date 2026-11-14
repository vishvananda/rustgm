#![feature(no_core,lang_items,rustc_attrs,fundamental,auto_traits)]
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
pub trait Clone: Sized {  #[lang = "clone_fn"] fn clone(&self) -> Self; }
#[lang = "copy"]
pub trait Copy: Clone {}


#[lang = "freeze"]
pub unsafe auto trait Freeze {}

#[lang="future_trait"]pub trait Future{#[lang="future_output"]type Output;}
#[lang = "async_drop_in_place"]
pub async unsafe fn cleanup<T: ?Sized>(_to_drop: &mut T) {
    // Code here does not matter - this is replaced by the
    // real implementation by the compiler.
}

pub fn choice_0<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_1<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_2<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_3<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_4<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_5<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_6<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_7<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_8<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_9<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_10<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_11<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_12<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_13<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_14<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_15<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_16<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_17<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_18<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_19<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_20<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_21<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_22<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_23<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_24<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_25<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_26<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_27<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_28<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_29<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_30<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_31<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_32<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_33<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_34<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_35<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_36<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_37<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_38<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_39<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_40<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_41<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_42<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_43<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_44<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_45<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_46<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_47<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_48<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_49<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_50<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_51<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_52<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_53<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_54<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_55<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_56<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_57<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_58<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_59<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_60<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_61<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_62<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_63<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_64<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_65<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_66<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_67<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_68<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_69<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_70<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_71<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_72<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_73<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_74<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_75<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_76<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_77<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_78<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_79<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_80<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_81<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_82<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_83<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_84<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_85<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_86<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_87<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_88<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_89<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_90<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_91<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_92<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_93<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_94<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_95<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_96<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_97<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_98<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_99<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_100<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_101<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_102<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_103<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_104<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_105<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_106<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_107<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_108<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_109<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_110<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_111<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_112<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_113<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_114<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_115<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_116<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_117<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_118<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_119<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_120<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_121<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_122<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_123<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_124<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_125<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_126<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
pub fn choice_127<T:Copy>(x:bool,a:T,b:T)->T{if x{a}else{b}}
