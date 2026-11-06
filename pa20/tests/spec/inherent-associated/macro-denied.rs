macro_rules! make{()=>{struct S;impl S{type A=u8;}}}make!();fn f(x:S::A)->u8{x}
