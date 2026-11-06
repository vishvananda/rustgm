#![feature(staged_api,const_trait_impl,auto_traits,negative_impls)]
#![unstable(feature="marker_fixture",issue="none")]
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

#[lang="freeze"]pub unsafe auto trait Freeze{}
impl Clone for u8{fn clone(&self)->Self{*self}}impl Copy for u8{}
#[lang="Range"]pub struct Bounds<A,B>{pub start:A,pub end:B}pub fn observe(value:u8)->Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,Bounds<u8,u8>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>{23u8..(23u8..(23u8..(23u8..(23u8..(23u8..(23u8..(23u8..(23u8..(23u8..(23u8..(23u8..(23u8..(23u8..(23u8..(23u8..(23u8..(23u8..(23u8..(23u8..(23u8..(23u8..(23u8..(23u8..(23u8..(23u8..(23u8..(23u8..(23u8..(23u8..(23u8..(23u8..(value))))))))))))))))))))))))))))))))}
pub struct Unrelated0<T>(pub T);
pub struct Unrelated1<T>(pub T);
pub struct Unrelated2<T>(pub T);
pub struct Unrelated3<T>(pub T);
pub struct Unrelated4<T>(pub T);
pub struct Unrelated5<T>(pub T);
pub struct Unrelated6<T>(pub T);
pub struct Unrelated7<T>(pub T);
pub struct Unrelated8<T>(pub T);
pub struct Unrelated9<T>(pub T);
pub struct Unrelated10<T>(pub T);
pub struct Unrelated11<T>(pub T);
pub struct Unrelated12<T>(pub T);
pub struct Unrelated13<T>(pub T);
pub struct Unrelated14<T>(pub T);
pub struct Unrelated15<T>(pub T);
pub struct Unrelated16<T>(pub T);
pub struct Unrelated17<T>(pub T);
pub struct Unrelated18<T>(pub T);
pub struct Unrelated19<T>(pub T);
pub struct Unrelated20<T>(pub T);
pub struct Unrelated21<T>(pub T);
pub struct Unrelated22<T>(pub T);
pub struct Unrelated23<T>(pub T);
pub struct Unrelated24<T>(pub T);
pub struct Unrelated25<T>(pub T);
pub struct Unrelated26<T>(pub T);
pub struct Unrelated27<T>(pub T);
pub struct Unrelated28<T>(pub T);
pub struct Unrelated29<T>(pub T);
pub struct Unrelated30<T>(pub T);
pub struct Unrelated31<T>(pub T);
pub struct Unrelated32<T>(pub T);
pub struct Unrelated33<T>(pub T);
pub struct Unrelated34<T>(pub T);
pub struct Unrelated35<T>(pub T);
pub struct Unrelated36<T>(pub T);
pub struct Unrelated37<T>(pub T);
pub struct Unrelated38<T>(pub T);
pub struct Unrelated39<T>(pub T);
pub struct Unrelated40<T>(pub T);
pub struct Unrelated41<T>(pub T);
pub struct Unrelated42<T>(pub T);
pub struct Unrelated43<T>(pub T);
pub struct Unrelated44<T>(pub T);
pub struct Unrelated45<T>(pub T);
pub struct Unrelated46<T>(pub T);
pub struct Unrelated47<T>(pub T);
pub struct Unrelated48<T>(pub T);
pub struct Unrelated49<T>(pub T);
pub struct Unrelated50<T>(pub T);
pub struct Unrelated51<T>(pub T);
pub struct Unrelated52<T>(pub T);
pub struct Unrelated53<T>(pub T);
pub struct Unrelated54<T>(pub T);
pub struct Unrelated55<T>(pub T);
pub struct Unrelated56<T>(pub T);
pub struct Unrelated57<T>(pub T);
pub struct Unrelated58<T>(pub T);
pub struct Unrelated59<T>(pub T);
pub struct Unrelated60<T>(pub T);
pub struct Unrelated61<T>(pub T);
pub struct Unrelated62<T>(pub T);
pub struct Unrelated63<T>(pub T);
pub struct Unrelated64<T>(pub T);
pub struct Unrelated65<T>(pub T);
pub struct Unrelated66<T>(pub T);
pub struct Unrelated67<T>(pub T);
pub struct Unrelated68<T>(pub T);
pub struct Unrelated69<T>(pub T);
pub struct Unrelated70<T>(pub T);
pub struct Unrelated71<T>(pub T);
pub struct Unrelated72<T>(pub T);
pub struct Unrelated73<T>(pub T);
pub struct Unrelated74<T>(pub T);
pub struct Unrelated75<T>(pub T);
pub struct Unrelated76<T>(pub T);
pub struct Unrelated77<T>(pub T);
pub struct Unrelated78<T>(pub T);
pub struct Unrelated79<T>(pub T);
pub struct Unrelated80<T>(pub T);
pub struct Unrelated81<T>(pub T);
pub struct Unrelated82<T>(pub T);
pub struct Unrelated83<T>(pub T);
pub struct Unrelated84<T>(pub T);
pub struct Unrelated85<T>(pub T);
pub struct Unrelated86<T>(pub T);
pub struct Unrelated87<T>(pub T);
pub struct Unrelated88<T>(pub T);
pub struct Unrelated89<T>(pub T);
pub struct Unrelated90<T>(pub T);
pub struct Unrelated91<T>(pub T);
pub struct Unrelated92<T>(pub T);
pub struct Unrelated93<T>(pub T);
pub struct Unrelated94<T>(pub T);
pub struct Unrelated95<T>(pub T);
pub struct Unrelated96<T>(pub T);
pub struct Unrelated97<T>(pub T);
pub struct Unrelated98<T>(pub T);
pub struct Unrelated99<T>(pub T);
pub struct Unrelated100<T>(pub T);
pub struct Unrelated101<T>(pub T);
pub struct Unrelated102<T>(pub T);
pub struct Unrelated103<T>(pub T);
pub struct Unrelated104<T>(pub T);
pub struct Unrelated105<T>(pub T);
pub struct Unrelated106<T>(pub T);
pub struct Unrelated107<T>(pub T);
pub struct Unrelated108<T>(pub T);
pub struct Unrelated109<T>(pub T);
pub struct Unrelated110<T>(pub T);
pub struct Unrelated111<T>(pub T);
pub struct Unrelated112<T>(pub T);
pub struct Unrelated113<T>(pub T);
pub struct Unrelated114<T>(pub T);
pub struct Unrelated115<T>(pub T);
pub struct Unrelated116<T>(pub T);
pub struct Unrelated117<T>(pub T);
pub struct Unrelated118<T>(pub T);
pub struct Unrelated119<T>(pub T);
pub struct Unrelated120<T>(pub T);
pub struct Unrelated121<T>(pub T);
pub struct Unrelated122<T>(pub T);
pub struct Unrelated123<T>(pub T);
pub struct Unrelated124<T>(pub T);
pub struct Unrelated125<T>(pub T);
pub struct Unrelated126<T>(pub T);
pub struct Unrelated127<T>(pub T);
