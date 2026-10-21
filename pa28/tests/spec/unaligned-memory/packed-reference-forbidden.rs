#[repr(C,packed)]struct P{a:u8,b:u32}fn main(){let p=P{a:3,b:37};let _=&p.b;}
