union U{word:u32,byte:u8}struct S{prefix:u8,u:U}const X:u32={let mut s=S{prefix:0,u:U{word:287454020}};s.u.byte=170;unsafe{s.u.word}};fn main(){let _=X;}
