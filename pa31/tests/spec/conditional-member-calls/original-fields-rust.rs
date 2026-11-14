#![feature(const_convert,const_trait_impl)]
#![no_std]
use core::mem::ManuallyDrop;
pub struct Guard<T>{inner:T}
pub const fn field<T>(guard:&ManuallyDrop<Guard<T>>)->&T{&guard.inner}
pub const fn unique<T>(guard:&mut ManuallyDrop<Guard<T>>)->&mut T{&mut guard.inner}
