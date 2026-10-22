fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}type P<T>=*const T;fn main(){let a=[1u8,7];let p=&raw const a as *const u8;check(unsafe{*P::<u8>::add(p,1)}==7);}
