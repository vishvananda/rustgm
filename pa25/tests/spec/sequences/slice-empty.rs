fn check(ok:bool){let d=if ok{1}else{0};let _=1/d;}const fn empty(x:&[u8])->u8{match x{[]=>1,_=>0}}const N:u8=empty(&[]);fn main(){check(N==1)}
