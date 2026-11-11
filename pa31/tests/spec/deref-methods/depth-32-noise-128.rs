#![recursion_limit="512"]
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
#[lang="deref"]pub trait Deref:PointeeSized{#[lang="deref_target"]type Target:PointeeSized;fn deref(&self)->&Self::Target;}
#[lang="deref_mut"]pub trait DerefMut:Deref+PointeeSized{fn deref_mut(&mut self)->&mut Self::Target;}
struct S{n:u8}impl S{fn get(&self)->u8{self.n}fn set(&mut self,n:u8){self.n=n;}fn take(self)->u8{self.n}}
struct Hold<T>(T);impl<T>Deref for Hold<T>{type Target=T;fn deref(&self)->&T{&self.0}}
impl<T>DerefMut for Hold<T>{fn deref_mut(&mut self)->&mut T{&mut self.0}}
type Layer0=S;
type Layer1=Hold<Layer0>;
type Layer2=Hold<Layer1>;
type Layer3=Hold<Layer2>;
type Layer4=Hold<Layer3>;
type Layer5=Hold<Layer4>;
type Layer6=Hold<Layer5>;
type Layer7=Hold<Layer6>;
type Layer8=Hold<Layer7>;
type Layer9=Hold<Layer8>;
type Layer10=Hold<Layer9>;
type Layer11=Hold<Layer10>;
type Layer12=Hold<Layer11>;
type Layer13=Hold<Layer12>;
type Layer14=Hold<Layer13>;
type Layer15=Hold<Layer14>;
type Layer16=Hold<Layer15>;
type Layer17=Hold<Layer16>;
type Layer18=Hold<Layer17>;
type Layer19=Hold<Layer18>;
type Layer20=Hold<Layer19>;
type Layer21=Hold<Layer20>;
type Layer22=Hold<Layer21>;
type Layer23=Hold<Layer22>;
type Layer24=Hold<Layer23>;
type Layer25=Hold<Layer24>;
type Layer26=Hold<Layer25>;
type Layer27=Hold<Layer26>;
type Layer28=Hold<Layer27>;
type Layer29=Hold<Layer28>;
type Layer30=Hold<Layer29>;
type Layer31=Hold<Layer30>;
type Layer32=Hold<Layer31>;
type Layer33=Hold<Layer32>;
type Layer34=Hold<Layer33>;
type Layer35=Hold<Layer34>;
type Layer36=Hold<Layer35>;
type Layer37=Hold<Layer36>;
type Layer38=Hold<Layer37>;
type Layer39=Hold<Layer38>;
type Layer40=Hold<Layer39>;
type Layer41=Hold<Layer40>;
type Layer42=Hold<Layer41>;
type Layer43=Hold<Layer42>;
type Layer44=Hold<Layer43>;
type Layer45=Hold<Layer44>;
type Layer46=Hold<Layer45>;
type Layer47=Hold<Layer46>;
type Layer48=Hold<Layer47>;
type Layer49=Hold<Layer48>;
type Layer50=Hold<Layer49>;
type Layer51=Hold<Layer50>;
type Layer52=Hold<Layer51>;
type Layer53=Hold<Layer52>;
type Layer54=Hold<Layer53>;
type Layer55=Hold<Layer54>;
type Layer56=Hold<Layer55>;
type Layer57=Hold<Layer56>;
type Layer58=Hold<Layer57>;
type Layer59=Hold<Layer58>;
type Layer60=Hold<Layer59>;
type Layer61=Hold<Layer60>;
type Layer62=Hold<Layer61>;
type Layer63=Hold<Layer62>;
type Layer64=Hold<Layer63>;
type Layer65=Hold<Layer64>;
type Layer66=Hold<Layer65>;
type Layer67=Hold<Layer66>;
type Layer68=Hold<Layer67>;
type Layer69=Hold<Layer68>;
type Layer70=Hold<Layer69>;
type Layer71=Hold<Layer70>;
type Layer72=Hold<Layer71>;
type Layer73=Hold<Layer72>;
type Layer74=Hold<Layer73>;
type Layer75=Hold<Layer74>;
type Layer76=Hold<Layer75>;
type Layer77=Hold<Layer76>;
type Layer78=Hold<Layer77>;
type Layer79=Hold<Layer78>;
type Layer80=Hold<Layer79>;
type Layer81=Hold<Layer80>;
type Layer82=Hold<Layer81>;
type Layer83=Hold<Layer82>;
type Layer84=Hold<Layer83>;
type Layer85=Hold<Layer84>;
type Layer86=Hold<Layer85>;
type Layer87=Hold<Layer86>;
type Layer88=Hold<Layer87>;
type Layer89=Hold<Layer88>;
type Layer90=Hold<Layer89>;
type Layer91=Hold<Layer90>;
type Layer92=Hold<Layer91>;
type Layer93=Hold<Layer92>;
type Layer94=Hold<Layer93>;
type Layer95=Hold<Layer94>;
type Layer96=Hold<Layer95>;
type Layer97=Hold<Layer96>;
type Layer98=Hold<Layer97>;
type Layer99=Hold<Layer98>;
type Layer100=Hold<Layer99>;
type Layer101=Hold<Layer100>;
type Layer102=Hold<Layer101>;
type Layer103=Hold<Layer102>;
type Layer104=Hold<Layer103>;
type Layer105=Hold<Layer104>;
type Layer106=Hold<Layer105>;
type Layer107=Hold<Layer106>;
type Layer108=Hold<Layer107>;
type Layer109=Hold<Layer108>;
type Layer110=Hold<Layer109>;
type Layer111=Hold<Layer110>;
type Layer112=Hold<Layer111>;
type Layer113=Hold<Layer112>;
type Layer114=Hold<Layer113>;
type Layer115=Hold<Layer114>;
type Layer116=Hold<Layer115>;
type Layer117=Hold<Layer116>;
type Layer118=Hold<Layer117>;
type Layer119=Hold<Layer118>;
type Layer120=Hold<Layer119>;
type Layer121=Hold<Layer120>;
type Layer122=Hold<Layer121>;
type Layer123=Hold<Layer122>;
type Layer124=Hold<Layer123>;
type Layer125=Hold<Layer124>;
type Layer126=Hold<Layer125>;
type Layer127=Hold<Layer126>;
type Layer128=Hold<Layer127>;
struct Unused0;trait Noise0{}
struct Unused1;trait Noise1{}
struct Unused2;trait Noise2{}
struct Unused3;trait Noise3{}
struct Unused4;trait Noise4{}
struct Unused5;trait Noise5{}
struct Unused6;trait Noise6{}
struct Unused7;trait Noise7{}
struct Unused8;trait Noise8{}
struct Unused9;trait Noise9{}
struct Unused10;trait Noise10{}
struct Unused11;trait Noise11{}
struct Unused12;trait Noise12{}
struct Unused13;trait Noise13{}
struct Unused14;trait Noise14{}
struct Unused15;trait Noise15{}
struct Unused16;trait Noise16{}
struct Unused17;trait Noise17{}
struct Unused18;trait Noise18{}
struct Unused19;trait Noise19{}
struct Unused20;trait Noise20{}
struct Unused21;trait Noise21{}
struct Unused22;trait Noise22{}
struct Unused23;trait Noise23{}
struct Unused24;trait Noise24{}
struct Unused25;trait Noise25{}
struct Unused26;trait Noise26{}
struct Unused27;trait Noise27{}
struct Unused28;trait Noise28{}
struct Unused29;trait Noise29{}
struct Unused30;trait Noise30{}
struct Unused31;trait Noise31{}
struct Unused32;trait Noise32{}
struct Unused33;trait Noise33{}
struct Unused34;trait Noise34{}
struct Unused35;trait Noise35{}
struct Unused36;trait Noise36{}
struct Unused37;trait Noise37{}
struct Unused38;trait Noise38{}
struct Unused39;trait Noise39{}
struct Unused40;trait Noise40{}
struct Unused41;trait Noise41{}
struct Unused42;trait Noise42{}
struct Unused43;trait Noise43{}
struct Unused44;trait Noise44{}
struct Unused45;trait Noise45{}
struct Unused46;trait Noise46{}
struct Unused47;trait Noise47{}
struct Unused48;trait Noise48{}
struct Unused49;trait Noise49{}
struct Unused50;trait Noise50{}
struct Unused51;trait Noise51{}
struct Unused52;trait Noise52{}
struct Unused53;trait Noise53{}
struct Unused54;trait Noise54{}
struct Unused55;trait Noise55{}
struct Unused56;trait Noise56{}
struct Unused57;trait Noise57{}
struct Unused58;trait Noise58{}
struct Unused59;trait Noise59{}
struct Unused60;trait Noise60{}
struct Unused61;trait Noise61{}
struct Unused62;trait Noise62{}
struct Unused63;trait Noise63{}
struct Unused64;trait Noise64{}
struct Unused65;trait Noise65{}
struct Unused66;trait Noise66{}
struct Unused67;trait Noise67{}
struct Unused68;trait Noise68{}
struct Unused69;trait Noise69{}
struct Unused70;trait Noise70{}
struct Unused71;trait Noise71{}
struct Unused72;trait Noise72{}
struct Unused73;trait Noise73{}
struct Unused74;trait Noise74{}
struct Unused75;trait Noise75{}
struct Unused76;trait Noise76{}
struct Unused77;trait Noise77{}
struct Unused78;trait Noise78{}
struct Unused79;trait Noise79{}
struct Unused80;trait Noise80{}
struct Unused81;trait Noise81{}
struct Unused82;trait Noise82{}
struct Unused83;trait Noise83{}
struct Unused84;trait Noise84{}
struct Unused85;trait Noise85{}
struct Unused86;trait Noise86{}
struct Unused87;trait Noise87{}
struct Unused88;trait Noise88{}
struct Unused89;trait Noise89{}
struct Unused90;trait Noise90{}
struct Unused91;trait Noise91{}
struct Unused92;trait Noise92{}
struct Unused93;trait Noise93{}
struct Unused94;trait Noise94{}
struct Unused95;trait Noise95{}
struct Unused96;trait Noise96{}
struct Unused97;trait Noise97{}
struct Unused98;trait Noise98{}
struct Unused99;trait Noise99{}
struct Unused100;trait Noise100{}
struct Unused101;trait Noise101{}
struct Unused102;trait Noise102{}
struct Unused103;trait Noise103{}
struct Unused104;trait Noise104{}
struct Unused105;trait Noise105{}
struct Unused106;trait Noise106{}
struct Unused107;trait Noise107{}
struct Unused108;trait Noise108{}
struct Unused109;trait Noise109{}
struct Unused110;trait Noise110{}
struct Unused111;trait Noise111{}
struct Unused112;trait Noise112{}
struct Unused113;trait Noise113{}
struct Unused114;trait Noise114{}
struct Unused115;trait Noise115{}
struct Unused116;trait Noise116{}
struct Unused117;trait Noise117{}
struct Unused118;trait Noise118{}
struct Unused119;trait Noise119{}
struct Unused120;trait Noise120{}
struct Unused121;trait Noise121{}
struct Unused122;trait Noise122{}
struct Unused123;trait Noise123{}
struct Unused124;trait Noise124{}
struct Unused125;trait Noise125{}
struct Unused126;trait Noise126{}
struct Unused127;trait Noise127{}
fn check(h:&Layer32)->u8{h.get()}
