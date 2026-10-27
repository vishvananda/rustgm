#[repr(C)]struct S{a:u8,b:u32}
#[cfg(any())]fn f(){let _=builtin # offset_of(S,b);}
