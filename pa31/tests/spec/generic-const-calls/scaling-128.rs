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
pub const fn apply0<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply1<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply2<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply3<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply4<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply5<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply6<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply7<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply8<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply9<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply10<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply11<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply12<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply13<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply14<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply15<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply16<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply17<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply18<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply19<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply20<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply21<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply22<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply23<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply24<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply25<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply26<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply27<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply28<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply29<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply30<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply31<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply32<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply33<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply34<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply35<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply36<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply37<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply38<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply39<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply40<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply41<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply42<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply43<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply44<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply45<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply46<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply47<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply48<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply49<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply50<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply51<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply52<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply53<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply54<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply55<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply56<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply57<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply58<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply59<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply60<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply61<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply62<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply63<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply64<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply65<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply66<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply67<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply68<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply69<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply70<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply71<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply72<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply73<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply74<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply75<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply76<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply77<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply78<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply79<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply80<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply81<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply82<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply83<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply84<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply85<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply86<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply87<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply88<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply89<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply90<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply91<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply92<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply93<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply94<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply95<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply96<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply97<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply98<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply99<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply100<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply101<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply102<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply103<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply104<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply105<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply106<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply107<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply108<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply109<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply110<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply111<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply112<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply113<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply114<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply115<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply116<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply117<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply118<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply119<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply120<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply121<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply122<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply123<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply124<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply125<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply126<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply127<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
fn main(){}
