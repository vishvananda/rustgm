fn view<'a,'b,T,const N:usize>(a:&'a[T;N],_:&'b())->&'b[T]{let _=a.len();a}fn main(){}
