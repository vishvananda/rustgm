#[derive(Clone,Copy)]union U{word:u32,byte:u8}const X:(u32,u32)={let mut u=U{word:287454020};let v=u;u.byte=170;unsafe{(u.word,v.word)}};fn main(){let _=X;}
