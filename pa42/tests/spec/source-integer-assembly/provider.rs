impl Clone for u64 { fn clone(&self)->Self { *self } } impl Copy for u64 {}
impl Clone for i64 { fn clone(&self)->Self { *self } } impl Copy for i64 {}
use crate::arch::asm;
// ORIGINAL___readeflags
// ORIGINAL___writeeflags
pub unsafe fn flags<T>() -> u64 { unsafe { let flags = __readeflags(); __writeeflags(flags); __readeflags() } }
pub fn increment<T>(mut value:u64)->u64 { unsafe { asm!("add {value}, 1", value=inout(reg)value, options(nomem,nostack)); } value }
pub fn late<T>(value:u64)->u64 { let result:u64; unsafe { asm!("mov {result}, {value}", value=in(reg)value, result=lateout(reg)result, options(nomem,nostack,preserves_flags)); } result }
pub fn aliased<T>()->u64 { let mut result:u64; unsafe { asm!("mov {a}, 11", "mov {b}, 22", a=out(reg)result, b=out(reg)result, options(nomem,nostack,preserves_flags)); } result }
pub fn stored<T>(target:&mut u64, value:u64) { unsafe { asm!("mov {result}, {value}", value=in(reg)value, result=out(reg)*target, options(nomem,nostack,preserves_flags)); } }
pub fn att<T>(value:i64)->i64 { let result:i64; unsafe { asm!("mov {value}, {result}", value=in(reg)value, result=out(reg)result, options(nomem,nostack,preserves_flags,att_syntax)); } result }
