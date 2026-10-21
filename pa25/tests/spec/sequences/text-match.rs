fn check(ok:bool){let d=if ok{1}else{0};let _=1/d;}const fn kind(x:&str)->u8{match x{"a"=>1,"é"=>2,_=>3}}const N:u8=kind("é");fn main(){check(N==2)}
