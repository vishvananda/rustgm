#![feature(no_core,lang_items,intrinsics,rustc_attrs,const_trait_impl)]
#![no_core]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="clone"]pub const trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized>LegacyReceiver for &T{}
impl<T:PointeeSized>LegacyReceiver for &mut T{}
#[lang="drop"]pub const trait Drop{fn drop(&mut self);}
pub mod intrinsics{use crate::Sized;#[rustc_intrinsic]pub const fn needs_drop<T:?Sized>()->bool;}
pub const fn needs_drop<T: ?Sized>() -> bool {
    const { intrinsics::needs_drop::<T>() }
}

struct Leaf;impl Drop for Leaf{fn drop(&mut self){}}
struct Root(Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf);
const Q0:bool=needs_drop::<Root>();type Check0=[u8;Q0 as usize];
const Q1:bool=needs_drop::<Root>();type Check1=[u8;Q1 as usize];
const Q2:bool=needs_drop::<Root>();type Check2=[u8;Q2 as usize];
const Q3:bool=needs_drop::<Root>();type Check3=[u8;Q3 as usize];
const Q4:bool=needs_drop::<Root>();type Check4=[u8;Q4 as usize];
const Q5:bool=needs_drop::<Root>();type Check5=[u8;Q5 as usize];
const Q6:bool=needs_drop::<Root>();type Check6=[u8;Q6 as usize];
const Q7:bool=needs_drop::<Root>();type Check7=[u8;Q7 as usize];
const Q8:bool=needs_drop::<Root>();type Check8=[u8;Q8 as usize];
const Q9:bool=needs_drop::<Root>();type Check9=[u8;Q9 as usize];
const Q10:bool=needs_drop::<Root>();type Check10=[u8;Q10 as usize];
const Q11:bool=needs_drop::<Root>();type Check11=[u8;Q11 as usize];
const Q12:bool=needs_drop::<Root>();type Check12=[u8;Q12 as usize];
const Q13:bool=needs_drop::<Root>();type Check13=[u8;Q13 as usize];
const Q14:bool=needs_drop::<Root>();type Check14=[u8;Q14 as usize];
const Q15:bool=needs_drop::<Root>();type Check15=[u8;Q15 as usize];
const Q16:bool=needs_drop::<Root>();type Check16=[u8;Q16 as usize];
const Q17:bool=needs_drop::<Root>();type Check17=[u8;Q17 as usize];
const Q18:bool=needs_drop::<Root>();type Check18=[u8;Q18 as usize];
const Q19:bool=needs_drop::<Root>();type Check19=[u8;Q19 as usize];
const Q20:bool=needs_drop::<Root>();type Check20=[u8;Q20 as usize];
const Q21:bool=needs_drop::<Root>();type Check21=[u8;Q21 as usize];
const Q22:bool=needs_drop::<Root>();type Check22=[u8;Q22 as usize];
const Q23:bool=needs_drop::<Root>();type Check23=[u8;Q23 as usize];
const Q24:bool=needs_drop::<Root>();type Check24=[u8;Q24 as usize];
const Q25:bool=needs_drop::<Root>();type Check25=[u8;Q25 as usize];
const Q26:bool=needs_drop::<Root>();type Check26=[u8;Q26 as usize];
const Q27:bool=needs_drop::<Root>();type Check27=[u8;Q27 as usize];
const Q28:bool=needs_drop::<Root>();type Check28=[u8;Q28 as usize];
const Q29:bool=needs_drop::<Root>();type Check29=[u8;Q29 as usize];
const Q30:bool=needs_drop::<Root>();type Check30=[u8;Q30 as usize];
const Q31:bool=needs_drop::<Root>();type Check31=[u8;Q31 as usize];
#[unsafe(no_mangle)]pub extern "C" fn observe()->u8{Q31 as u8}
