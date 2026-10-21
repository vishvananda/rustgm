union U<T:Copy>{value:T,byte:u8}const X:u32=unsafe{U{value:37u32}.value};fn main(){let _=X;}
