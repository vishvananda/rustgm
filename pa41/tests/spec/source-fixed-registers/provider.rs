impl Clone for u32 { fn clone(&self)->Self { *self } } impl Copy for u32 {}
impl Clone for i32 { fn clone(&self)->Self { *self } } impl Copy for i32 {}
use crate::arch::asm;
// ORIGINAL_RESULT
pub fn query<T>(leaf:u32, sub_leaf:u32)->CpuidResult {
// ORIGINAL_CPUID_TAIL
pub fn increment<T>(mut value:i32)->i32 { unsafe { asm!("add eax, 1", inlateout("eax")value, options(nomem,nostack)); } value }
pub fn ordinary<T>(value:u32)->u32 { value }
pub fn late_split<T>(value:u32)->u32 { let result:u32; unsafe { asm!("add eax, 1", in("eax")value, lateout("eax")result, options(nomem,nostack)); } result }
pub fn late_reverse<T>(value:u32)->u32 { let result:u32; unsafe { asm!("add eax, 1", lateout("eax")result, in("eax")value, options(nomem,nostack)); } result }
