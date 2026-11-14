#![feature(const_trait_impl,unboxed_closures,fn_traits)]
pub struct Relay<F>{pub f:F}
const impl<F:[const]FnMut(u8)->u8>FnOnce<(u8,)> for &mut Relay<F>{type Output=u8;extern "rust-call" fn call_once(mut self,args:(u8,))->u8{self.call_mut(args)}}
const impl<F:[const]FnMut(u8)->u8>FnMut<(u8,)> for &mut Relay<F>{extern "rust-call" fn call_mut(&mut self,(value,):(u8,))->u8{(self.f)(value)}}
pub const fn invoke_0<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_1<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_2<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_3<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_4<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_5<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_6<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_7<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
fn main(){}
