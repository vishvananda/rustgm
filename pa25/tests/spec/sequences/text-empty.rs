fn check(ok:bool){let d=if ok{1}else{0};let _=1/d;}const fn kind(x:&str)->u8{match x{""=>1,_=>2}}const N:u8=kind("");fn main(){check(N==1)}
