trait A{type T;} impl A for i32{type T=u8;} type X=Vec<<i32 as A>::T>;
