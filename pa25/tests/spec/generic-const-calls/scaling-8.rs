#![feature(const_trait_impl,const_destruct,const_cmp)]
use core::marker::Destruct;
pub const fn apply0<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply1<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply2<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply3<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply4<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply5<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply6<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
pub const fn apply7<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x);f(x)}
fn main(){}
