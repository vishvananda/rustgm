// Authored reducer following pinned core/contracts.rs and callable language traits.
// Authored protocol reducer; interfaces follow pinned core/ops/try_trait.rs and control_flow.rs.
#![feature(unboxed_closures,auto_traits,const_closures,stmt_expr_attributes)]
#![feature(no_core,lang_items,rustc_attrs,const_trait_impl)]
#![no_core]
#![feature(const_destruct)]
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

#[lang="freeze"]pub unsafe auto trait Freeze{}
#[lang="destruct"]#[rustc_deny_explicit_impl]pub const trait Destruct:PointeeSized{}
const fn consume<F:[const]FnOnce(u8)->u8>(f:F)->u8{f(23)}
const fn local<T>(x:u8)->u8{
let call0=const move|v|x;consume(call0);
let call1=const move|v|x;consume(call1);
let call2=const move|v|x;consume(call2);
let call3=const move|v|x;consume(call3);
let call4=const move|v|x;consume(call4);
let call5=const move|v|x;consume(call5);
let call6=const move|v|x;consume(call6);
let call7=const move|v|x;consume(call7);
let call8=const move|v|x;consume(call8);
let call9=const move|v|x;consume(call9);
let call10=const move|v|x;consume(call10);
let call11=const move|v|x;consume(call11);
let call12=const move|v|x;consume(call12);
let call13=const move|v|x;consume(call13);
let call14=const move|v|x;consume(call14);
let call15=const move|v|x;consume(call15);
let call16=const move|v|x;consume(call16);
let call17=const move|v|x;consume(call17);
let call18=const move|v|x;consume(call18);
let call19=const move|v|x;consume(call19);
let call20=const move|v|x;consume(call20);
let call21=const move|v|x;consume(call21);
let call22=const move|v|x;consume(call22);
let call23=const move|v|x;consume(call23);
let call24=const move|v|x;consume(call24);
let call25=const move|v|x;consume(call25);
let call26=const move|v|x;consume(call26);
let call27=const move|v|x;consume(call27);
let call28=const move|v|x;consume(call28);
let call29=const move|v|x;consume(call29);
let call30=const move|v|x;consume(call30);
let call31=const move|v|x;consume(call31);
23}
struct Unrelated0<A>(A);
struct Unrelated1<A>(A);
struct Unrelated2<A>(A);
struct Unrelated3<A>(A);
struct Unrelated4<A>(A);
struct Unrelated5<A>(A);
struct Unrelated6<A>(A);
struct Unrelated7<A>(A);
struct Unrelated8<A>(A);
struct Unrelated9<A>(A);
struct Unrelated10<A>(A);
struct Unrelated11<A>(A);
struct Unrelated12<A>(A);
struct Unrelated13<A>(A);
struct Unrelated14<A>(A);
struct Unrelated15<A>(A);
struct Unrelated16<A>(A);
struct Unrelated17<A>(A);
struct Unrelated18<A>(A);
struct Unrelated19<A>(A);
struct Unrelated20<A>(A);
struct Unrelated21<A>(A);
struct Unrelated22<A>(A);
struct Unrelated23<A>(A);
struct Unrelated24<A>(A);
struct Unrelated25<A>(A);
struct Unrelated26<A>(A);
struct Unrelated27<A>(A);
struct Unrelated28<A>(A);
struct Unrelated29<A>(A);
struct Unrelated30<A>(A);
struct Unrelated31<A>(A);
struct Unrelated32<A>(A);
struct Unrelated33<A>(A);
struct Unrelated34<A>(A);
struct Unrelated35<A>(A);
struct Unrelated36<A>(A);
struct Unrelated37<A>(A);
struct Unrelated38<A>(A);
struct Unrelated39<A>(A);
struct Unrelated40<A>(A);
struct Unrelated41<A>(A);
struct Unrelated42<A>(A);
struct Unrelated43<A>(A);
struct Unrelated44<A>(A);
struct Unrelated45<A>(A);
struct Unrelated46<A>(A);
struct Unrelated47<A>(A);
struct Unrelated48<A>(A);
struct Unrelated49<A>(A);
struct Unrelated50<A>(A);
struct Unrelated51<A>(A);
struct Unrelated52<A>(A);
struct Unrelated53<A>(A);
struct Unrelated54<A>(A);
struct Unrelated55<A>(A);
struct Unrelated56<A>(A);
struct Unrelated57<A>(A);
struct Unrelated58<A>(A);
struct Unrelated59<A>(A);
struct Unrelated60<A>(A);
struct Unrelated61<A>(A);
struct Unrelated62<A>(A);
struct Unrelated63<A>(A);
struct Unrelated64<A>(A);
struct Unrelated65<A>(A);
struct Unrelated66<A>(A);
struct Unrelated67<A>(A);
struct Unrelated68<A>(A);
struct Unrelated69<A>(A);
struct Unrelated70<A>(A);
struct Unrelated71<A>(A);
struct Unrelated72<A>(A);
struct Unrelated73<A>(A);
struct Unrelated74<A>(A);
struct Unrelated75<A>(A);
struct Unrelated76<A>(A);
struct Unrelated77<A>(A);
struct Unrelated78<A>(A);
struct Unrelated79<A>(A);
struct Unrelated80<A>(A);
struct Unrelated81<A>(A);
struct Unrelated82<A>(A);
struct Unrelated83<A>(A);
struct Unrelated84<A>(A);
struct Unrelated85<A>(A);
struct Unrelated86<A>(A);
struct Unrelated87<A>(A);
struct Unrelated88<A>(A);
struct Unrelated89<A>(A);
struct Unrelated90<A>(A);
struct Unrelated91<A>(A);
struct Unrelated92<A>(A);
struct Unrelated93<A>(A);
struct Unrelated94<A>(A);
struct Unrelated95<A>(A);
struct Unrelated96<A>(A);
struct Unrelated97<A>(A);
struct Unrelated98<A>(A);
struct Unrelated99<A>(A);
struct Unrelated100<A>(A);
struct Unrelated101<A>(A);
struct Unrelated102<A>(A);
struct Unrelated103<A>(A);
struct Unrelated104<A>(A);
struct Unrelated105<A>(A);
struct Unrelated106<A>(A);
struct Unrelated107<A>(A);
struct Unrelated108<A>(A);
struct Unrelated109<A>(A);
struct Unrelated110<A>(A);
struct Unrelated111<A>(A);
struct Unrelated112<A>(A);
struct Unrelated113<A>(A);
struct Unrelated114<A>(A);
struct Unrelated115<A>(A);
struct Unrelated116<A>(A);
struct Unrelated117<A>(A);
struct Unrelated118<A>(A);
struct Unrelated119<A>(A);
struct Unrelated120<A>(A);
struct Unrelated121<A>(A);
struct Unrelated122<A>(A);
struct Unrelated123<A>(A);
struct Unrelated124<A>(A);
struct Unrelated125<A>(A);
struct Unrelated126<A>(A);
struct Unrelated127<A>(A);
