#[derive(Copy,Clone)]union U{a:u32,b:u32}const X:u32={let mut u=U{a:7};let old=u;unsafe{{let r=&mut u.a;*r=37;}let x={let r=&old.b;*r};let y={let r=&u.b;*r};x+y}};fn main(){let _=X;}
