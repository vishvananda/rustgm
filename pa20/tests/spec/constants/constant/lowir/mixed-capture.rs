trait A<T,const N:usize>{const C:[T;N];}struct S<const N:usize>;impl<const N:usize>A<u8,N>for S<N>{const C:[u8;N]=[7;N];}fn main(){}
