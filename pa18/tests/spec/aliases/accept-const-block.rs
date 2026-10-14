type A<T,const N:usize>=[T;{N}];type B<U,const M:usize>=A<U,{M}>;fn f(x:B<u8,3>)->[u8;3]{x}
fn main(){}
