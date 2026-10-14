fn f<T:Copy+'static>(mut x:T,y:T){let r=&x;let z=r;x=y;}
fn main(){}
