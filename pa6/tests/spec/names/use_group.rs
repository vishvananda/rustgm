mod a{pub fn f(){} pub struct S;} use a::{self as b,f,S};fn h(){f();b::f();S;}
