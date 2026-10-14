fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f<const B:bool,const C:char>()->u32{if B{C as u32}else{0}}fn main(){check(f::<true,'🦀'>()==129408);check(f::<false,'A'>()==0);}
