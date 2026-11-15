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
pub const fn wrap0<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap1<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap2<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap3<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap4<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap5<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap6<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap7<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap8<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap9<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap10<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap11<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap12<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap13<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap14<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap15<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap16<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap17<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap18<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap19<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap20<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap21<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap22<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap23<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap24<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap25<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap26<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap27<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap28<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap29<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap30<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap31<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap32<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap33<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap34<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap35<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap36<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap37<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap38<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap39<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap40<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap41<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap42<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap43<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap44<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap45<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap46<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap47<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap48<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap49<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap50<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap51<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap52<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap53<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap54<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap55<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap56<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap57<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap58<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap59<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap60<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap61<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap62<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap63<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap64<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap65<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap66<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap67<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap68<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap69<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap70<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap71<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap72<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap73<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap74<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap75<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap76<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap77<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap78<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap79<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap80<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap81<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap82<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap83<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap84<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap85<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap86<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap87<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap88<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap89<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap90<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap91<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap92<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap93<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap94<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap95<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap96<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap97<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap98<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap99<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap100<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap101<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap102<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap103<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap104<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap105<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap106<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap107<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap108<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap109<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap110<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap111<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap112<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap113<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap114<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap115<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap116<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap117<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap118<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap119<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap120<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap121<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap122<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap123<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap124<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap125<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap126<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap127<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
fn main(){}
