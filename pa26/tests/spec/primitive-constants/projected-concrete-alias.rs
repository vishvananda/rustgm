fn check(b:bool){if !b{let z=0;let _=1/z;}}trait K{type Out;}struct S;impl K for S{type Out=u16;}type W<T>=<T as K>::Out;fn main(){check(W::<S>::MAX==65535u16);}
