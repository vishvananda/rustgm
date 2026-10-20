#[repr(packed)]struct W<T:?Sized>(u8,T);trait A{}impl A for u8{}fn main(){let w=W(2,7u8);let p:&W<dyn A>=&w;}
