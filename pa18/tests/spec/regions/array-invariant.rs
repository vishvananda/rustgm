fn f<'a:'b,'b,T,const N:usize>(x:&'a mut[&'a T;N])->&'b mut[&'b T;N]{x}
fn main(){}
