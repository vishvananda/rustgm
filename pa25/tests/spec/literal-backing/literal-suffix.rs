fn check(ok:bool){let d=if ok{1}else{0};let _=1/d;}const fn f(x:&[u8])->u8{match x{[_,..,z]=>*z,_=>0}}const N:u8=f(b"abc");fn main(){check(N==99)}
