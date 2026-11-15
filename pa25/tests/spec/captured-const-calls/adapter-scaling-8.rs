#![feature(const_closures,const_trait_impl,const_destruct)]
#![allow(dead_code,unused_variables)]
use core::marker::Destruct;
pub const fn wrap0<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap1<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap2<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap3<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap4<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap5<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap6<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn wrap7<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
fn main(){}
