trait Has{type Meta:Copy;}const fn consume<T:Has>(x:T::Meta){}struct One;impl Has for One{type Meta=u8;}const ANSWER:u8={consume::<One>(7);7};fn main(){if ANSWER!=7{let x=0u8;let _=1/x;}}
