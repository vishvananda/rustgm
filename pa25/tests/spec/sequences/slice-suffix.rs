fn check(ok:bool){let d=if ok{1}else{0};let _=1/d;}const fn last(x:&[u8])->u8{match x{[..,n]=>*n,_=>0}}const N:u8=last(&[3,5,7]);fn main(){check(N==7)}
