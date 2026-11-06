#![feature(const_closures, const_trait_impl, const_destruct)]
struct Guard<'a>(&'a mut u8);
const impl Drop for Guard<'_>{fn drop(&mut self){*self.0+=1;}}
const fn compute()->u8 {
    let mut count=0;
    {let guard=Guard(&mut count);let call=const move || {let _owned=guard;};call();}
    count
}
pub const ANSWER:u8=compute();
