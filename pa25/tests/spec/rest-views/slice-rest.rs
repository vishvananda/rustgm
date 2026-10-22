fn check(ok:bool){let d=if ok{1}else{0};let _=1/d;}const fn second(x:&[u8])->u8{match x{[_,rest @ ..]=>rest[0],_=>0}}const N:u8=second(&[3,5,7]);fn main(){check(N==5)}
