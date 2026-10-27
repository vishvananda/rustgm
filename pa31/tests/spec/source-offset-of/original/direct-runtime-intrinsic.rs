#![feature(core_intrinsics)]
#[repr(C)]struct S{a:u8,b:u32}
pub fn runtime()->usize{core::intrinsics::offset_of::<S>(0,1)}
