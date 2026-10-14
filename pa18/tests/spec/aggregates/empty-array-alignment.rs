fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
#[repr(C)]struct S<T>{empty:[T;0],value:u8}fn main(){let a=S::<u64>{empty:[],value:7};check(a.value==7);}
