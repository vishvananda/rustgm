fn check(ok:bool){let d=if ok{1}else{0};let _=1/d;}const fn sum(x:[u8;4])->u8{let [_,rest @ ..,_]=x;rest[0]+rest[1]}const N:u8=sum([2,3,5,7]);fn main(){check(N==8)}
