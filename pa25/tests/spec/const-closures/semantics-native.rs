#![feature(const_closures,const_trait_impl,const_destruct)]
const fn shared(value:u8)->u8 {let call=const |extra:u8|value+extra;call(2)}
const fn mutable(mut value:u8)->u8 {
    let mut call=const |extra:u8|{value+=extra;value};
    let first=call(2);call(first)
}
struct Owned(u8);
const fn once<F:[const]FnOnce()->Owned>(call:F)->Owned {call()}
const fn consumed(value:u8)->u8 {let owned=Owned(value);once(const move ||owned).0}
struct Guard<'a>(&'a mut u8);
const impl Drop for Guard<'_>{fn drop(&mut self){*self.0+=1;}}
const fn cleanup()->u8 {
    let mut count=0;
    {let guard=Guard(&mut count);let call=const move ||{let _owned=guard;};call();}
    count
}
const SHARED:u8=shared(21);
const MUTABLE:u8=mutable(11);
const ONCE:u8=consumed(23);
const DROPS:u8=cleanup();
const fn make(value:u8)->impl Fn(u8)->u8 {const move |extra:u8|value+extra}
fn main(){
    let value: u8=std::env::args().nth(1).unwrap().parse().unwrap();
    let runtime=make(value)(2);
    println!("closure {SHARED} {MUTABLE} {ONCE} {DROPS} {runtime}");
    assert_eq!((SHARED,MUTABLE,ONCE,DROPS,runtime),(23,26,23,1,25));
}
