fn check(ok:bool){let d=if ok{1}else{0};let _=1/d;}const fn f()->u8{let p=b"abc" as *const [u8;3] as *mut [u8;3];unsafe{(*p)[0]=1;(*p)[0]}}const N:u8=f();fn main(){let _=N;}
