struct S{n:u8}fn main(){let _=match 0u8{#[cfg(unix)] _ if let S{n}=S{n:7}&&n==7=>n,_=>99};}
