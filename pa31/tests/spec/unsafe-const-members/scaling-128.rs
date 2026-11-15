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
pub const trait Base<U>{fn read(value:U)->u8;}
pub const trait Bridge<U>{unsafe fn read(value:U)->u8;}
const impl<T:[const]Base<U>,U>Bridge<U>for T{unsafe fn read(value:U)->u8{<T as Base<U>>::read(value)}}
pub const fn apply0<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply1<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply2<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply3<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply4<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply5<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply6<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply7<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply8<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply9<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply10<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply11<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply12<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply13<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply14<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply15<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply16<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply17<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply18<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply19<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply20<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply21<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply22<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply23<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply24<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply25<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply26<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply27<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply28<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply29<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply30<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply31<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply32<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply33<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply34<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply35<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply36<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply37<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply38<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply39<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply40<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply41<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply42<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply43<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply44<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply45<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply46<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply47<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply48<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply49<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply50<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply51<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply52<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply53<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply54<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply55<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply56<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply57<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply58<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply59<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply60<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply61<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply62<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply63<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply64<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply65<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply66<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply67<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply68<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply69<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply70<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply71<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply72<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply73<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply74<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply75<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply76<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply77<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply78<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply79<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply80<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply81<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply82<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply83<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply84<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply85<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply86<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply87<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply88<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply89<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply90<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply91<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply92<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply93<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply94<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply95<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply96<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply97<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply98<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply99<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply100<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply101<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply102<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply103<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply104<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply105<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply106<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply107<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply108<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply109<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply110<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply111<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply112<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply113<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply114<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply115<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply116<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply117<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply118<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply119<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply120<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply121<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply122<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply123<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply124<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply125<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply126<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply127<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
fn main(){}
