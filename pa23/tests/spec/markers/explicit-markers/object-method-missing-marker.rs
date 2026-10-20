trait A{fn f(&self)->u8 where Self:Send{7}}struct W;unsafe impl Send for W{}impl A for W{}fn call(p:&dyn A)->u8{p.f()}
fn main(){}
