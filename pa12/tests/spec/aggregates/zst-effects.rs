fn check(ok:bool){if !ok{let _=failure(0);}} fn failure(x:i32)->i32{1/x}
#[derive(Clone,Copy)]struct Z;fn main(){let mut n=0;let a=[{n+=1;Z};0];let b=[{n+=1;Z};1];let c=[{n+=1;Z};18446744073709551615];check(n==3);let _=(a,b,c);}
