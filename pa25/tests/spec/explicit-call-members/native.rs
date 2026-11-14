#![feature(const_trait_impl,unboxed_closures,fn_traits)]
pub struct Relay<F>{pub f:F}
const impl<F:[const]FnMut(u8)->u8>FnOnce<(u8,)> for &mut Relay<F>{type Output=u8;extern "rust-call" fn call_once(mut self,args:(u8,))->u8{self.call_mut(args)}}
const impl<F:[const]FnMut(u8)->u8>FnMut<(u8,)> for &mut Relay<F>{extern "rust-call" fn call_mut(&mut self,(value,):(u8,))->u8{(self.f)(value)}}
pub const fn pass(value:u8)->u8{value}
pub const VALUE:u8={let mut relay=Relay{f:pass};(&mut relay).call_once((41u8,))};
#[unsafe(no_mangle)]pub extern "C" fn entry()->u8{VALUE}
#[unsafe(no_mangle)]pub extern "C" fn probe(value:u8)->u8{let mut relay=Relay{f:pass};(&mut relay).call_once((value,))}
fn main(){}
