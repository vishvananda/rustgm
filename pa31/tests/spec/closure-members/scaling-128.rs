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
pub const trait Compare{fn compare(&self,other:&Self)->u8;}
const impl Compare for u8{fn compare(&self,_other:&Self)->u8{*self}}
impl Compare for u16{fn compare(&self,_other:&Self)->u8{0}}
pub const fn make0<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make1<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make2<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make3<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make4<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make5<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make6<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make7<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make8<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make9<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make10<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make11<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make12<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make13<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make14<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make15<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make16<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make17<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make18<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make19<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make20<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make21<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make22<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make23<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make24<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make25<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make26<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make27<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make28<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make29<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make30<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make31<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make32<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make33<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make34<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make35<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make36<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make37<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make38<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make39<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make40<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make41<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make42<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make43<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make44<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make45<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make46<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make47<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make48<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make49<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make50<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make51<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make52<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make53<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make54<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make55<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make56<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make57<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make58<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make59<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make60<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make61<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make62<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make63<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make64<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make65<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make66<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make67<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make68<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make69<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make70<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make71<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make72<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make73<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make74<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make75<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make76<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make77<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make78<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make79<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make80<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make81<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make82<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make83<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make84<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make85<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make86<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make87<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make88<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make89<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make90<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make91<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make92<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make93<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make94<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make95<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make96<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make97<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make98<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make99<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make100<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make101<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make102<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make103<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make104<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make105<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make106<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make107<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make108<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make109<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make110<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make111<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make112<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make113<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make114<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make115<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make116<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make117<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make118<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make119<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make120<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make121<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make122<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make123<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make124<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make125<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make126<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make127<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
fn main(){}
