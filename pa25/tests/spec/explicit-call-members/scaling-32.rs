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
pub const fn invoke_8<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_9<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_10<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_11<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_12<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_13<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_14<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_15<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_16<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_17<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_18<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_19<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_20<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_21<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_22<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_23<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_24<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_25<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_26<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_27<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_28<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_29<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_30<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
pub const fn invoke_31<F:[const]FnMut(u8)->u8>(relay:&mut Relay<F>,value:u8)->u8{relay.call_once((value,))}
fn main(){}
