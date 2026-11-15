#![feature(const_closures)]
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
pub const fn check0<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check1<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check2<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check3<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check4<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check5<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check6<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check7<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check8<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check9<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check10<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check11<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check12<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check13<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check14<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check15<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check16<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check17<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check18<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check19<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check20<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check21<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check22<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check23<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check24<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check25<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check26<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check27<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check28<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check29<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check30<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check31<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check32<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check33<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check34<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check35<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check36<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check37<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check38<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check39<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check40<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check41<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check42<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check43<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check44<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check45<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check46<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check47<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check48<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check49<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check50<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check51<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check52<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check53<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check54<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check55<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check56<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check57<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check58<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check59<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check60<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check61<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check62<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check63<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check64<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check65<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check66<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check67<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check68<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check69<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check70<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check71<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check72<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check73<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check74<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check75<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check76<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check77<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check78<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check79<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check80<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check81<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check82<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check83<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check84<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check85<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check86<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check87<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check88<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check89<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check90<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check91<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check92<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check93<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check94<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check95<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check96<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check97<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check98<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check99<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check100<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check101<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check102<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check103<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check104<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check105<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check106<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check107<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check108<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check109<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check110<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check111<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check112<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check113<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check114<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check115<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check116<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check117<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check118<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check119<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check120<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check121<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check122<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check123<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check124<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check125<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check126<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check127<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
fn main(){}
