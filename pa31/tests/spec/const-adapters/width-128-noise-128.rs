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
struct Noise0(u8);
struct Noise1(u8);
struct Noise2(u8);
struct Noise3(u8);
struct Noise4(u8);
struct Noise5(u8);
struct Noise6(u8);
struct Noise7(u8);
struct Noise8(u8);
struct Noise9(u8);
struct Noise10(u8);
struct Noise11(u8);
struct Noise12(u8);
struct Noise13(u8);
struct Noise14(u8);
struct Noise15(u8);
struct Noise16(u8);
struct Noise17(u8);
struct Noise18(u8);
struct Noise19(u8);
struct Noise20(u8);
struct Noise21(u8);
struct Noise22(u8);
struct Noise23(u8);
struct Noise24(u8);
struct Noise25(u8);
struct Noise26(u8);
struct Noise27(u8);
struct Noise28(u8);
struct Noise29(u8);
struct Noise30(u8);
struct Noise31(u8);
struct Noise32(u8);
struct Noise33(u8);
struct Noise34(u8);
struct Noise35(u8);
struct Noise36(u8);
struct Noise37(u8);
struct Noise38(u8);
struct Noise39(u8);
struct Noise40(u8);
struct Noise41(u8);
struct Noise42(u8);
struct Noise43(u8);
struct Noise44(u8);
struct Noise45(u8);
struct Noise46(u8);
struct Noise47(u8);
struct Noise48(u8);
struct Noise49(u8);
struct Noise50(u8);
struct Noise51(u8);
struct Noise52(u8);
struct Noise53(u8);
struct Noise54(u8);
struct Noise55(u8);
struct Noise56(u8);
struct Noise57(u8);
struct Noise58(u8);
struct Noise59(u8);
struct Noise60(u8);
struct Noise61(u8);
struct Noise62(u8);
struct Noise63(u8);
struct Noise64(u8);
struct Noise65(u8);
struct Noise66(u8);
struct Noise67(u8);
struct Noise68(u8);
struct Noise69(u8);
struct Noise70(u8);
struct Noise71(u8);
struct Noise72(u8);
struct Noise73(u8);
struct Noise74(u8);
struct Noise75(u8);
struct Noise76(u8);
struct Noise77(u8);
struct Noise78(u8);
struct Noise79(u8);
struct Noise80(u8);
struct Noise81(u8);
struct Noise82(u8);
struct Noise83(u8);
struct Noise84(u8);
struct Noise85(u8);
struct Noise86(u8);
struct Noise87(u8);
struct Noise88(u8);
struct Noise89(u8);
struct Noise90(u8);
struct Noise91(u8);
struct Noise92(u8);
struct Noise93(u8);
struct Noise94(u8);
struct Noise95(u8);
struct Noise96(u8);
struct Noise97(u8);
struct Noise98(u8);
struct Noise99(u8);
struct Noise100(u8);
struct Noise101(u8);
struct Noise102(u8);
struct Noise103(u8);
struct Noise104(u8);
struct Noise105(u8);
struct Noise106(u8);
struct Noise107(u8);
struct Noise108(u8);
struct Noise109(u8);
struct Noise110(u8);
struct Noise111(u8);
struct Noise112(u8);
struct Noise113(u8);
struct Noise114(u8);
struct Noise115(u8);
struct Noise116(u8);
struct Noise117(u8);
struct Noise118(u8);
struct Noise119(u8);
struct Noise120(u8);
struct Noise121(u8);
struct Noise122(u8);
struct Noise123(u8);
struct Noise124(u8);
struct Noise125(u8);
struct Noise126(u8);
struct Noise127(u8);
const fn one0<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one1<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one2<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one3<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one4<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one5<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one6<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one7<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one8<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one9<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one10<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one11<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one12<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one13<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one14<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one15<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one16<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one17<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one18<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one19<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one20<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one21<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one22<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one23<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one24<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one25<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one26<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one27<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one28<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one29<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one30<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one31<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one32<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one33<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one34<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one35<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one36<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one37<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one38<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one39<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one40<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one41<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one42<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one43<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one44<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one45<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one46<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one47<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one48<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one49<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one50<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one51<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one52<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one53<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one54<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one55<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one56<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one57<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one58<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one59<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one60<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one61<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one62<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one63<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one64<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one65<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one66<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one67<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one68<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one69<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one70<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one71<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one72<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one73<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one74<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one75<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one76<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one77<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one78<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one79<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one80<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one81<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one82<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one83<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one84<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one85<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one86<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one87<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one88<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one89<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one90<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one91<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one92<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one93<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one94<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one95<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one96<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one97<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one98<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one99<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one100<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one101<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one102<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one103<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one104<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one105<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one106<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one107<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one108<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one109<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one110<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one111<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one112<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one113<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one114<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one115<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one116<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one117<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one118<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one119<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one120<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one121<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one122<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one123<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one124<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one125<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one126<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
const fn one127<T,A,F>(mut f:F)->impl [const]FnMut(A)->T+[const]Destruct where F:[const]FnMut(A)->T+[const]Destruct{const move|a|f(a)}
