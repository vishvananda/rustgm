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

pub struct Tag<const N:usize>;
pub fn make_0<const N:usize>()->Tag<N>{Tag}
pub fn make_1<const N:usize>()->Tag<N>{Tag}
pub fn make_2<const N:usize>()->Tag<N>{Tag}
pub fn make_3<const N:usize>()->Tag<N>{Tag}
pub fn make_4<const N:usize>()->Tag<N>{Tag}
pub fn make_5<const N:usize>()->Tag<N>{Tag}
pub fn make_6<const N:usize>()->Tag<N>{Tag}
pub fn make_7<const N:usize>()->Tag<N>{Tag}
pub fn make_8<const N:usize>()->Tag<N>{Tag}
pub fn make_9<const N:usize>()->Tag<N>{Tag}
pub fn make_10<const N:usize>()->Tag<N>{Tag}
pub fn make_11<const N:usize>()->Tag<N>{Tag}
pub fn make_12<const N:usize>()->Tag<N>{Tag}
pub fn make_13<const N:usize>()->Tag<N>{Tag}
pub fn make_14<const N:usize>()->Tag<N>{Tag}
pub fn make_15<const N:usize>()->Tag<N>{Tag}
pub fn make_16<const N:usize>()->Tag<N>{Tag}
pub fn make_17<const N:usize>()->Tag<N>{Tag}
pub fn make_18<const N:usize>()->Tag<N>{Tag}
pub fn make_19<const N:usize>()->Tag<N>{Tag}
pub fn make_20<const N:usize>()->Tag<N>{Tag}
pub fn make_21<const N:usize>()->Tag<N>{Tag}
pub fn make_22<const N:usize>()->Tag<N>{Tag}
pub fn make_23<const N:usize>()->Tag<N>{Tag}
pub fn make_24<const N:usize>()->Tag<N>{Tag}
pub fn make_25<const N:usize>()->Tag<N>{Tag}
pub fn make_26<const N:usize>()->Tag<N>{Tag}
pub fn make_27<const N:usize>()->Tag<N>{Tag}
pub fn make_28<const N:usize>()->Tag<N>{Tag}
pub fn make_29<const N:usize>()->Tag<N>{Tag}
pub fn make_30<const N:usize>()->Tag<N>{Tag}
pub fn make_31<const N:usize>()->Tag<N>{Tag}
pub fn make_32<const N:usize>()->Tag<N>{Tag}
pub fn make_33<const N:usize>()->Tag<N>{Tag}
pub fn make_34<const N:usize>()->Tag<N>{Tag}
pub fn make_35<const N:usize>()->Tag<N>{Tag}
pub fn make_36<const N:usize>()->Tag<N>{Tag}
pub fn make_37<const N:usize>()->Tag<N>{Tag}
pub fn make_38<const N:usize>()->Tag<N>{Tag}
pub fn make_39<const N:usize>()->Tag<N>{Tag}
pub fn make_40<const N:usize>()->Tag<N>{Tag}
pub fn make_41<const N:usize>()->Tag<N>{Tag}
pub fn make_42<const N:usize>()->Tag<N>{Tag}
pub fn make_43<const N:usize>()->Tag<N>{Tag}
pub fn make_44<const N:usize>()->Tag<N>{Tag}
pub fn make_45<const N:usize>()->Tag<N>{Tag}
pub fn make_46<const N:usize>()->Tag<N>{Tag}
pub fn make_47<const N:usize>()->Tag<N>{Tag}
pub fn make_48<const N:usize>()->Tag<N>{Tag}
pub fn make_49<const N:usize>()->Tag<N>{Tag}
pub fn make_50<const N:usize>()->Tag<N>{Tag}
pub fn make_51<const N:usize>()->Tag<N>{Tag}
pub fn make_52<const N:usize>()->Tag<N>{Tag}
pub fn make_53<const N:usize>()->Tag<N>{Tag}
pub fn make_54<const N:usize>()->Tag<N>{Tag}
pub fn make_55<const N:usize>()->Tag<N>{Tag}
pub fn make_56<const N:usize>()->Tag<N>{Tag}
pub fn make_57<const N:usize>()->Tag<N>{Tag}
pub fn make_58<const N:usize>()->Tag<N>{Tag}
pub fn make_59<const N:usize>()->Tag<N>{Tag}
pub fn make_60<const N:usize>()->Tag<N>{Tag}
pub fn make_61<const N:usize>()->Tag<N>{Tag}
pub fn make_62<const N:usize>()->Tag<N>{Tag}
pub fn make_63<const N:usize>()->Tag<N>{Tag}
pub fn make_64<const N:usize>()->Tag<N>{Tag}
pub fn make_65<const N:usize>()->Tag<N>{Tag}
pub fn make_66<const N:usize>()->Tag<N>{Tag}
pub fn make_67<const N:usize>()->Tag<N>{Tag}
pub fn make_68<const N:usize>()->Tag<N>{Tag}
pub fn make_69<const N:usize>()->Tag<N>{Tag}
pub fn make_70<const N:usize>()->Tag<N>{Tag}
pub fn make_71<const N:usize>()->Tag<N>{Tag}
pub fn make_72<const N:usize>()->Tag<N>{Tag}
pub fn make_73<const N:usize>()->Tag<N>{Tag}
pub fn make_74<const N:usize>()->Tag<N>{Tag}
pub fn make_75<const N:usize>()->Tag<N>{Tag}
pub fn make_76<const N:usize>()->Tag<N>{Tag}
pub fn make_77<const N:usize>()->Tag<N>{Tag}
pub fn make_78<const N:usize>()->Tag<N>{Tag}
pub fn make_79<const N:usize>()->Tag<N>{Tag}
pub fn make_80<const N:usize>()->Tag<N>{Tag}
pub fn make_81<const N:usize>()->Tag<N>{Tag}
pub fn make_82<const N:usize>()->Tag<N>{Tag}
pub fn make_83<const N:usize>()->Tag<N>{Tag}
pub fn make_84<const N:usize>()->Tag<N>{Tag}
pub fn make_85<const N:usize>()->Tag<N>{Tag}
pub fn make_86<const N:usize>()->Tag<N>{Tag}
pub fn make_87<const N:usize>()->Tag<N>{Tag}
pub fn make_88<const N:usize>()->Tag<N>{Tag}
pub fn make_89<const N:usize>()->Tag<N>{Tag}
pub fn make_90<const N:usize>()->Tag<N>{Tag}
pub fn make_91<const N:usize>()->Tag<N>{Tag}
pub fn make_92<const N:usize>()->Tag<N>{Tag}
pub fn make_93<const N:usize>()->Tag<N>{Tag}
pub fn make_94<const N:usize>()->Tag<N>{Tag}
pub fn make_95<const N:usize>()->Tag<N>{Tag}
pub fn make_96<const N:usize>()->Tag<N>{Tag}
pub fn make_97<const N:usize>()->Tag<N>{Tag}
pub fn make_98<const N:usize>()->Tag<N>{Tag}
pub fn make_99<const N:usize>()->Tag<N>{Tag}
pub fn make_100<const N:usize>()->Tag<N>{Tag}
pub fn make_101<const N:usize>()->Tag<N>{Tag}
pub fn make_102<const N:usize>()->Tag<N>{Tag}
pub fn make_103<const N:usize>()->Tag<N>{Tag}
pub fn make_104<const N:usize>()->Tag<N>{Tag}
pub fn make_105<const N:usize>()->Tag<N>{Tag}
pub fn make_106<const N:usize>()->Tag<N>{Tag}
pub fn make_107<const N:usize>()->Tag<N>{Tag}
pub fn make_108<const N:usize>()->Tag<N>{Tag}
pub fn make_109<const N:usize>()->Tag<N>{Tag}
pub fn make_110<const N:usize>()->Tag<N>{Tag}
pub fn make_111<const N:usize>()->Tag<N>{Tag}
pub fn make_112<const N:usize>()->Tag<N>{Tag}
pub fn make_113<const N:usize>()->Tag<N>{Tag}
pub fn make_114<const N:usize>()->Tag<N>{Tag}
pub fn make_115<const N:usize>()->Tag<N>{Tag}
pub fn make_116<const N:usize>()->Tag<N>{Tag}
pub fn make_117<const N:usize>()->Tag<N>{Tag}
pub fn make_118<const N:usize>()->Tag<N>{Tag}
pub fn make_119<const N:usize>()->Tag<N>{Tag}
pub fn make_120<const N:usize>()->Tag<N>{Tag}
pub fn make_121<const N:usize>()->Tag<N>{Tag}
pub fn make_122<const N:usize>()->Tag<N>{Tag}
pub fn make_123<const N:usize>()->Tag<N>{Tag}
pub fn make_124<const N:usize>()->Tag<N>{Tag}
pub fn make_125<const N:usize>()->Tag<N>{Tag}
pub fn make_126<const N:usize>()->Tag<N>{Tag}
pub fn make_127<const N:usize>()->Tag<N>{Tag}
