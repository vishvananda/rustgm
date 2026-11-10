struct Tail<T:?Sized>{tag:u8,tail:T}
#[repr(packed)]struct Packed{pad:u8,inner:Tail<[u32;2]>}fn main(){let p=Packed{pad:0,inner:Tail{tag:7,tail:[3,5]}};let _=&p.inner as &Tail<[u32]>;}
