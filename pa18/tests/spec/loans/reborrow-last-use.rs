fn f<T>(x:&mut T){let r=&mut*x;let s=&*r;let use_s=s;let again=&mut*x;}
fn main(){}
