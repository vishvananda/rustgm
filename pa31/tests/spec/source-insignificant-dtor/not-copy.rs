#![feature(rustc_attrs)]
#![allow(internal_features,dead_code)]
#[rustc_insignificant_dtor]struct S;fn take(_:S){}fn f(){let s=S;take(s);take(s);}
