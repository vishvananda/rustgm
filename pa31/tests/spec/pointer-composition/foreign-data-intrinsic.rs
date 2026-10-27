#![feature(ptr_metadata,extern_types,core_intrinsics)]
#![allow(dead_code,internal_features)]
use core::{ptr,slice};
static DATA:[u8;3]=[7,8,9];
unsafe extern "C"{type Foreign;}const UNIT:()=ptr::metadata(core::intrinsics::aggregate_raw_ptr::<*const Foreign,*const Foreign,()>(1usize as *const Foreign,()));#[unsafe(no_mangle)]pub extern "C" fn answer(){UNIT}
