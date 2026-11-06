#![feature(inherent_associated_types)]
struct S;trait T{type A;}impl T for S{type A=u8;}fn f(x:<S as T>::A)->u8{x}
