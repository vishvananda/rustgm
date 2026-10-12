trait A{type T;} impl A for i32{type T=u8;} type X=<<i32 as A>::T as core::ops::Add>::Output;
