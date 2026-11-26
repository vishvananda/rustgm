#[repr(simd)]pub struct Narrow128([u32;4]);
impl Clone for Narrow128{fn clone(&self)->Self{*self}}impl Copy for Narrow128{}
pub fn missing(value:Narrow128){unsafe{crate::arch::asm!("# {value}",value=in(ymm_reg)value,options(nostack));}}
