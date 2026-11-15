#![feature(const_closures,const_trait_impl,const_destruct)]
#![allow(dead_code,unused_variables)]
use core::marker::Destruct;
pub const fn check0<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check1<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check2<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check3<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check4<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check5<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check6<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
pub const fn check7<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F){let _c=const move |a| f(a);}
fn main(){}
