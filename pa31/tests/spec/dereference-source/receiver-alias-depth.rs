#![feature(no_core,lang_items,rustc_attrs,staged_api,auto_traits,const_trait_impl,prelude_import)]
#![no_core]
#![unstable(feature="async_iterator_fixture",issue="none")]
#![allow(internal_features,dead_code,unused_variables)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized>LegacyReceiver for &T{}
impl<T:PointeeSized>LegacyReceiver for &mut T{}
#[lang="freeze"]pub unsafe auto trait Freeze{}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]#[lang="destruct"]#[rustc_deny_explicit_impl]#[rustc_dyn_incompatible_trait]pub const trait Destruct:PointeeSized{}
#[lang="unpin"]pub auto trait Unpin{}
#[lang="clone"]pub trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
impl Clone for u8{fn clone(&self)->Self{*self}}impl Copy for u8{}
impl Clone for bool{fn clone(&self)->Self{*self}}impl Copy for bool{}
pub enum Option<T>{None,Some(T)}
pub mod prelude{pub use crate::{Sized,Unpin,Option,Option::{Some,None}};}
#[prelude_import]use crate::prelude::*;
pub mod ops{
#[lang="deref"]pub trait Deref:PointeeSized{#[lang="deref_target"]type Target:PointeeSized;fn deref(&self)->&Self::Target;}
#[lang="deref_mut"]pub trait DerefMut:Deref{fn deref_mut(&mut self)->&mut Self::Target;}
use crate::{PointeeSized,LegacyReceiver};
impl<T:PointeeSized>Deref for &T{type Target=T;fn deref(&self)->&T{*self}}
impl<T:PointeeSized>Deref for &mut T{type Target=T;fn deref(&self)->&T{&**self}}
impl<T:PointeeSized>DerefMut for &mut T{fn deref_mut(&mut self)->&mut T{&mut **self}}
}
pub mod pin{
use crate::{ops::{Deref,DerefMut},PointeeSized,Unpin,LegacyReceiver};
#[lang="pin"]#[repr(transparent)]pub struct Pin<P>{pub pointer:P}
impl<P>Pin<P>{#[lang="new_unchecked"]pub const unsafe fn new_unchecked(pointer:P)->Self{Self{pointer}}}
impl<P:Deref>Pin<P>{pub fn new(pointer:P)->Self where P::Target:Unpin{Self{pointer}}}
impl<P:Deref>Deref for Pin<P>{type Target=P::Target;fn deref(&self)->&Self::Target{self.pointer.deref()}}
impl<P:DerefMut>DerefMut for Pin<P> where P::Target:Unpin{fn deref_mut(&mut self)->&mut Self::Target{self.pointer.deref_mut()}}
impl<P:DerefMut>Pin<P>{pub fn as_deref_mut(self:Pin<&mut Self>)->Pin<&mut P::Target>{unsafe{Pin::new_unchecked(self.pointer.pointer.deref_mut())}}}
impl<P:Deref>LegacyReceiver for Pin<P>{}
}
pub mod task{
#[lang="Poll"]pub enum Poll<T>{#[lang="Ready"]Ready(T),#[lang="Pending"]Pending}
#[lang="Context"]pub struct Context<'a>{pub token:&'a ()}
}


type Layer0<'a,T>=&'a mut T;
type Layer1<'a,T>=pin::Pin<Layer0<'a,T>>;
type Layer2<'a,T>=pin::Pin<Layer1<'a,T>>;
type Layer3<'a,T>=pin::Pin<Layer2<'a,T>>;
type Layer4<'a,T>=pin::Pin<Layer3<'a,T>>;
type Layer5<'a,T>=pin::Pin<Layer4<'a,T>>;
type Layer6<'a,T>=pin::Pin<Layer5<'a,T>>;
type Layer7<'a,T>=pin::Pin<Layer6<'a,T>>;
type Layer8<'a,T>=pin::Pin<Layer7<'a,T>>;
type Layer9<'a,T>=pin::Pin<Layer8<'a,T>>;
type Layer10<'a,T>=pin::Pin<Layer9<'a,T>>;
type Layer11<'a,T>=pin::Pin<Layer10<'a,T>>;
type Layer12<'a,T>=pin::Pin<Layer11<'a,T>>;
type Layer13<'a,T>=pin::Pin<Layer12<'a,T>>;
type Layer14<'a,T>=pin::Pin<Layer13<'a,T>>;
type Layer15<'a,T>=pin::Pin<Layer14<'a,T>>;
type Layer16<'a,T>=pin::Pin<Layer15<'a,T>>;
type Layer17<'a,T>=pin::Pin<Layer16<'a,T>>;
type Layer18<'a,T>=pin::Pin<Layer17<'a,T>>;
type Layer19<'a,T>=pin::Pin<Layer18<'a,T>>;
type Layer20<'a,T>=pin::Pin<Layer19<'a,T>>;
type Layer21<'a,T>=pin::Pin<Layer20<'a,T>>;
type Layer22<'a,T>=pin::Pin<Layer21<'a,T>>;
type Layer23<'a,T>=pin::Pin<Layer22<'a,T>>;
type Layer24<'a,T>=pin::Pin<Layer23<'a,T>>;
type Layer25<'a,T>=pin::Pin<Layer24<'a,T>>;
type Layer26<'a,T>=pin::Pin<Layer25<'a,T>>;
type Layer27<'a,T>=pin::Pin<Layer26<'a,T>>;
type Layer28<'a,T>=pin::Pin<Layer27<'a,T>>;
type Layer29<'a,T>=pin::Pin<Layer28<'a,T>>;
type Layer30<'a,T>=pin::Pin<Layer29<'a,T>>;
type Layer31<'a,T>=pin::Pin<Layer30<'a,T>>;
type Layer32<'a,T>=pin::Pin<Layer31<'a,T>>;
type Layer33<'a,T>=pin::Pin<Layer32<'a,T>>;
type Layer34<'a,T>=pin::Pin<Layer33<'a,T>>;
type Layer35<'a,T>=pin::Pin<Layer34<'a,T>>;
type Layer36<'a,T>=pin::Pin<Layer35<'a,T>>;
type Layer37<'a,T>=pin::Pin<Layer36<'a,T>>;
type Layer38<'a,T>=pin::Pin<Layer37<'a,T>>;
type Layer39<'a,T>=pin::Pin<Layer38<'a,T>>;
type Layer40<'a,T>=pin::Pin<Layer39<'a,T>>;
type Layer41<'a,T>=pin::Pin<Layer40<'a,T>>;
type Layer42<'a,T>=pin::Pin<Layer41<'a,T>>;
type Layer43<'a,T>=pin::Pin<Layer42<'a,T>>;
type Layer44<'a,T>=pin::Pin<Layer43<'a,T>>;
type Layer45<'a,T>=pin::Pin<Layer44<'a,T>>;
type Layer46<'a,T>=pin::Pin<Layer45<'a,T>>;
type Layer47<'a,T>=pin::Pin<Layer46<'a,T>>;
type Layer48<'a,T>=pin::Pin<Layer47<'a,T>>;
type Layer49<'a,T>=pin::Pin<Layer48<'a,T>>;
type Layer50<'a,T>=pin::Pin<Layer49<'a,T>>;
type Layer51<'a,T>=pin::Pin<Layer50<'a,T>>;
type Layer52<'a,T>=pin::Pin<Layer51<'a,T>>;
type Layer53<'a,T>=pin::Pin<Layer52<'a,T>>;
type Layer54<'a,T>=pin::Pin<Layer53<'a,T>>;
type Layer55<'a,T>=pin::Pin<Layer54<'a,T>>;
type Layer56<'a,T>=pin::Pin<Layer55<'a,T>>;
type Layer57<'a,T>=pin::Pin<Layer56<'a,T>>;
type Layer58<'a,T>=pin::Pin<Layer57<'a,T>>;
type Layer59<'a,T>=pin::Pin<Layer58<'a,T>>;
type Layer60<'a,T>=pin::Pin<Layer59<'a,T>>;
type Layer61<'a,T>=pin::Pin<Layer60<'a,T>>;
type Layer62<'a,T>=pin::Pin<Layer61<'a,T>>;
type Layer63<'a,T>=pin::Pin<Layer62<'a,T>>;
type Layer64<'a,T>=pin::Pin<Layer63<'a,T>>;
type Layer65<'a,T>=pin::Pin<Layer64<'a,T>>;
type Layer66<'a,T>=pin::Pin<Layer65<'a,T>>;
type Layer67<'a,T>=pin::Pin<Layer66<'a,T>>;
type Layer68<'a,T>=pin::Pin<Layer67<'a,T>>;
type Layer69<'a,T>=pin::Pin<Layer68<'a,T>>;
type Layer70<'a,T>=pin::Pin<Layer69<'a,T>>;
type Layer71<'a,T>=pin::Pin<Layer70<'a,T>>;
type Layer72<'a,T>=pin::Pin<Layer71<'a,T>>;
type Layer73<'a,T>=pin::Pin<Layer72<'a,T>>;
type Layer74<'a,T>=pin::Pin<Layer73<'a,T>>;
type Layer75<'a,T>=pin::Pin<Layer74<'a,T>>;
type Layer76<'a,T>=pin::Pin<Layer75<'a,T>>;
type Layer77<'a,T>=pin::Pin<Layer76<'a,T>>;
type Layer78<'a,T>=pin::Pin<Layer77<'a,T>>;
type Layer79<'a,T>=pin::Pin<Layer78<'a,T>>;
type Layer80<'a,T>=pin::Pin<Layer79<'a,T>>;
type Layer81<'a,T>=pin::Pin<Layer80<'a,T>>;
type Layer82<'a,T>=pin::Pin<Layer81<'a,T>>;
type Layer83<'a,T>=pin::Pin<Layer82<'a,T>>;
type Layer84<'a,T>=pin::Pin<Layer83<'a,T>>;
type Layer85<'a,T>=pin::Pin<Layer84<'a,T>>;
type Layer86<'a,T>=pin::Pin<Layer85<'a,T>>;
type Layer87<'a,T>=pin::Pin<Layer86<'a,T>>;
type Layer88<'a,T>=pin::Pin<Layer87<'a,T>>;
type Layer89<'a,T>=pin::Pin<Layer88<'a,T>>;
type Layer90<'a,T>=pin::Pin<Layer89<'a,T>>;
type Layer91<'a,T>=pin::Pin<Layer90<'a,T>>;
type Layer92<'a,T>=pin::Pin<Layer91<'a,T>>;
type Layer93<'a,T>=pin::Pin<Layer92<'a,T>>;
type Layer94<'a,T>=pin::Pin<Layer93<'a,T>>;
type Layer95<'a,T>=pin::Pin<Layer94<'a,T>>;
type Layer96<'a,T>=pin::Pin<Layer95<'a,T>>;
type Layer97<'a,T>=pin::Pin<Layer96<'a,T>>;
type Layer98<'a,T>=pin::Pin<Layer97<'a,T>>;
type Layer99<'a,T>=pin::Pin<Layer98<'a,T>>;
type Layer100<'a,T>=pin::Pin<Layer99<'a,T>>;
type Layer101<'a,T>=pin::Pin<Layer100<'a,T>>;
type Layer102<'a,T>=pin::Pin<Layer101<'a,T>>;
type Layer103<'a,T>=pin::Pin<Layer102<'a,T>>;
type Layer104<'a,T>=pin::Pin<Layer103<'a,T>>;
type Layer105<'a,T>=pin::Pin<Layer104<'a,T>>;
type Layer106<'a,T>=pin::Pin<Layer105<'a,T>>;
type Layer107<'a,T>=pin::Pin<Layer106<'a,T>>;
type Layer108<'a,T>=pin::Pin<Layer107<'a,T>>;
type Layer109<'a,T>=pin::Pin<Layer108<'a,T>>;
type Layer110<'a,T>=pin::Pin<Layer109<'a,T>>;
type Layer111<'a,T>=pin::Pin<Layer110<'a,T>>;
type Layer112<'a,T>=pin::Pin<Layer111<'a,T>>;
type Layer113<'a,T>=pin::Pin<Layer112<'a,T>>;
type Layer114<'a,T>=pin::Pin<Layer113<'a,T>>;
type Layer115<'a,T>=pin::Pin<Layer114<'a,T>>;
type Layer116<'a,T>=pin::Pin<Layer115<'a,T>>;
type Layer117<'a,T>=pin::Pin<Layer116<'a,T>>;
type Layer118<'a,T>=pin::Pin<Layer117<'a,T>>;
type Layer119<'a,T>=pin::Pin<Layer118<'a,T>>;
type Layer120<'a,T>=pin::Pin<Layer119<'a,T>>;
type Layer121<'a,T>=pin::Pin<Layer120<'a,T>>;
type Layer122<'a,T>=pin::Pin<Layer121<'a,T>>;
type Layer123<'a,T>=pin::Pin<Layer122<'a,T>>;
type Layer124<'a,T>=pin::Pin<Layer123<'a,T>>;
type Layer125<'a,T>=pin::Pin<Layer124<'a,T>>;
type Layer126<'a,T>=pin::Pin<Layer125<'a,T>>;
type Layer127<'a,T>=pin::Pin<Layer126<'a,T>>;
type Layer128<'a,T>=pin::Pin<Layer127<'a,T>>;
impl<P:ops::DerefMut>pin::Pin<P>{pub fn deep(self:Layer128<'_,Self>){}}
