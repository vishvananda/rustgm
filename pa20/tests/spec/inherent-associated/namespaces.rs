#![feature(inherent_associated_types)]
#[allow(non_snake_case)]struct S;impl S{type A=u8;fn A()->u8{1}}fn f(x:S::A)->u8{let _=x;S::A()}
