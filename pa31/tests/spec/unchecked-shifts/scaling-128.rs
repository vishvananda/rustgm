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
#[rustc_intrinsic_const_stable_indirect]
#[rustc_nounwind]
#[rustc_intrinsic]
pub const unsafe fn unchecked_shl<T: Copy, U: Copy>(x: T, y: U) -> T;
#[rustc_intrinsic_const_stable_indirect]
#[rustc_nounwind]
#[rustc_intrinsic]
pub const unsafe fn unchecked_shr<T: Copy, U: Copy>(x: T, y: U) -> T;
const S0:u64=unsafe{unchecked_shl::<u64,u32>(1,0)};
const S1:u64=unsafe{unchecked_shl::<u64,u32>(1,1)};
const S2:u64=unsafe{unchecked_shl::<u64,u32>(1,2)};
const S3:u64=unsafe{unchecked_shl::<u64,u32>(1,3)};
const S4:u64=unsafe{unchecked_shl::<u64,u32>(1,4)};
const S5:u64=unsafe{unchecked_shl::<u64,u32>(1,5)};
const S6:u64=unsafe{unchecked_shl::<u64,u32>(1,6)};
const S7:u64=unsafe{unchecked_shl::<u64,u32>(1,7)};
const S8:u64=unsafe{unchecked_shl::<u64,u32>(1,8)};
const S9:u64=unsafe{unchecked_shl::<u64,u32>(1,9)};
const S10:u64=unsafe{unchecked_shl::<u64,u32>(1,10)};
const S11:u64=unsafe{unchecked_shl::<u64,u32>(1,11)};
const S12:u64=unsafe{unchecked_shl::<u64,u32>(1,12)};
const S13:u64=unsafe{unchecked_shl::<u64,u32>(1,13)};
const S14:u64=unsafe{unchecked_shl::<u64,u32>(1,14)};
const S15:u64=unsafe{unchecked_shl::<u64,u32>(1,15)};
const S16:u64=unsafe{unchecked_shl::<u64,u32>(1,16)};
const S17:u64=unsafe{unchecked_shl::<u64,u32>(1,17)};
const S18:u64=unsafe{unchecked_shl::<u64,u32>(1,18)};
const S19:u64=unsafe{unchecked_shl::<u64,u32>(1,19)};
const S20:u64=unsafe{unchecked_shl::<u64,u32>(1,20)};
const S21:u64=unsafe{unchecked_shl::<u64,u32>(1,21)};
const S22:u64=unsafe{unchecked_shl::<u64,u32>(1,22)};
const S23:u64=unsafe{unchecked_shl::<u64,u32>(1,23)};
const S24:u64=unsafe{unchecked_shl::<u64,u32>(1,24)};
const S25:u64=unsafe{unchecked_shl::<u64,u32>(1,25)};
const S26:u64=unsafe{unchecked_shl::<u64,u32>(1,26)};
const S27:u64=unsafe{unchecked_shl::<u64,u32>(1,27)};
const S28:u64=unsafe{unchecked_shl::<u64,u32>(1,28)};
const S29:u64=unsafe{unchecked_shl::<u64,u32>(1,29)};
const S30:u64=unsafe{unchecked_shl::<u64,u32>(1,30)};
const S31:u64=unsafe{unchecked_shl::<u64,u32>(1,31)};
const S32:u64=unsafe{unchecked_shl::<u64,u32>(1,32)};
const S33:u64=unsafe{unchecked_shl::<u64,u32>(1,33)};
const S34:u64=unsafe{unchecked_shl::<u64,u32>(1,34)};
const S35:u64=unsafe{unchecked_shl::<u64,u32>(1,35)};
const S36:u64=unsafe{unchecked_shl::<u64,u32>(1,36)};
const S37:u64=unsafe{unchecked_shl::<u64,u32>(1,37)};
const S38:u64=unsafe{unchecked_shl::<u64,u32>(1,38)};
const S39:u64=unsafe{unchecked_shl::<u64,u32>(1,39)};
const S40:u64=unsafe{unchecked_shl::<u64,u32>(1,40)};
const S41:u64=unsafe{unchecked_shl::<u64,u32>(1,41)};
const S42:u64=unsafe{unchecked_shl::<u64,u32>(1,42)};
const S43:u64=unsafe{unchecked_shl::<u64,u32>(1,43)};
const S44:u64=unsafe{unchecked_shl::<u64,u32>(1,44)};
const S45:u64=unsafe{unchecked_shl::<u64,u32>(1,45)};
const S46:u64=unsafe{unchecked_shl::<u64,u32>(1,46)};
const S47:u64=unsafe{unchecked_shl::<u64,u32>(1,47)};
const S48:u64=unsafe{unchecked_shl::<u64,u32>(1,48)};
const S49:u64=unsafe{unchecked_shl::<u64,u32>(1,49)};
const S50:u64=unsafe{unchecked_shl::<u64,u32>(1,50)};
const S51:u64=unsafe{unchecked_shl::<u64,u32>(1,51)};
const S52:u64=unsafe{unchecked_shl::<u64,u32>(1,52)};
const S53:u64=unsafe{unchecked_shl::<u64,u32>(1,53)};
const S54:u64=unsafe{unchecked_shl::<u64,u32>(1,54)};
const S55:u64=unsafe{unchecked_shl::<u64,u32>(1,55)};
const S56:u64=unsafe{unchecked_shl::<u64,u32>(1,56)};
const S57:u64=unsafe{unchecked_shl::<u64,u32>(1,57)};
const S58:u64=unsafe{unchecked_shl::<u64,u32>(1,58)};
const S59:u64=unsafe{unchecked_shl::<u64,u32>(1,59)};
const S60:u64=unsafe{unchecked_shl::<u64,u32>(1,60)};
const S61:u64=unsafe{unchecked_shl::<u64,u32>(1,61)};
const S62:u64=unsafe{unchecked_shl::<u64,u32>(1,62)};
const S63:u64=unsafe{unchecked_shl::<u64,u32>(1,63)};
const S64:u64=unsafe{unchecked_shl::<u64,u32>(1,0)};
const S65:u64=unsafe{unchecked_shl::<u64,u32>(1,1)};
const S66:u64=unsafe{unchecked_shl::<u64,u32>(1,2)};
const S67:u64=unsafe{unchecked_shl::<u64,u32>(1,3)};
const S68:u64=unsafe{unchecked_shl::<u64,u32>(1,4)};
const S69:u64=unsafe{unchecked_shl::<u64,u32>(1,5)};
const S70:u64=unsafe{unchecked_shl::<u64,u32>(1,6)};
const S71:u64=unsafe{unchecked_shl::<u64,u32>(1,7)};
const S72:u64=unsafe{unchecked_shl::<u64,u32>(1,8)};
const S73:u64=unsafe{unchecked_shl::<u64,u32>(1,9)};
const S74:u64=unsafe{unchecked_shl::<u64,u32>(1,10)};
const S75:u64=unsafe{unchecked_shl::<u64,u32>(1,11)};
const S76:u64=unsafe{unchecked_shl::<u64,u32>(1,12)};
const S77:u64=unsafe{unchecked_shl::<u64,u32>(1,13)};
const S78:u64=unsafe{unchecked_shl::<u64,u32>(1,14)};
const S79:u64=unsafe{unchecked_shl::<u64,u32>(1,15)};
const S80:u64=unsafe{unchecked_shl::<u64,u32>(1,16)};
const S81:u64=unsafe{unchecked_shl::<u64,u32>(1,17)};
const S82:u64=unsafe{unchecked_shl::<u64,u32>(1,18)};
const S83:u64=unsafe{unchecked_shl::<u64,u32>(1,19)};
const S84:u64=unsafe{unchecked_shl::<u64,u32>(1,20)};
const S85:u64=unsafe{unchecked_shl::<u64,u32>(1,21)};
const S86:u64=unsafe{unchecked_shl::<u64,u32>(1,22)};
const S87:u64=unsafe{unchecked_shl::<u64,u32>(1,23)};
const S88:u64=unsafe{unchecked_shl::<u64,u32>(1,24)};
const S89:u64=unsafe{unchecked_shl::<u64,u32>(1,25)};
const S90:u64=unsafe{unchecked_shl::<u64,u32>(1,26)};
const S91:u64=unsafe{unchecked_shl::<u64,u32>(1,27)};
const S92:u64=unsafe{unchecked_shl::<u64,u32>(1,28)};
const S93:u64=unsafe{unchecked_shl::<u64,u32>(1,29)};
const S94:u64=unsafe{unchecked_shl::<u64,u32>(1,30)};
const S95:u64=unsafe{unchecked_shl::<u64,u32>(1,31)};
const S96:u64=unsafe{unchecked_shl::<u64,u32>(1,32)};
const S97:u64=unsafe{unchecked_shl::<u64,u32>(1,33)};
const S98:u64=unsafe{unchecked_shl::<u64,u32>(1,34)};
const S99:u64=unsafe{unchecked_shl::<u64,u32>(1,35)};
const S100:u64=unsafe{unchecked_shl::<u64,u32>(1,36)};
const S101:u64=unsafe{unchecked_shl::<u64,u32>(1,37)};
const S102:u64=unsafe{unchecked_shl::<u64,u32>(1,38)};
const S103:u64=unsafe{unchecked_shl::<u64,u32>(1,39)};
const S104:u64=unsafe{unchecked_shl::<u64,u32>(1,40)};
const S105:u64=unsafe{unchecked_shl::<u64,u32>(1,41)};
const S106:u64=unsafe{unchecked_shl::<u64,u32>(1,42)};
const S107:u64=unsafe{unchecked_shl::<u64,u32>(1,43)};
const S108:u64=unsafe{unchecked_shl::<u64,u32>(1,44)};
const S109:u64=unsafe{unchecked_shl::<u64,u32>(1,45)};
const S110:u64=unsafe{unchecked_shl::<u64,u32>(1,46)};
const S111:u64=unsafe{unchecked_shl::<u64,u32>(1,47)};
const S112:u64=unsafe{unchecked_shl::<u64,u32>(1,48)};
const S113:u64=unsafe{unchecked_shl::<u64,u32>(1,49)};
const S114:u64=unsafe{unchecked_shl::<u64,u32>(1,50)};
const S115:u64=unsafe{unchecked_shl::<u64,u32>(1,51)};
const S116:u64=unsafe{unchecked_shl::<u64,u32>(1,52)};
const S117:u64=unsafe{unchecked_shl::<u64,u32>(1,53)};
const S118:u64=unsafe{unchecked_shl::<u64,u32>(1,54)};
const S119:u64=unsafe{unchecked_shl::<u64,u32>(1,55)};
const S120:u64=unsafe{unchecked_shl::<u64,u32>(1,56)};
const S121:u64=unsafe{unchecked_shl::<u64,u32>(1,57)};
const S122:u64=unsafe{unchecked_shl::<u64,u32>(1,58)};
const S123:u64=unsafe{unchecked_shl::<u64,u32>(1,59)};
const S124:u64=unsafe{unchecked_shl::<u64,u32>(1,60)};
const S125:u64=unsafe{unchecked_shl::<u64,u32>(1,61)};
const S126:u64=unsafe{unchecked_shl::<u64,u32>(1,62)};
const S127:u64=unsafe{unchecked_shl::<u64,u32>(1,63)};
