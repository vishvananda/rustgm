// Authored protocol reducer; interfaces follow pinned core/ops/try_trait.rs and control_flow.rs.
#![feature(try_blocks)]
#![feature(unboxed_closures)]
#![feature(no_core,lang_items,rustc_attrs,const_trait_impl)]
#![no_core]
#![allow(internal_features,dead_code,unused_variables)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized> LegacyReceiver for &T{}
impl<T:PointeeSized> LegacyReceiver for &mut T{}
#[lang="clone"]pub const trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
impl Clone for u8{fn clone(&self)->Self{*self}}impl Copy for u8{}
#[lang="tuple_trait"]#[rustc_deny_explicit_impl]pub trait Tuple{}
#[lang="fn_once"]pub const trait FnOnce<Args:Tuple>{#[lang="fn_once_output"]type Output;extern "rust-call" fn call_once(self,args:Args)->Self::Output;}
#[lang="fn_mut"]pub const trait FnMut<Args:Tuple>:FnOnce<Args>{extern "rust-call" fn call_mut(&mut self,args:Args)->Self::Output;}
#[lang="fn"]pub const trait Fn<Args:Tuple>:[const]FnMut<Args>{extern "rust-call" fn call(&self,args:Args)->Self::Output;}

pub enum ControlFlow<B,C>{#[lang="Continue"]Continue(C),#[lang="Break"]Break(B)}
pub trait FromResidual<R=<Self as Try>::Residual>{#[lang="from_residual"]fn from_residual(value:R)->Self;}
#[lang="Try"]pub trait Try:FromResidual{
    type Output;
    type Residual:Residual<Self::Output>;
    #[lang="from_output"]fn from_output(value:Self::Output)->Self;
    #[lang="branch"]fn branch(self)->ControlFlow<Self::Residual,Self::Output>;
}
pub trait Residual<O>:Sized{type TryType:Try<Output=O,Residual=Self>;}
#[lang="into_try_type"]pub fn residual_into_try_type<R:Residual<O>,O>(value:R)-><R as Residual<O>>::TryType{
    <<R as Residual<O>>::TryType as FromResidual<R>>::from_residual(value)
}
pub enum Outcome<T>{Value(T),Error(u8)}
pub enum Short{Error(u8)}
impl<T>Residual<T> for Short{type TryType=Outcome<T>;}
impl<T>FromResidual<Short> for Outcome<T>{fn from_residual(value:Short)->Self{match value{Short::Error(e)=>Outcome::Error(e)}}}
impl<T>Try for Outcome<T>{type Output=T;type Residual=Short;
 fn from_output(value:T)->Self{Outcome::Value(value)}
 fn branch(self)->ControlFlow<Short,T>{match self{Outcome::Value(v)=>ControlFlow::Continue(v),Outcome::Error(e)=>ControlFlow::Break(Short::Error(e))}}
}
pub fn make<T>(slice:&[T])->Outcome<&[T]>{Outcome::Value(slice)}
pub fn probe0<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe1<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe2<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe3<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe4<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe5<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe6<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe7<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe8<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe9<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe10<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe11<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe12<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe13<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe14<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe15<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe16<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe17<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe18<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe19<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe20<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe21<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe22<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe23<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe24<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe25<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe26<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe27<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe28<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe29<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe30<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe31<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe32<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe33<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe34<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe35<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe36<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe37<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe38<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe39<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe40<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe41<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe42<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe43<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe44<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe45<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe46<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe47<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe48<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe49<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe50<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe51<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe52<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe53<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe54<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe55<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe56<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe57<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe58<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe59<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe60<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe61<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe62<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe63<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe64<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe65<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe66<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe67<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe68<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe69<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe70<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe71<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe72<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe73<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe74<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe75<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe76<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe77<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe78<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe79<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe80<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe81<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe82<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe83<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe84<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe85<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe86<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe87<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe88<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe89<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe90<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe91<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe92<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe93<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe94<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe95<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe96<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe97<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe98<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe99<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe100<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe101<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe102<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe103<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe104<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe105<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe106<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe107<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe108<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe109<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe110<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe111<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe112<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe113<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe114<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe115<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe116<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe117<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe118<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe119<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe120<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe121<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe122<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe123<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe124<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe125<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe126<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
pub fn probe127<T>(input:Outcome<u8>,slice:&[T])->Outcome<&[T]>{input?;make(slice)}
fn main(){}
