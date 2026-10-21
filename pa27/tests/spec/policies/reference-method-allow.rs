struct S;impl S{fn g(&self){}}static mut X:S=S;#[allow(static_mut_refs)]fn f(){unsafe{X.g();}}fn main(){}
