fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn first<T:Copy,const N:usize>(x:[T;N])->T{let f=move||x[0];f()}fn main(){check(first([7u8;2])==7);check(first([900u16;3])==900);}
