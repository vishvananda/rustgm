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
let call32=const move|v|x;consume(call32);
let call33=const move|v|x;consume(call33);
let call34=const move|v|x;consume(call34);
let call35=const move|v|x;consume(call35);
let call36=const move|v|x;consume(call36);
let call37=const move|v|x;consume(call37);
let call38=const move|v|x;consume(call38);
let call39=const move|v|x;consume(call39);
let call40=const move|v|x;consume(call40);
let call41=const move|v|x;consume(call41);
let call42=const move|v|x;consume(call42);
let call43=const move|v|x;consume(call43);
let call44=const move|v|x;consume(call44);
let call45=const move|v|x;consume(call45);
let call46=const move|v|x;consume(call46);
let call47=const move|v|x;consume(call47);
let call48=const move|v|x;consume(call48);
let call49=const move|v|x;consume(call49);
let call50=const move|v|x;consume(call50);
let call51=const move|v|x;consume(call51);
let call52=const move|v|x;consume(call52);
let call53=const move|v|x;consume(call53);
let call54=const move|v|x;consume(call54);
let call55=const move|v|x;consume(call55);
let call56=const move|v|x;consume(call56);
let call57=const move|v|x;consume(call57);
let call58=const move|v|x;consume(call58);
let call59=const move|v|x;consume(call59);
let call60=const move|v|x;consume(call60);
let call61=const move|v|x;consume(call61);
let call62=const move|v|x;consume(call62);
let call63=const move|v|x;consume(call63);
let call64=const move|v|x;consume(call64);
let call65=const move|v|x;consume(call65);
let call66=const move|v|x;consume(call66);
let call67=const move|v|x;consume(call67);
let call68=const move|v|x;consume(call68);
let call69=const move|v|x;consume(call69);
let call70=const move|v|x;consume(call70);
let call71=const move|v|x;consume(call71);
let call72=const move|v|x;consume(call72);
let call73=const move|v|x;consume(call73);
let call74=const move|v|x;consume(call74);
let call75=const move|v|x;consume(call75);
let call76=const move|v|x;consume(call76);
let call77=const move|v|x;consume(call77);
let call78=const move|v|x;consume(call78);
let call79=const move|v|x;consume(call79);
let call80=const move|v|x;consume(call80);
let call81=const move|v|x;consume(call81);
let call82=const move|v|x;consume(call82);
let call83=const move|v|x;consume(call83);
let call84=const move|v|x;consume(call84);
let call85=const move|v|x;consume(call85);
let call86=const move|v|x;consume(call86);
let call87=const move|v|x;consume(call87);
let call88=const move|v|x;consume(call88);
let call89=const move|v|x;consume(call89);
let call90=const move|v|x;consume(call90);
let call91=const move|v|x;consume(call91);
let call92=const move|v|x;consume(call92);
let call93=const move|v|x;consume(call93);
let call94=const move|v|x;consume(call94);
let call95=const move|v|x;consume(call95);
let call96=const move|v|x;consume(call96);
let call97=const move|v|x;consume(call97);
let call98=const move|v|x;consume(call98);
let call99=const move|v|x;consume(call99);
let call100=const move|v|x;consume(call100);
let call101=const move|v|x;consume(call101);
let call102=const move|v|x;consume(call102);
let call103=const move|v|x;consume(call103);
let call104=const move|v|x;consume(call104);
let call105=const move|v|x;consume(call105);
let call106=const move|v|x;consume(call106);
let call107=const move|v|x;consume(call107);
let call108=const move|v|x;consume(call108);
let call109=const move|v|x;consume(call109);
let call110=const move|v|x;consume(call110);
let call111=const move|v|x;consume(call111);
let call112=const move|v|x;consume(call112);
let call113=const move|v|x;consume(call113);
let call114=const move|v|x;consume(call114);
let call115=const move|v|x;consume(call115);
let call116=const move|v|x;consume(call116);
let call117=const move|v|x;consume(call117);
let call118=const move|v|x;consume(call118);
let call119=const move|v|x;consume(call119);
let call120=const move|v|x;consume(call120);
let call121=const move|v|x;consume(call121);
let call122=const move|v|x;consume(call122);
let call123=const move|v|x;consume(call123);
let call124=const move|v|x;consume(call124);
let call125=const move|v|x;consume(call125);
let call126=const move|v|x;consume(call126);
let call127=const move|v|x;consume(call127);
23}
