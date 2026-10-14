fn f<'a,T,const N:usize>(a:&'a [T;N],i:usize)->&'a T{&a[i]}
fn main(){}
