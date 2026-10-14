struct N;#[derive(Clone,Copy)]struct S<T>(T);fn main(){let a=S(N);let b=a;let c=a;}
