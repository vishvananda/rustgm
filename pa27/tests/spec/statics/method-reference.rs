struct S;impl S{fn g(&self){}}static mut X:S=S;fn f(){unsafe{X.g();}}fn main(){}
