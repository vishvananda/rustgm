#![feature(derive_const,const_trait_impl)]
#[derive_const(Default)]enum Const<T>{#[default]First,Payload(T)}
