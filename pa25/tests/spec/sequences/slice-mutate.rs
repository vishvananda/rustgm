fn check(ok:bool){let d=if ok{1}else{0};let _=1/d;}const fn change(x:&mut [u16])->u16{x[1]=13;x[1]}const N:u16={let mut a=[2,7,11];let n=change(&mut a);n+a[1]};fn main(){check(N==26)}
