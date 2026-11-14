trait Has{type Meta;}const fn consume<T:Has>(x:T::Meta){}struct One;impl Has for One{type Meta=u8;}const ANSWER:u8={consume::<One>(7);7};fn main(){let _=ANSWER;}
