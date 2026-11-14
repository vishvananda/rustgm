#![feature(no_core,lang_items,rustc_attrs,intrinsics,auto_traits)]
#![no_core]
#![allow(internal_features,dead_code)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="freeze"]pub unsafe auto trait Freeze{}
#[lang="clone"]pub trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized>LegacyReceiver for &T{}
impl Clone for u8{fn clone(&self)->Self{*self}}impl Copy for u8{}
impl Clone for u16{fn clone(&self)->Self{*self}}impl Copy for u16{}
impl Clone for u32{fn clone(&self)->Self{*self}}impl Copy for u32{}
impl Clone for u64{fn clone(&self)->Self{*self}}impl Copy for u64{}
impl Clone for u128{fn clone(&self)->Self{*self}}impl Copy for u128{}
impl Clone for usize{fn clone(&self)->Self{*self}}impl Copy for usize{}
impl Clone for i8{fn clone(&self)->Self{*self}}impl Copy for i8{}
impl Clone for i16{fn clone(&self)->Self{*self}}impl Copy for i16{}
impl Clone for i32{fn clone(&self)->Self{*self}}impl Copy for i32{}
impl Clone for i64{fn clone(&self)->Self{*self}}impl Copy for i64{}
impl Clone for i128{fn clone(&self)->Self{*self}}impl Copy for i128{}
impl Clone for isize{fn clone(&self)->Self{*self}}impl Copy for isize{}
#[rustc_intrinsic_const_stable_indirect]#[rustc_nounwind]#[rustc_intrinsic]pub const fn mul_with_overflow<T:Copy>(x:T,y:T)->(T,bool);
const M0:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M1:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M2:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M3:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M4:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M5:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M6:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M7:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M8:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M9:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M10:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M11:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M12:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M13:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M14:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M15:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M16:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M17:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M18:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M19:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M20:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M21:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M22:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M23:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M24:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M25:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M26:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M27:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M28:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M29:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M30:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M31:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M32:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M33:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M34:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M35:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M36:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M37:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M38:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M39:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M40:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M41:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M42:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M43:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M44:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M45:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M46:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M47:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M48:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M49:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M50:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M51:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M52:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M53:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M54:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M55:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M56:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M57:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M58:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M59:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M60:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M61:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M62:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M63:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M64:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M65:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M66:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M67:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M68:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M69:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M70:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M71:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M72:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M73:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M74:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M75:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M76:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M77:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M78:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M79:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M80:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M81:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M82:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M83:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M84:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M85:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M86:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M87:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M88:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M89:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M90:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M91:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M92:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M93:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M94:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M95:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M96:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M97:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M98:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M99:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M100:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M101:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M102:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M103:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M104:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M105:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M106:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M107:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M108:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M109:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M110:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M111:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M112:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M113:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M114:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M115:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M116:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M117:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M118:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M119:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M120:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M121:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M122:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M123:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M124:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M125:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M126:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
const M127:(u64,bool)=mul_with_overflow::<u64>(3u64,11u64);
