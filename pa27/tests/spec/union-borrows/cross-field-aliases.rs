union U{a:u32,b:u32}const X:u32={let mut u=U{a:7};unsafe{{let r=&mut u.a;*r=37;}let first={let r=&u.b;*r};{let r=&mut u.b;*r=41;}let last={let r=&u.a;*r};first+last}};fn main(){let _=X;}
