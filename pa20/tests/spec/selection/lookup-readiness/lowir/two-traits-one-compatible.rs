fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}

trait B{fn value()->usize;}struct X<const N:usize>;impl B for X<{S::C}>{fn value()->usize{3}}struct S;struct U;trait A{const C:usize=3;}trait D{const C:usize=9;}impl A for S{}impl D for U{}fn main(){check(<X<3>as B>::value()==3);}
