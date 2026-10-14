fn id<T:Copy>(x:T)->T{x}fn f<T:Copy>(mut x:T){let r=&mut x;let y=id(x);let z=r;}
fn main(){}
