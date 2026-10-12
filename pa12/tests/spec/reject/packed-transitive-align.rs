#[repr(align(16))]struct A;struct B(A);#[repr(packed)]struct S(B);fn main(){}
